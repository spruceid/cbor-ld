use crate::IdMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTERED_CONTEXTS: IdMap = [
        ("https://www.w3.org/ns/activitystreams", 0x10),
        ("https://www.w3.org/2018/credentials/v1", 0x11),
        ("https://www.w3.org/ns/did/v1", 0x12),
        ("https://w3id.org/security/suites/ed25519-2018/v1", 0x13),
        ("https://w3id.org/security/suites/ed25519-2020/v1", 0x14),
        ("https://w3id.org/cit/v1", 0x15),
        ("https://w3id.org/age/v1", 0x16),
        ("https://w3id.org/security/suites/x25519-2020/v1", 0x17),
        ("https://w3id.org/veres-one/v1", 0x18),
        ("https://w3id.org/webkms/v1", 0x19),
        ("https://w3id.org/zcap/v1", 0x1A),
        ("https://w3id.org/security/suites/hmac-2019/v1", 0x1B),
        ("https://w3id.org/security/suites/aes-2019/v1", 0x1C),
        ("https://w3id.org/vaccination/v1", 0x1D),
        ("https://w3id.org/vc-revocation-list-2020/v1", 0x1E),
        ("https://w3id.org/dcc/v1", 0x1F),
        ("https://w3id.org/vc/status-list/v1", 0x20),
        ("https://www.w3.org/ns/credentials/v2", 0x21),
        ("https://w3id.org/security/data-integrity/v1", 0x30),
        ("https://w3id.org/security/multikey/v1", 0x31),
        ("https://purl.imsglobal.org/spec/ob/v3p0/context.json", 0x32),
        ("https://w3id.org/security/data-integrity/v2", 0x33)
    ]
    .into_iter()
    .collect();
}
