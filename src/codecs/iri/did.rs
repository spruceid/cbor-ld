use multibase::Base;

use super::IriCodec;
use crate::{CborValue, DecodeError, EncodeError};

pub struct Base58DidMethodCodec;

impl IriCodec for Base58DidMethodCodec {
    fn encode(&self, suffix: &str) -> Result<Vec<CborValue>, EncodeError> {
        match suffix.split_once('#') {
            Some((id, fragment)) => {
                let (_, bytes) = multibase::decode(id)
                    .map_err(|e| EncodeError::Codec("base58-did-method", e.to_string()))?;

                let (_, fragment_bytes) = multibase::decode(fragment).map_err(|e| {
                    EncodeError::Codec("base58-did-method(fragment)", e.to_string())
                })?;

                Ok(vec![
                    CborValue::Bytes(bytes),
                    CborValue::Bytes(fragment_bytes),
                ])
            }
            None => {
                let (_, bytes) = multibase::decode(suffix)
                    .map_err(|e| EncodeError::Codec("base58-did-method", e.to_string()))?;
                Ok(vec![CborValue::Bytes(bytes)])
            }
        }
    }

    fn decode(&self, array: &[CborValue]) -> Result<String, DecodeError> {
        match array.len() {
            1 => {
                let bytes = array[0].as_bytes().ok_or_else(|| {
                    DecodeError::Codec("base58-did-method", "expected bytes".to_string())
                })?;

                Ok(multibase::encode(Base::Base58Btc, bytes))
            }
            2 => {
                let bytes = array[0].as_bytes().ok_or_else(|| {
                    DecodeError::Codec("base58-did-method", "expected bytes".to_string())
                })?;

                let fragment_bytes = array[1].as_bytes().ok_or_else(|| {
                    DecodeError::Codec("base58-did-method", "expected bytes".to_string())
                })?;

                let id = multibase::encode(Base::Base58Btc, bytes);
                let fragment = multibase::encode(Base::Base58Btc, fragment_bytes);

                Ok(format!("{id}#{fragment}"))
            }
            _ => Err(DecodeError::Codec(
                "base58-did-method",
                "invalid array length".to_string(),
            )),
        }
    }
}
