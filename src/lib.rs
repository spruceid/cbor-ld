pub use ciborium::Value as CborValue;
pub use json_ld::syntax::Value as JsonValue;

pub type CborObject = Vec<(CborValue, CborValue)>;
pub type JsonObject = json_ld::syntax::Object;

pub mod codecs;
pub mod contexts;
mod decode;
mod encode;
mod id_alloc;
pub mod id_map;
pub mod keywords;
pub mod utils;
pub use decode::*;
pub use encode::*;
pub mod diagnostic;
pub mod transform;

pub use codecs::Codecs;
pub use id_alloc::IdAllocator;
pub use id_map::IdMap;

#[derive(Debug, Default)]
pub enum CompressionMode {
    Uncompressed,

    #[default]
    Version1,
}

impl CompressionMode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Uncompressed),
            1 => Some(Self::Version1),
            _ => None,
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Uncompressed => 0,
            Self::Version1 => 1,
        }
    }

    pub fn to_tag(&self) -> u64 {
        0x0500 | self.to_byte() as u64
    }

    pub fn from_tag(tag: u64) -> Result<Self, DecodeError> {
        if tag >> 8 != 0x05 {
            return Err(DecodeError::NotCborLd);
        }

        let mode = (tag & 0xff) as u8;
        Self::from_byte(mode).ok_or(DecodeError::UnsupportedCompressionMode(mode))
    }
}
