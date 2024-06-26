use super::TypeCodec;
use crate::{transform::TransformerState, CborValue, DecodeError, EncodeError};
use iref::{Iri, IriBuf};

pub struct IdCodec;

impl TypeCodec for IdCodec {
    fn encode(
        &self,
        state: &TransformerState,
        _active_context: &json_ld::Context,
        value: &str,
    ) -> Result<CborValue, EncodeError> {
        let iri = Iri::new(value).map_err(|e| EncodeError::InvalidId(e.0.to_owned()))?;
        state.codecs.iri.encode(iri)
    }

    fn decode(
        &self,
        state: &TransformerState,
        _active_context: &json_ld::Context,
        value: &CborValue,
    ) -> Result<String, DecodeError> {
        state.codecs.iri.decode(value).map(IriBuf::into_string)
    }
}
