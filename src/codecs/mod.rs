use core::fmt;
use iref::{Iri, IriBuf};
use static_iref::iri;
use std::collections::HashMap;

use crate::{transform::TransformerState, CborValue, DecodeError, EncodeError};

mod iri;
pub use iri::*;

mod id;
pub use id::*;

mod vocab;
pub use vocab::*;

mod multibase;
pub use multibase::*;

mod xsd_date;
pub use xsd_date::*;

mod xsd_date_time;
pub use xsd_date_time::*;

pub trait TypeCodec: Send + Sync {
    fn encode(
        &self,
        state: &TransformerState,
        active_context: &json_ld::Context,
        value: &str,
    ) -> Result<CborValue, EncodeError>;

    fn decode(
        &self,
        state: &TransformerState,
        active_context: &json_ld::Context,
        value: &CborValue,
    ) -> Result<String, DecodeError>;
}

pub struct TypeCodecs {
    map: HashMap<json_ld::Type<IriBuf>, Box<dyn TypeCodec>>,
}

impl TypeCodecs {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, type_: &json_ld::Type<IriBuf>) -> Option<&dyn TypeCodec> {
        self.map.get(type_).map(|b| &**b)
    }

    pub fn insert(&mut self, type_: json_ld::Type<IriBuf>, encoder: impl 'static + TypeCodec) {
        self.map.insert(type_, Box::new(encoder));
    }
}

impl fmt::Debug for TypeCodecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeCodecs").finish()
    }
}

pub const MULTIBASE: &Iri = iri!("https://w3id.org/security#multibase");

impl Default for TypeCodecs {
    fn default() -> Self {
        let mut result = Self::new();

        result.insert(json_ld::Type::Id, IdCodec);
        result.insert(json_ld::Type::Vocab, VocabCodec);
        result.insert(json_ld::Type::Iri(MULTIBASE.to_owned()), MultibaseCodec);
        result.insert(
            json_ld::Type::Iri(xsd_types::XSD_DATE.to_owned()),
            XsdDateCodec,
        );
        result.insert(
            json_ld::Type::Iri(xsd_types::XSD_DATE_TIME.to_owned()),
            XsdDateTimeCodec,
        );

        result
    }
}

#[derive(Debug, Default)]
pub struct Codecs {
    pub iri: IriCodecs,
    pub type_: TypeCodecs,
}
