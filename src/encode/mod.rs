use crate::{
    contexts::REGISTERED_CONTEXTS,
    transform::{Transformer, TransformerState},
    CborObject, CborValue, Codecs, CompressionMode, IdMap, JsonObject, JsonValue,
};
use iref::{Iri, IriBuf, IriRef, IriRefBuf};
mod error;
pub use error::*;

/// Encoding options.
#[derive(Debug)]
pub struct EncodeOptions {
    /// Compression mode.
    pub compression_mode: CompressionMode,

    /// Map associating JSON-LD context URLs to CBOR-LD (integer) identifiers.
    pub context_map: IdMap,

    /// Datatype codecs.
    pub codecs: Codecs,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            compression_mode: CompressionMode::Version1,
            context_map: IdMap::new_derived(Some(&REGISTERED_CONTEXTS)),
            codecs: Default::default(),
        }
    }
}

/// Encodes a JSON-LD document into CBOR-LD using the given JSON-LD context
/// loader and the default options.
pub async fn encode(
    json_ld_document: &json_ld::syntax::Value,
    loader: impl json_ld::Loader,
) -> Result<CborValue, EncodeError> {
    encode_with(json_ld_document, loader, Default::default()).await
}

/// Encodes a JSON-LD document into CBOR-LD using the given JSON-LD context
/// loader and the given options.
pub async fn encode_with(
    json_ld_document: &json_ld::syntax::Value,
    loader: impl json_ld::Loader,
    options: EncodeOptions,
) -> Result<CborValue, EncodeError> {
    let cbor_value = match options.compression_mode {
        CompressionMode::Uncompressed => {
            todo!()
        }
        CompressionMode::Version1 => {
            let mut compressor = Encoder::new(loader, options.context_map, options.codecs);

            compressor.encode(json_ld_document).await
        }
    }?;

    Ok(CborValue::Tag(
        options.compression_mode.to_tag(),
        Box::new(cbor_value),
    ))
}

/// Encodes a JSON-LD document into CBOR-LD bytes using the given JSON-LD
/// context loader and the default options.
pub async fn encode_to_bytes(
    json_ld_document: &json_ld::syntax::Value,
    loader: impl json_ld::Loader,
) -> Result<Vec<u8>, EncodeError> {
    encode_to_bytes_with(json_ld_document, loader, Default::default()).await
}

/// Encodes a JSON-LD document into CBOR-LD bytes using the given JSON-LD
/// context loader and the given options.
pub async fn encode_to_bytes_with(
    json_ld_document: &json_ld::syntax::Value,
    loader: impl json_ld::Loader,
    options: EncodeOptions,
) -> Result<Vec<u8>, EncodeError> {
    encode_with(json_ld_document, loader, options)
        .await
        .map(cbor_into_bytes)
}

pub fn cbor_into_bytes(cbor: CborValue) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::into_writer(&cbor, &mut bytes).unwrap();
    bytes
}

pub struct Encoder<L> {
    loader: L,
    state: TransformerState,
}

impl<L> Encoder<L> {
    pub fn new(loader: L, application_context_map: IdMap, codecs: Codecs) -> Self {
        Self {
            loader,
            state: TransformerState::new(application_context_map, codecs),
        }
    }
}

impl<L> Encoder<L>
where
    L: json_ld::Loader,
{
    pub async fn encode(&mut self, json_ld_document: &JsonValue) -> Result<CborValue, EncodeError> {
        let active_context = json_ld::Context::new(None);
        self.transform(&active_context, json_ld_document).await
    }

    fn encode_vocab_term(
        &self,
        active_context: &json_ld::Context,
        value: &JsonValue,
    ) -> Result<CborValue, EncodeError> {
        let value = value.as_str().ok_or(EncodeError::InvalidVocabTermKind)?;
        self.state.encode_vocab_term(active_context, value)
    }
}

impl<L> Transformer for Encoder<L>
where
    L: json_ld::Loader,
{
    type Input = JsonValue;
    type Output = CborValue;

    type InputObject = JsonObject;
    type OutputObject = CborObject;

    type InputKey = json_ld::syntax::object::Key;
    type OutputKey = CborValue;

    type Loader = L;
    type Error = EncodeError;

    fn context_iri_ref(&self, value: &Self::Input) -> Result<IriRefBuf, Self::Error> {
        value
            .as_str()
            .ok_or(EncodeError::InvalidContextEntry)?
            .parse()
            .map_err(|_| EncodeError::InvalidContextEntry)
    }

    fn context_id(&self, _value: &Self::Input, iri_ref: &IriRef) -> Self::Output {
        match self.state.context_map.get_id(iri_ref) {
            Some(id) => CborValue::Integer(id.into()),
            None => CborValue::Text(iri_ref.as_str().to_owned()),
        }
    }

    fn term_key(&self, term: &str, plural: bool) -> Result<Self::OutputKey, Self::Error> {
        let term_id = self
            .state
            .allocator
            .encode_term(term, plural)
            .ok_or_else(|| EncodeError::MissingIdFor(term.to_string()))?;

        Ok(CborValue::Integer(term_id.into()))
    }

    fn term_value(&self, term: &str) -> Result<Self::Output, Self::Error> {
        self.term_key(term, false)
    }

    fn key_term<'a>(
        &'a self,
        key: &'a Self::InputKey,
        value: &Self::Input,
    ) -> Result<Option<(&'a str, bool)>, Self::Error> {
        Ok(Some((key.as_str(), value.is_array())))
    }

    fn value_term<'a>(&'a self, value: &'a Self::Input) -> Result<&'a str, Self::Error> {
        value.as_str().ok_or(EncodeError::InvalidVocabTermKind)
    }

    fn transform_id(&self, value: &Self::Input) -> Result<Self::Output, Self::Error> {
        let id = value.as_str().ok_or(EncodeError::InvalidIdKind)?;
        let id = Iri::new(id).map_err(|_| EncodeError::InvalidId(id.to_owned()))?;
        self.state.codecs.iri.encode(id)
    }

    fn transform_vocab(
        &self,
        active_context: &json_ld::Context,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        self.encode_vocab_term(active_context, value)
    }

    fn state_and_loader_mut(&mut self) -> (&mut TransformerState, &mut Self::Loader) {
        (&mut self.state, &mut self.loader)
    }

    fn transform_typed_value(
        &mut self,
        active_context: &json_ld::Context,
        value: &Self::Input,
        type_: Option<&json_ld::Type<IriBuf>>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        match value {
            JsonValue::String(value) => match type_ {
                Some(type_) => match self.state.codecs.type_.get(type_) {
                    Some(codec) => codec.encode(&self.state, active_context, value).map(Some),
                    None => Ok(None),
                },
                None => Ok(None),
            },
            _ => Ok(None),
        }
    }

    async fn transform_object(
        &mut self,
        active_context: &json_ld::Context,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        match value {
            JsonValue::Null => Ok(CborValue::Null),
            JsonValue::Boolean(b) => Ok(CborValue::Bool(*b)),
            JsonValue::Number(n) => match n.as_u64() {
                Some(u) => Ok(CborValue::Integer(u.into())),
                None => match n.as_i64() {
                    Some(i) => Ok(CborValue::Integer(i.into())),
                    None => Ok(CborValue::Float(n.as_f64_lossy())),
                },
            },
            JsonValue::String(s) => Ok(CborValue::Text(s.as_str().to_owned())),
            JsonValue::Array(array) => {
                let mut cbor_array = Vec::with_capacity(array.len());

                for item in array {
                    cbor_array.push(Box::pin(self.transform(active_context, item)).await?);
                }

                Ok(CborValue::Array(cbor_array))
            }
            JsonValue::Object(object) => Ok(CborValue::Map(
                Box::pin(self.transform_node(active_context, object)).await?,
            )),
        }
    }
}
