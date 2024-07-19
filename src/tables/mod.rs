use iref::{Iri, IriRef, IriRefBuf};
use json_ld::Type;
use std::collections::HashMap;

mod registry;
pub use registry::*;

use crate::{CborValue, DecodeError, JsonValue};

#[derive(Debug, Default, Clone)]
pub struct Tables {
    pub context: ContextTable,
    pub types: HashMap<Type, TypeTable>,
}

#[derive(Debug, Default, Clone)]
pub struct ContextTable {
    forward: HashMap<IriRefBuf, u64>,
    backward: HashMap<u64, IriRefBuf>,
}

impl ContextTable {
    pub fn get_id(&self, iri_ref: &IriRef) -> Option<u64> {
        self.forward.get(iri_ref).copied()
    }

    pub fn get_iri_ref(&self, i: u64) -> Option<&IriRef> {
        self.backward.get(&i).map(IriRefBuf::as_iri_ref)
    }

    pub fn insert(&mut self, iri: IriRefBuf, i: u64) {
        self.forward.insert(iri.clone(), i);
        self.backward.insert(i, iri);
    }
}

impl<'a> FromIterator<(&'a Iri, u64)> for ContextTable {
    fn from_iter<T: IntoIterator<Item = (&'a Iri, u64)>>(iter: T) -> Self {
        let mut result = Self::default();

        for (k, v) in iter {
            result.insert(k.to_owned().into(), v);
        }

        result
    }
}

#[derive(Debug, Default, Clone)]
pub struct TypeTable {
    forward: HashMap<String, u64>,
    backward: HashMap<u64, String>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_id(&self, value: &str) -> Option<u64> {
        self.forward.get(value).copied()
    }

    pub fn get_value(&self, id: u64) -> Option<&str> {
        self.backward.get(&id).map(String::as_str)
    }

    pub fn encode(&self, value: &str) -> CborValue {
        match self.get_id(value) {
            Some(id) => CborValue::Integer(id.into()),
            None => CborValue::Text(value.to_owned()),
        }
    }

    pub fn decode(&self, value: &CborValue) -> Result<JsonValue, DecodeError> {
        match value {
            CborValue::Integer(id) => {
                let id: u64 = (*id).try_into().map_err(|_| DecodeError::InvalidValue)?;
                let value = self.get_value(id).ok_or(DecodeError::InvalidValue)?;
                Ok(JsonValue::String(value.into()))
            }
            _ => Err(DecodeError::InvalidValue),
        }
    }

    pub fn insert(&mut self, value: String, id: u64) {
        self.forward.insert(value.clone(), id);
        self.backward.insert(id, value);
    }
}

impl<'a> FromIterator<(&'a str, u64)> for TypeTable {
    fn from_iter<T: IntoIterator<Item = (&'a str, u64)>>(iter: T) -> Self {
        let mut result = Self::new();

        for (k, v) in iter {
            result.insert(k.to_owned(), v);
        }

        result
    }
}
