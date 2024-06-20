use std::str::FromStr;

use chrono::{DateTime, FixedOffset, TimeZone};
use iref::IriBuf;
use rdf_types::BlankIdBuf;

use super::TypeCodec;
use crate::{transform::TransformerState, CborValue, DecodeError, EncodeError};

pub struct XsdDateCodec;

impl TypeCodec for XsdDateCodec {
    fn encode(
        &self,
        _state: &TransformerState,
        _active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &str,
    ) -> Result<CborValue, EncodeError> {
        let date = xsd_types::Date::from_str(value)
            .map_err(|e| EncodeError::Codec("xsd-date", e.to_string()))?;

        if let Some(offset) = date.offset {
            if let Some(date_time) = offset.from_local_datetime(&date.date.into()).single() {
                let seconds = date_time.timestamp();
                return Ok(CborValue::Integer(seconds.into()));
            }
        }

        // No compression.
        Ok(CborValue::Text(value.to_owned()))
    }

    fn decode(
        &self,
        _state: &TransformerState,
        _active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &CborValue,
    ) -> Result<String, DecodeError> {
        match value {
            CborValue::Text(text) => Ok(text.clone()),
            CborValue::Integer(seconds) => {
                let seconds: i64 = (*seconds)
                    .try_into()
                    .map_err(|_| DecodeError::Codec("xsd-date", "overflow".to_string()))?;

                let date_time = DateTime::from_timestamp(seconds, 0)
                    .ok_or_else(|| DecodeError::Codec("xsd-date", "overflow".to_string()))?;

                Ok(xsd_types::Date::new(
                    date_time.date_naive(),
                    Some(FixedOffset::east_opt(0).unwrap()),
                )
                .to_string())
            }
            _ => todo!(),
        }
    }
}
