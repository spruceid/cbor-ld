use uuid::Uuid;

use super::IriCodec;
use crate::{CborValue, DecodeError, EncodeError};

pub struct UrnUuidCodec;

impl IriCodec for UrnUuidCodec {
    fn encode(&self, suffix: &str) -> Result<Vec<CborValue>, EncodeError> {
        let uuid = uuid::Uuid::parse_str(suffix)
            .map_err(|e| EncodeError::Codec("urn:uuid", e.to_string()))?;
        Ok(vec![CborValue::Bytes(uuid.into_bytes().to_vec())])
    }

    fn decode(&self, array: &[CborValue]) -> Result<String, DecodeError> {
        if array.len() != 1 {
            return Err(DecodeError::Codec(
                "urn:uuid",
                "invalid array length".to_string(),
            ));
        }

        let bytes = array[0]
            .as_bytes()
            .ok_or_else(|| DecodeError::Codec("urn:uuid", "expected bytes".to_string()))?;

        let uuid = Uuid::from_slice(bytes)
            .map_err(|_| DecodeError::Codec("urn:uuid", "invalid UUID".to_string()))?;

        Ok(uuid.to_string())
    }
}
