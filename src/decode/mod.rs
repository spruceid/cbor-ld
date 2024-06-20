use core::fmt;

use crate::{
    contexts::REGISTERED_CONTEXTS,
    transform::{TransformedValue, Transformer, TransformerState},
    CborObject, CborValue, Codecs, CompressionMode, IdMap, JsonObject, JsonValue,
};

mod error;
pub use error::*;
use iref::{IriBuf, IriRef, IriRefBuf};
use rdf_types::BlankIdBuf;

#[derive(Debug)]
pub struct DecodeOptions {
    pub context_map: IdMap,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            context_map: IdMap::new_derived(Some(&REGISTERED_CONTEXTS)),
        }
    }
}

pub async fn decode<L>(cbor_ld_document: &CborValue, loader: L) -> Result<JsonValue, DecodeError>
where
    L: json_ld::Loader,
    L::Error: fmt::Display,
{
    decode_with(cbor_ld_document, loader, Default::default()).await
}

pub async fn decode_with<L>(
    cbor_ld_document: &CborValue,
    loader: L,
    options: DecodeOptions,
) -> Result<JsonValue, DecodeError>
where
    L: json_ld::Loader,
    L::Error: fmt::Display,
{
    match cbor_ld_document {
        CborValue::Tag(tag, value) => match CompressionMode::from_tag(*tag)? {
            CompressionMode::Uncompressed => {
                todo!()
            }
            CompressionMode::Version1 => {
                let mut decoder = Decoder::new(loader, options.context_map, Default::default());
                decoder.decode(value).await
            }
        },
        _ => Err(DecodeError::NotCborLd),
    }
}

pub async fn decode_from_bytes<L>(bytes: &[u8], loader: L) -> Result<JsonValue, DecodeError>
where
    L: json_ld::Loader,
    L::Error: fmt::Display,
{
    decode_from_bytes_with(bytes, loader, Default::default()).await
}

pub async fn decode_from_bytes_with<L>(
    bytes: &[u8],
    loader: L,
    options: DecodeOptions,
) -> Result<JsonValue, DecodeError>
where
    L: json_ld::Loader,
    L::Error: fmt::Display,
{
    let cbor_ld_document = ciborium::from_reader(bytes)?;
    decode_with(&cbor_ld_document, loader, options).await
}

pub struct Decoder<L> {
    loader: L,
    state: TransformerState,
}

impl<L> Decoder<L> {
    pub fn new(loader: L, application_context_map: IdMap, codecs: Codecs) -> Self {
        Self {
            loader,
            state: TransformerState::new(application_context_map, codecs),
        }
    }
}

impl<L> Decoder<L>
where
    L: json_ld::Loader,
    L::Error: fmt::Display,
{
    pub async fn decode(&mut self, json_ld_document: &CborValue) -> Result<JsonValue, DecodeError> {
        let active_context = json_ld::Context::new(None);
        self.transform(&active_context, json_ld_document).await
    }

    fn decode_vocab_term(
        &self,
        active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &CborValue,
    ) -> Result<JsonValue, DecodeError> {
        Ok(JsonValue::String(
            self.state.decode_vocab_term(active_context, value)?.into(),
        ))
    }
}

impl<L> Transformer for Decoder<L>
where
    L: json_ld::Loader,
    L::Error: fmt::Display,
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
        let i = value
            .as_integer()
            .ok_or(DecodeError::InvalidVocabTermKind)?;

        let i = u64::try_from(i).map_err(|_| DecodeError::MissingTermFor(value.clone()))?;

        Ok(self
            .state
            .context_map
            .get_term(i)
            .ok_or_else(|| DecodeError::MissingTermFor(value.clone()))?
            .parse()
            .unwrap())
    }

    fn context_id(
        &self,
        _value: &Self::Input,
        iri_ref: &IriRef,
    ) -> Result<Self::Output, Self::Error> {
        Ok(JsonValue::String(iri_ref.as_str().into()))
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

        let i = u64::try_from(i).map_err(|_| DecodeError::MissingTermFor(key.clone()))?;

        Ok(self.state.allocator.decode_term(i))
    }

    fn value_term<'a>(&'a self, value: &'a Self::Input) -> Result<&'a str, Self::Error> {
        let i = value
            .as_integer()
            .ok_or(DecodeError::InvalidVocabTermKind)?;

        let i = u64::try_from(i).map_err(|_| DecodeError::MissingTermFor(value.clone()))?;

        self.state
            .allocator
            .decode_term(i)
            .map(|(term, _)| term)
            .ok_or_else(|| DecodeError::MissingTermFor(value.clone()))
    }

    fn transform_id(&self, value: &Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(JsonValue::String(
            self.state.codecs.iri.decode(value)?.into_string().into(),
        ))
    }

    fn transform_vocab(
        &self,
        active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        self.decode_vocab_term(active_context, value)
    }

    fn state_and_loader_mut(&mut self) -> (&mut TransformerState, &mut Self::Loader) {
        (&mut self.state, &mut self.loader)
    }

    fn transform_typed_value(
        &mut self,
        active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &Self::Input,
        type_: Option<&json_ld::Type<IriBuf>>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        if value.is_object() {
            Ok(None)
        } else {
            match type_ {
                Some(type_) => match self.state.codecs.type_.get(type_) {
                    Some(codec) => codec
                        .decode(&self.state, active_context, value)
                        .map(|ty| Some(JsonValue::String(ty.into()))),
                    None => Ok(None),
                },
                None => Ok(None),
            }
        }
    }

    async fn transform_object(
        &mut self,
        active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
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