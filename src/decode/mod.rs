use std::borrow::Cow;

use crate::{
    transform::{TransformedValue, Transformer, TransformerState},
    CborObject, CborValue, Codecs, CompressionMode, JsonObject, JsonValue, Tables,
    CBOR_LD_TAG_HIGH,
};

mod error;
pub use error::*;
use iref::{IriBuf, IriRef, IriRefBuf};

/// Decoding options.
#[derive(Debug, Default)]
pub struct DecodeOptions<'a> {
    // /// Map associating JSON-LD context URLs to CBOR-LD (integer) identifiers.
    // pub context_map: IdMap,
    /// Datatype codecs.
    pub codecs: Codecs,

    /// Tables.
    pub default_tables: Cow<'a, Tables>,
}

/// Decodes a CBOR-LD document using the given JSON-LD context loader and the
/// default options.
pub async fn decode(
    cbor_ld_document: &CborValue,
    loader: impl json_ld::Loader,
) -> Result<JsonValue, DecodeError> {
    decode_with(cbor_ld_document, loader, Default::default()).await
}

/// Decodes a CBOR-LD document using the given JSON-LD context loader and the
/// given options.
pub async fn decode_with(
    cbor_ld_document: &CborValue,
    loader: impl json_ld::Loader,
    options: DecodeOptions<'_>,
) -> Result<JsonValue, DecodeError> {
    match cbor_ld_document {
        CborValue::Tag(tag, value) => {
            if tag >> 8 != CBOR_LD_TAG_HIGH as u64 {
                return Err(DecodeError::NotCborLd);
            }

            let varint_high = (tag & 0xff) as u8;

            let compression_mode = if varint_high >= 128 {
                todo!()
            } else {
                CompressionMode::from_id(varint_high as u64)
            };

            match compression_mode {
                CompressionMode::Uncompressed => todo!("uncompressed"),
                CompressionMode::Compressed(registry_entry) => {
                    let tables = registry_entry.tables(options.default_tables)?;
                    let mut decoder = Decoder::new(loader, options.codecs, tables);
                    decoder.decode(value).await
                }
            }
        }
        _ => Err(DecodeError::NotCborLd),
    }
}

/// Decodes a CBOR-LD document bytes using the given JSON-LD context loader and
/// the default options.
pub async fn decode_from_bytes(
    bytes: &[u8],
    loader: impl json_ld::Loader,
) -> Result<JsonValue, DecodeError> {
    decode_from_bytes_with(bytes, loader, Default::default()).await
}

/// Decodes a CBOR-LD document bytes using the given JSON-LD context loader and
/// the given options.
pub async fn decode_from_bytes_with(
    bytes: &[u8],
    loader: impl json_ld::Loader,
    options: DecodeOptions<'_>,
) -> Result<JsonValue, DecodeError> {
    let cbor_ld_document = ciborium::from_reader(bytes)?;
    decode_with(&cbor_ld_document, loader, options).await
}

/// CBOR-LD decoder.
pub struct Decoder<'a, L> {
    loader: L,
    state: TransformerState<'a>,
}

impl<'a, L> Decoder<'a, L> {
    pub fn new(
        loader: L,
        // application_context_map: IdMap,
        codecs: Codecs,
        tables: Cow<'a, Tables>,
    ) -> Self {
        Self {
            loader,
            state: TransformerState::new(
                // application_context_map,
                codecs, tables,
            ),
        }
    }
}

impl<'a, L> Decoder<'a, L>
where
    L: json_ld::Loader,
{
    pub async fn decode(&mut self, json_ld_document: &CborValue) -> Result<JsonValue, DecodeError> {
        let active_context = json_ld::Context::new(None);
        self.transform(&active_context, json_ld_document).await
    }

    fn decode_vocab_term(
        &self,
        active_context: &json_ld::Context,
        value: &CborValue,
    ) -> Result<String, DecodeError> {
        self.state.decode_vocab_term(active_context, value)
    }
}

impl<'t, L> Transformer<'t> for Decoder<'t, L>
where
    L: json_ld::Loader,
{
    type Input = CborValue;
    type Output = JsonValue;

    type InputObject = CborObject;
    type OutputObject = JsonObject;

    type InputKey = CborValue;
    type OutputKey = json_ld::syntax::object::Key;

    type Loader = L;
    type Error = DecodeError;

    fn context_iri_ref(&self, value: &Self::Input) -> Result<IriRefBuf, Self::Error> {
        match value {
            CborValue::Integer(i) => {
                let i = u64::try_from(*i)
                    .map_err(|_| DecodeError::UndefinedCompressedContext(value.clone()))?;

                self.state
                    .tables
                    .context
                    .get_iri_ref(i)
                    .ok_or_else(|| DecodeError::UndefinedCompressedContext(value.clone()))
                    .map(ToOwned::to_owned)
            }
            CborValue::Text(t) => {
                IriRefBuf::new(t.clone()).map_err(|e| DecodeError::InvalidContextIriRef(e.0))
            }
            _ => Err(DecodeError::InvalidContextTermKind),
        }
    }

    fn context_id(&self, _value: &Self::Input, iri_ref: &IriRef) -> Self::Output {
        JsonValue::String(iri_ref.as_str().into())
    }

    fn term_key(&self, term: &str, _plural: bool) -> Result<Self::OutputKey, Self::Error> {
        Ok(term.into())
    }

    fn term_value(&self, term: &str) -> Result<Self::Output, Self::Error> {
        Ok(JsonValue::String(term.into()))
    }

    fn key_term<'a>(
        &'a self,
        key: &'a Self::InputKey,
        _value: &Self::Input,
    ) -> Result<Option<(&'a str, bool)>, Self::Error> {
        let i = key.as_integer().ok_or(DecodeError::InvalidVocabTermKind)?;

        let i = u64::try_from(i).map_err(|_| DecodeError::UndefinedCompressedTerm(key.clone()))?;

        Ok(self.state.allocator.decode_term(i))
    }

    fn value_term<'a>(
        &'a self,
        active_context: &json_ld::Context,
        value: &'a Self::Input,
    ) -> Result<Cow<'a, str>, Self::Error> {
        self.decode_vocab_term(active_context, value)
            .map(Cow::Owned)
    }

    fn transform_id(&self, value: &Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(JsonValue::String(
            self.state.codecs.iri.decode(value)?.into_string().into(),
        ))
    }

    fn transform_vocab(
        &self,
        active_context: &json_ld::Context,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        self.decode_vocab_term(active_context, value)
            .map(|v| JsonValue::String(v.into()))
    }

    fn state_and_loader_mut(&mut self) -> (&mut TransformerState<'t>, &mut Self::Loader) {
        (&mut self.state, &mut self.loader)
    }

    fn transform_typed_value(
        &mut self,
        active_context: &json_ld::Context,
        value: &Self::Input,
        type_: Option<&json_ld::Type<IriBuf>>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        if value.is_object() {
            Ok(None)
        } else {
            match type_ {
                Some(type_) => match self.state.tables.types.get(type_) {
                    Some(t) => t.decode(value).map(Some),
                    None => match self.state.codecs.type_.get(type_) {
                        Some(codec) => codec
                            .decode(&self.state, active_context, value)
                            .map(|ty| Some(JsonValue::String(ty.into()))),
                        None => Ok(None),
                    },
                },
                None => Ok(None),
            }
        }
    }

    async fn transform_object(
        &mut self,
        active_context: &json_ld::Context,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        match value {
            CborValue::Null => Ok(JsonValue::Null),
            CborValue::Bool(b) => Ok(JsonValue::Boolean(*b)),
            CborValue::Integer(n) => {
                let n: i128 = (*n).into();
                Ok(JsonValue::Number(n.to_string().parse().unwrap()))
            }
            CborValue::Float(f) => Ok(JsonValue::Number(
                (*f).try_into().map_err(|_| DecodeError::NonFiniteFloat)?,
            )),
            CborValue::Text(s) => Ok(JsonValue::String(s.as_str().into())),
            CborValue::Array(array) => {
                let mut json_array = Vec::with_capacity(array.len());

                for item in array {
                    json_array.push(Box::pin(self.transform(active_context, item)).await?);
                }

                Ok(JsonValue::Array(json_array))
            }
            CborValue::Map(object) => Ok(JsonValue::Object(
                Box::pin(self.transform_node(active_context, object)).await?,
            )),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}
