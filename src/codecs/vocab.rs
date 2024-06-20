use iref::IriBuf;
use rdf_types::BlankIdBuf;

use super::TypeCodec;
use crate::{transform::TransformerState, CborValue, DecodeError, EncodeError};

pub struct VocabCodec;

impl TypeCodec for VocabCodec {
    fn encode(
        &self,
        state: &TransformerState,
        active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &str,
    ) -> Result<CborValue, EncodeError> {
        state.encode_vocab_term(active_context, value)
    }

    fn decode(
        &self,
        state: &TransformerState,
        active_context: &json_ld::Context<IriBuf, BlankIdBuf>,
        value: &CborValue,
    ) -> Result<String, DecodeError> {
        state.decode_vocab_term(active_context, value)
    }
}
