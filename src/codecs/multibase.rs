use super::TypeCodec;
use crate::{transform::TransformerState, CborValue, DecodeError, EncodeError};
use multibase::Base;

pub struct MultibaseCodec;

impl TypeCodec for MultibaseCodec {
    fn encode(
        &self,
        _state: &TransformerState,
        _active_context: &json_ld::Context,
        value: &str,
    ) -> Result<CborValue, EncodeError> {
        let (base, bytes) =
            multibase::decode(value).map_err(|e| EncodeError::Codec("multibase", e.to_string()))?;

        let mut data = Vec::with_capacity(1 + bytes.len());
        data.push(base.code() as u8);
        data.extend(bytes);
        Ok(CborValue::Bytes(data))
    }

    fn decode(
        &self,
        _state: &TransformerState,
        _active_context: &json_ld::Context,
        value: &CborValue,
    ) -> Result<String, DecodeError> {
        let bytes = value
            .as_bytes()
            .ok_or_else(|| DecodeError::Codec("multibase", "expected bytes".to_string()))?;

        if bytes.is_empty() {
            return Err(DecodeError::Codec("multibase", "empty bytes".to_owned()));
        }

        let base = Base::from_code(bytes[0].into())
            .map_err(|_| DecodeError::Codec("multibase", "unknown base".to_owned()))?;

        Ok(multibase::encode(base, &bytes[1..]))
    }
}
