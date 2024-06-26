use crate::{
    transform::{DuplicateKey, ExpectedObject, MissingKeyTerm, UndefinedTerm},
    CborValue,
};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    Cbor(#[from] ciborium::de::Error<std::io::Error>),

    #[error("not CBOR-LD")]
    NotCborLd,

    #[error("unsupported compression mode {0}")]
    UnsupportedCompressionMode(u8),

    #[error("expected node object")]
    ExpectedNodeObject,

    #[error("JSON-LD context processing failed: {0}")]
    ContextProcessing(#[from] json_ld::context_processing::Error),

    #[error("duplicate entry")]
    DuplicateEntry(CborValue),

    #[error("missing term")]
    MissingTermFor(CborValue),

    #[error("undefined term")]
    UndefinedTerm(String),

    #[error("non finite float")]
    NonFiniteFloat,

    #[error("invalid value")]
    InvalidValue,

    #[error("invalid id kind")]
    InvalidIdKind,

    #[error("invalid vocab value kind")]
    InvalidVocabTermKind,

    #[error("invalid JSON-LD context IRI reference: {0}")]
    InvalidContextIriRef(String),

    #[error("`{0}` codec error: {1}")]
    Codec(&'static str, String),
}

impl From<DuplicateKey<CborValue>> for DecodeError {
    fn from(value: DuplicateKey<CborValue>) -> Self {
        Self::DuplicateEntry(value.0)
    }
}

impl From<MissingKeyTerm<CborValue>> for DecodeError {
    fn from(value: MissingKeyTerm<CborValue>) -> Self {
        Self::MissingTermFor(value.0)
    }
}

impl From<UndefinedTerm> for DecodeError {
    fn from(value: UndefinedTerm) -> Self {
        Self::UndefinedTerm(value.0)
    }
}

impl From<ExpectedObject> for DecodeError {
    fn from(_value: ExpectedObject) -> Self {
        Self::ExpectedNodeObject
    }
}
