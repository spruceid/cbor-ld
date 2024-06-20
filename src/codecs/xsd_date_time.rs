use std::str::FromStr;

use chrono::{TimeZone, Utc};
use iref::IriBuf;
use rdf_types::BlankIdBuf;

use super::TypeCodec;
use crate::{transform::TransformerState, CborValue, DecodeError, EncodeError};

pub struct XsdDateTimeCodec;

impl TypeCodec for XsdDateTimeCodec {
    fn encode(
        &self,
        _state: &TransformerState,
        _active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &str,
    ) -> Result<CborValue, EncodeError> {
        let date_time = xsd_types::DateTime::from_str(value)
            .map_err(|e| EncodeError::Codec("xsd-date-time", e.to_string()))?;

        let exact_date_time = date_time.earliest();

        if exact_date_time == date_time.latest() {
            let timestamp = exact_date_time.timestamp_micros();
            let seconds = timestamp / 1_000_000;
            let milliseconds = (timestamp / 1000) % 1000;

            if seconds * 1_000_000 == timestamp {
                // second precision.
                return Ok(CborValue::Integer(seconds.into()));
            }

            if seconds * 1_000_000 + milliseconds * 1000 == timestamp {
                // millisecond precision.
                return Ok(CborValue::Array(vec![
                    CborValue::Integer(seconds.into()),
                    CborValue::Integer(milliseconds.into()),
                ]));
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
    ) -> Result<String, crate::DecodeError> {
        match value {
            CborValue::Text(text) => Ok(text.clone()),
            CborValue::Integer(i) => {
                let seconds: i128 = (*i).into();
                match i64::try_from(seconds) {
                    Ok(seconds) => match Utc.timestamp_opt(seconds, 0).single() {
                        Some(date_time) => Ok(xsd_types::DateTime::from(date_time).into_string()),
                        None => {
                            todo!()
                        }
                    },
                    Err(_) => {
                        todo!()
                    }
                }
            }
            CborValue::Array(items) => {
                if items.len() != 2 {
                    todo!()
                }

                let seconds: i64 = items[0]
                    .as_integer()
                    .ok_or_else(|| {
                        DecodeError::Codec("xsd-date-time", "expected integer".to_string())
                    })?
                    .try_into()
                    .map_err(|_| DecodeError::Codec("xsd-date-time", "overflow".to_string()))?;

                let milliseconds: u32 = items[0]
                    .as_integer()
                    .ok_or_else(|| {
                        DecodeError::Codec("xsd-date-time", "expected integer".to_string())
                    })?
                    .try_into()
                    .map_err(|_| DecodeError::Codec("xsd-date-time", "overflow".to_string()))?;

                match Utc
                    .timestamp_opt(seconds, milliseconds * 1_000_000)
                    .single()
                {
                    Some(date_time) => Ok(xsd_types::DateTime::from(date_time).into_string()),
                    None => {
                        todo!()
                    }
                }
            }
            _ => todo!(),
        }
    }
}
