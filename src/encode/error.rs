use crate::transform::{
    DuplicateKey, ExpectedObject, InvalidTypeKind, MissingKeyTerm, UndefinedTerm,
};
use iref::IriRefBuf;

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("expected node object")]
    ExpectedNodeObject,

    #[error("invalid JSON-LD context entry")]
    InvalidContextEntry,

    #[error("invalid JSON-LD context: {0}")]
    InvalidContext(#[from] json_ld::syntax::context::InvalidContext),

    #[error("invalid JSON-LD context term definition")]
    InvalidTermDefinition,

    #[error("JSON-LD context processing failed: {0}")]
    ContextProcessing(#[from] json_ld::context_processing::Error),

    #[error("duplicate JSON entry `{0}`")]
    DuplicateEntry(String),

    #[error("undefined term `{0}`")]
    UndefinedTerm(String),

    #[error("node ID must be a string")]
    InvalidIdKind,

    #[error("invalid vocabulary term")]
    InvalidVocabTermKind,

    #[error("invalid vocabulary term `{0}`")]
    InvalidVocabTerm(String),

    #[error("invalid node ID `{0}`")]
    InvalidId(String),

    #[error("missing CBOR-LD context ID for `{0}`")]
    MissingContextId(IriRefBuf),

    #[error("missing CBOR-LD ID for `{0}`")]
    MissingIdFor(String),

    #[error("`{0}` codec error: {1}")]
    Codec(&'static str, String),
}

impl From<DuplicateKey<json_ld::syntax::object::Key>> for EncodeError {
    fn from(value: DuplicateKey<json_ld::syntax::object::Key>) -> Self {
        Self::DuplicateEntry(value.0.into_string())
    }
}

impl From<MissingKeyTerm<json_ld::syntax::object::Key>> for EncodeError {
    fn from(value: MissingKeyTerm<json_ld::syntax::object::Key>) -> Self {
        Self::MissingIdFor(value.0.into_string())
    }
}

impl From<UndefinedTerm> for EncodeError {
    fn from(value: UndefinedTerm) -> Self {
        Self::UndefinedTerm(value.0)
    }
}

impl From<ExpectedObject> for EncodeError {
    fn from(_value: ExpectedObject) -> Self {
        Self::ExpectedNodeObject
    }
}

impl From<InvalidTypeKind> for EncodeError {
    fn from(_value: InvalidTypeKind) -> Self {
        Self::InvalidVocabTermKind
    }
}
