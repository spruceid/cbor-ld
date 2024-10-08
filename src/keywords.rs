use crate::IdMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref KEYWORDS_MAP: IdMap = [
        ("@context", 0),
        ("@type", 2),
        ("@id", 4),
        ("@value", 6),
        // alphabetized after `@context`, `@type`, `@id`, `@value`
        // IDs <= 24 represented with 1 byte, IDs > 24 use 2+ bytes
        ("@direction", 8),
        ("@graph", 10),
        ("@included", 12),
        ("@index", 14),
        ("@json", 16),
        ("@language", 18),
        ("@list", 20),
        ("@nest", 22),
        ("@reverse", 24),
        // these only appear in frames and contexts, not docs
        ("@base", 26),
        ("@container", 28),
        ("@default", 30),
        ("@embed", 32),
        ("@explicit", 34),
        ("@none", 36),
        ("@omitDefault", 38),
        ("@prefix", 40),
        ("@preserve", 42),
        ("@protected", 44),
        ("@requireAll", 46),
        ("@set", 48),
        ("@version", 50),
        ("@vocab", 52)
    ]
    .into_iter()
    .collect();
}

pub const FIRST_CUSTOM_TERM_ID: u64 = 100;
