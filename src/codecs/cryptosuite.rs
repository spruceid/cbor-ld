use iref::IriBuf;
use lazy_static::lazy_static;
use rdf_types::BlankIdBuf;

use super::TypeCodec;
use crate::{transform::TransformerState, CborValue, DecodeError, EncodeError, IdMap};

lazy_static! {
    pub static ref REGISTERED_CRYPTOSUITES: IdMap = [
        ("ecdsa-rdfc-2019", 0x34),
        ("ecdsa-sd-2023", 0x35),
        ("eddsa-rdfc-2022", 0x36),
    ]
    .into_iter()
    .collect();
}

pub struct CryptosuiteCodec {
    cryptosuites: IdMap,
}

impl CryptosuiteCodec {
    pub fn new(cryptosuites: IdMap) -> Self {
        Self { cryptosuites }
    }
}

impl Default for CryptosuiteCodec {
    fn default() -> Self {
        Self::new(IdMap::new_derived(Some(&REGISTERED_CRYPTOSUITES)))
    }
}

impl TypeCodec for CryptosuiteCodec {
    fn encode(
        &self,
        _state: &TransformerState,
        _active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &str,
    ) -> Result<CborValue, EncodeError> {
        match self.cryptosuites.get_id(value) {
            Some(id) => Ok(CborValue::Integer(id.into())),
            None => Ok(CborValue::Text(value.to_owned())),
        }
    }

    fn decode(
        &self,
        state: &TransformerState,
        active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &CborValue,
    ) -> Result<String, DecodeError> {
        match value {
            CborValue::Integer(id) => {
                let id: u64 = (*id).try_into().map_err(|_| {
                    DecodeError::Codec("cryptosuite", "unknown cryptosuite ID".to_owned())
                })?;

                let name = self.cryptosuites.get_term(id).ok_or_else(|| {
                    DecodeError::Codec("cryptosuite", "unknown cryptosuite ID".to_owned())
                })?;

                Ok(name.to_owned())
            }
            CborValue::Text(name) => Ok(name.to_owned()),
            _ => Err(DecodeError::Codec(
                "cryptosuite",
                "expected integer or text".to_owned(),
            )),
        }
    }
}
