use super::IriCodec;
use crate::{CborValue, DecodeError, EncodeError};

pub struct HttpUrlCodec;

impl IriCodec for HttpUrlCodec {
    fn encode(&self, suffix: &str) -> Result<Vec<CborValue>, EncodeError> {
        // FIXME: presumes authority.
        let content = &suffix[2..];
        Ok(vec![CborValue::Text(content.to_owned())])
    }

    fn decode(&self, array: &[CborValue]) -> Result<String, DecodeError> {
        if array.len() != 1 {
            return Err(DecodeError::Codec(
                "url",
                "invalid array length".to_string(),
            ));
        }

        let text = array[0]
            .as_text()
            .ok_or_else(|| DecodeError::Codec("url", "expected text".to_string()))?;

        Ok(format!("//{text}"))
    }
}
