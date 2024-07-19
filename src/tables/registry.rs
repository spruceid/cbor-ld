use std::borrow::Cow;

use json_ld::Type;
use lazy_static::lazy_static;
use static_iref::iri;

use crate::Tables;

/// Compression tables registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegistryEntry {
    /// Default compression tables.
    Default,

    /// Verifiable Credential Barcodes Specification Test Vectors.
    ///
    /// See: <https://w3c-ccg.github.io/vc-barcodes/>
    VcBarcodes,

    /// Unknown compression table.
    Unknown(u64),
}

impl RegistryEntry {
    pub fn from_id(id: u64) -> Self {
        match id {
            1 => Self::Default,
            100 => Self::VcBarcodes,
            n => Self::Unknown(n),
        }
    }

    pub fn id(&self) -> u64 {
        match self {
            Self::Default => 1,
            Self::VcBarcodes => 100,
            Self::Unknown(id) => *id,
        }
    }

    pub fn tables<'a>(
        &self,
        default: Cow<'a, Tables>,
    ) -> Result<Cow<'a, Tables>, UnknownCompressionTable> {
        match self {
            Self::Default => Ok(default),
            Self::VcBarcodes => Ok(Cow::Borrowed(&VC_BARCODES)),
            Self::Unknown(id) => Err(UnknownCompressionTable(*id)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown compression table `{0}`")]
pub struct UnknownCompressionTable(pub u64);

lazy_static! {
    static ref VC_BARCODES: Tables = {
        Tables {
            context: [
                (iri!("https://www.w3.org/ns/credentials/v2"), 32768),
                (iri!("https://w3id.org/vc-barcodes/v1"), 32769),
                (iri!("https://w3id.org/utopia/v2"), 32770),
            ]
            .into_iter()
            .collect(),
            types: [(
                Type::Iri(iri!("https://w3id.org/security#cryptosuiteString").to_owned()),
                [
                    ("ecdsa-rdfc-2019", 1),
                    ("ecdsa-sd-2023", 2),
                    ("eddsa-rdfc-2022", 3),
                    ("ecdsa-xi-2023", 4),
                ]
                .into_iter()
                .collect(),
            )]
            .into_iter()
            .collect(),
        }
    };
}
