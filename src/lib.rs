use std::collections::HashMap;
use truage::TruageCborLdError;
pub mod encoders;
pub mod truage;

//Note: These context and vocab maps are focused on TruAge and subject to change.
//Cbor-ld is not an established standard, and Digital Bazaar can make unilateral changes to encoding vocabulary.
//Before using Cbor-ld for anything, make sure that you can't use base64url encoding instead.

pub fn get_contextmap() -> HashMap<String, u8> {
    HashMap::<String, u8>::from([
        (String::from("https://www.w3.org/ns/activitystreams"), 0x10),
        (String::from("https://www.w3.org/2018/credentials/v1"), 0x11),
        (String::from("https://www.w3.org/ns/did/v1"), 0x12),
        (
            String::from("https://w3id.org/security/suites/ed25519-2018/v1"),
            0x13,
        ),
        (
            String::from("https://w3id.org/security/suites/ed25519-2020/v1"),
            0x14,
        ),
        (String::from("https://w3id.org/cit/v1"), 0x15),
        (String::from("https://w3id.org/age/v1"), 0x16),
        (
            String::from("https://w3id.org/security/suites/x25519-2020/v1"),
            0x17,
        ),
        (String::from("https://w3id.org/veres-one/v1"), 0x18),
        (String::from("https://w3id.org/webkms/v1"), 0x19),
        (String::from("https://w3id.org/zcap/v1"), 0x1A),
        (
            String::from("https://w3id.org/security/suites/hmac-2019/v1"),
            0x1B,
        ),
        (
            String::from("https://w3id.org/security/suites/aes-2019/v1"),
            0x1C,
        ),
        (String::from("https://w3id.org/vaccination/v1"), 0x1D),
        (
            String::from("https://w3id.org/vc-revocation-list-2020/v1"),
            0x1E,
        ),
        (String::from("https://w3id.org/dcc/v1c"), 0x1F),
        (String::from("https://w3id.org/vc/status-list/v1"), 0x20),
    ])
}

pub fn get_keywordsmap() -> HashMap<String, u8> {
    HashMap::<String, u8>::from([
        (String::from("@context"), 0),
        (String::from("@type"), 2),
        (String::from("@id"), 4),
        (String::from("@value"), 6),
        (String::from("@direction"), 8),
        (String::from("@graph"), 10),
        (String::from("@graph"), 12),
        (String::from("@index"), 14),
        (String::from("@json"), 16),
        (String::from("@language"), 18),
        (String::from("@list"), 20),
        (String::from("@nest"), 22),
        (String::from("@reverse"), 24),
        //digitalbazaar might remove the following
        (String::from("@base"), 26),
        (String::from("@container"), 28),
        (String::from("@default"), 30),
        (String::from("@embed"), 32),
        (String::from("@explicit"), 34),
        (String::from("@none"), 36),
        (String::from("@omitDefault"), 38),
        (String::from("@prefix"), 40),
        (String::from("@preserve"), 42),
        (String::from("@protected"), 44),
        (String::from("@requireAll"), 46),
        (String::from("@set"), 48),
        (String::from("@version"), 50),
        (String::from("@vocab"), 52),
        //Hardcoded for Truage implementation
        (String::from("EcdsaSecp256k1Signature2019"), 100),
        (String::from("EcdsaSecp256r1Signature2019"), 102),
        (String::from("Ed25519Signature2018"), 104),
        (String::from("RsaSignature2018"), 106),
        (String::from("VerifiableCredential"), 108),
        (String::from("VerifiablePresentation"), 110),
        (String::from("id"), 112),
        (String::from("proof"), 114),
        (String::from("type"), 116),
        (String::from("cred"), 118),
        (String::from("holder"), 120),
        (String::from("sec"), 122),
        (String::from("verifiableCredential"), 124),
        (String::from("AgeVerificationContainerCredential"), 126),
        (String::from("AgeVerificationCredential"), 128),
        (String::from("OverAgeTokenCredential"), 130),
        (String::from("PersonalPhotoCredential"), 132),
        (String::from("VerifiableCredentialRefreshService2021"), 134),
        (String::from("anchoredRes&ource"), 136),
        (String::from("concealedIdToken"), 138),
        (String::from("description"), 140),
        (String::from("digestMultibase"), 142),
        (String::from("image"), 144),
        (String::from("name"), 146),
        (String::from("overAge"), 148),
        (String::from("Ed25519Signature2020"), 150),
        (String::from("Ed25519VerificationKey2020"), 152),
        (String::from("credentialSchema"), 154),
        (String::from("credentialStatus"), 156),
        (String::from("credentialSubject"), 158),
        (String::from("evidence"), 160),
        (String::from("expirationDate"), 162),
        (String::from("issuanceDate"), 164),
        (String::from("issued"), 166),
        (String::from("issuer"), 168),
        (String::from("refreshService"), 170),
        (String::from("termsOfUse"), 172),
        (String::from("validForm"), 174),
        (String::from("validUntil"), 176),
        (String::from("xsd"), 178),
        (String::from("challenge"), 180),
        (String::from("created"), 182),
        (String::from("domain"), 184),
        (String::from("expires"), 186),
        (String::from("nonce"), 188),
        (String::from("proofPurpose"), 190),
        (String::from("proofValue"), 192),
        (String::from("verificationMethod"), 194),
        (String::from("assertionMethod"), 196),
        (String::from("authentication"), 198),
        (String::from("capabilityDelegation"), 200),
        (String::from("capabilityInvocation"), 202),
        (String::from("keyAgreement"), 204),
    ])
}
