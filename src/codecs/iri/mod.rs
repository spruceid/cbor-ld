use crate::{CborValue, DecodeError, EncodeError};
use core::fmt;
use iref::{Iri, IriBuf};
use std::collections::HashMap;

mod url;
pub use url::*;

mod urn;
pub use urn::*;

mod did;
pub use did::*;

pub trait IriCodec {
    fn encode(&self, suffix: &str) -> Result<Vec<CborValue>, EncodeError>;

    fn decode(&self, array: &[CborValue]) -> Result<String, DecodeError>;
}

pub struct IriCodecs {
    codecs: Vec<Box<dyn IriCodec>>,
    by_prefix: HashMap<String, (u64, usize)>,
    by_id: HashMap<u64, (String, usize)>,
}

impl IriCodecs {
    pub fn new() -> Self {
        Self {
            codecs: Vec::new(),
            by_prefix: HashMap::new(),
            by_id: HashMap::new(),
        }
    }

    pub fn get_for_iri<'a>(&self, iri: &'a str) -> Option<(&'a str, u64, &dyn IriCodec)> {
        for (prefix, &(id, i)) in &self.by_prefix {
            if let Some(suffix) = iri.strip_prefix(prefix) {
                return Some((suffix, id, &*self.codecs[i]));
            }
        }

        None
    }

    pub fn get_by_id(&self, id: u64) -> Option<(&str, &dyn IriCodec)> {
        self.by_id
            .get(&id)
            .map(|(prefix, i)| (prefix.as_str(), &*self.codecs[*i]))
    }

    pub fn insert(&mut self, scheme: String, id: u64, codec: impl 'static + IriCodec) {
        let i = self.codecs.len();
        self.codecs.push(Box::new(codec));

        self.by_prefix.insert(format!("{scheme}:"), (id, i));
        self.by_id.insert(id, (scheme, i));
    }

    pub fn encode(&self, iri: &Iri) -> Result<CborValue, EncodeError> {
        match self.get_for_iri(iri.as_str()) {
            Some((suffix, id, codec)) => {
                let mut array = vec![CborValue::Integer(id.into())];
                array.extend(codec.encode(suffix)?);
                Ok(CborValue::Array(array))
            }
            None => Ok(CborValue::Text(iri.as_str().to_owned())),
        }
    }

    pub fn decode(&self, value: &CborValue) -> Result<IriBuf, DecodeError> {
        let text = match value {
            CborValue::Array(array) => {
                if array.is_empty() {
                    return Err(DecodeError::Codec("iri", "missing IRI type".to_owned()));
                }

                let id: u64 = array[0]
                    .as_integer()
                    .ok_or_else(|| {
                        DecodeError::Codec(
                            "iri",
                            "invalid IRI codec ID: expected integer".to_owned(),
                        )
                    })?
                    .try_into()
                    .map_err(|_| DecodeError::Codec("iri", "unknown IRI codec ID".to_owned()))?;

                let (prefix, codec) = self
                    .get_by_id(id)
                    .ok_or_else(|| DecodeError::Codec("iri", "unknown IRI codec ID".to_owned()))?;

                let suffix = codec.decode(&array[1..])?;

                format!("{prefix}:{suffix}")
            }
            CborValue::Text(text) => text.clone(),
            _ => {
                return Err(DecodeError::Codec(
                    "iri",
                    "expected text or array".to_owned(),
                ))
            }
        };

        IriBuf::new(text).map_err(|_| DecodeError::Codec("iri", "invalid IRI".to_owned()))
    }
}

impl fmt::Debug for IriCodecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IriCodecs").finish()
    }
}

impl Default for IriCodecs {
    fn default() -> Self {
        let mut result = Self::new();

        result.insert("http".to_owned(), 1, UrlCodec);
        result.insert("https".to_owned(), 2, UrlCodec);
        result.insert("urn:uuid".to_owned(), 3, UrnUuidCodec);
        result.insert("did:v1:nym".to_owned(), 1024, Base58DidMethodCodec);
        result.insert("did:key".to_owned(), 1025, Base58DidMethodCodec);

        result
    }
}
