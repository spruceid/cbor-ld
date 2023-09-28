use crate::get_contextmap;
use crate::TruageCborLdError;
use chrono::DateTime;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, vec};
use uuid::Uuid;
pub const DID_KEY: &str = "did:key:";
pub const DID_V1: &str = "did:v1:nym";

fn encode_context(context: String) -> Result<Vec<u8>, TruageCborLdError> {
    let context_map = get_contextmap();
    let value = context_map.get(&context);
    match value {
        Some(v) => Ok(vec![v.to_owned()]),
        None => Err(TruageCborLdError::UnexpectedFormat(format!(
            "unknown key word: {}",
            context
        ))),
    }
}

fn encode_urnuuid(urn_uuid: String) -> Result<Vec<u8>, TruageCborLdError> {
    let Some(bare_uuid) = urn_uuid.strip_prefix("urn:uuid:") else {
        return Err(TruageCborLdError::UnexpectedFormat(format!("invalid urn:uuid formatting: {:?}", urn_uuid)))
    };
    let uuid = Uuid::parse_str(bare_uuid)?;

    let mut uuid_bytes: Vec<u8> = uuid.as_bytes().to_vec();
    //cbor tag 130 indicating an array with 2 items
    //cbor tag 3 indicating a negative bignum (?)
    //cbor tag 80: IEEE 754 binary16 big endian, Typed Array
    let mut cborld_encoded: Vec<u8> = vec![130, 3, 80];
    cborld_encoded.append(&mut uuid_bytes);

    Ok(cborld_encoded)
}

pub fn encode_vocab_term(
    terms: String,
    term_to_id_map: HashMap<String, u8>,
) -> Result<Vec<u8>, TruageCborLdError> {
    //let bare_term = terms.replace("\"", "");
    let Some(term_id) = term_to_id_map.get(&terms) else {
        return Err(TruageCborLdError::UnexpectedFormat("unknown keyword".to_string()))
    };
    Ok(vec![term_id.to_owned()])
}

fn encode_xsd_datetime(value: String) -> Result<Vec<u8>, TruageCborLdError> {
    let iso_date = DateTime::parse_from_rfc3339(value.as_str())?;
    let system_time: SystemTime = iso_date.into();
    let secs_since_epoch = system_time.duration_since(UNIX_EPOCH)?.as_secs();
    let xsd_result = serde_cbor::ser::to_vec(&secs_since_epoch)?;
    Ok(xsd_result)
}

fn encode_multi_base(value: String) -> Result<Vec<u8>, TruageCborLdError> {
    let value_string = value.as_str();
    let mut multi_base_bytes: Vec<u8> = vec![];
    if let Some(strip) = value_string.strip_prefix('z') {
        let mut decoded = bs58::decode(strip)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()?;
        //tag 88 as a (unregistered?) cbor tag for a multibase encoding followed by the length of the encoding
        multi_base_bytes.append(&mut vec![88, decoded.len() as u8 + 1]);
        //multibase tag for a base58 encoding
        multi_base_bytes.push(0x7a);
        multi_base_bytes.append(&mut decoded);
    } else if let Some(stripped) = value_string.strip_prefix('M') {
        multi_base_bytes.push(0x4d);
        let mut decoded = base64::decode(stripped)?;
        multi_base_bytes.append(&mut decoded);
    } else {
        return Err(TruageCborLdError::UnexpectedFormat(format!(
            "error encoding a multibase value: {}",
            value
        )));
    }
    Ok(multi_base_bytes)
}

fn encode_base_58_did_url(value: String) -> Result<Vec<u8>, TruageCborLdError> {
    let (prefix, suffix) = if value.starts_with(DID_V1) {
        value.split_at(DID_V1.len())
    } else {
        value.split_at(DID_KEY.len())
    };

    let to_decode: Vec<&str> = suffix.split('#').collect();

    // individually encoded substrings are encoded as an array of substrings
    let mut did_url: Vec<u8> = vec![to_decode.len() as u8 + 129];

    //tag for did scheme
    if prefix.starts_with(DID_V1) {
        did_url.append(&mut vec![25, 4, 0]);
    } else if prefix.starts_with(DID_KEY) {
        did_url.append(&mut vec![25, 4, 1]);
    }

    for s in to_decode {
        let dec = &s[1..];
        let mut did = bs58::decode(dec).into_vec()?;
        // cbor tag 88 to indicate a multibase encoding + length of encoding
        did_url.append(&mut vec![88, did.len() as u8]);
        did_url.append(&mut did);
    }
    Ok(did_url)
}

pub fn encode_string_value(val: String) -> Result<Vec<u8>, TruageCborLdError> {
    if val.starts_with("https:/") {
        encode_context(val)
    } else if val.starts_with("urn:uuid:") {
        encode_urnuuid(val)
    } else if val.starts_with("did:key") || val.starts_with("did:v1") {
        encode_base_58_did_url(val)
    } else if val.starts_with('z') || val.starts_with('M') {
        encode_multi_base(val)
    } else {
        let date = encode_xsd_datetime(val.clone());
        match date {
            Ok(d) => Ok(d),
            Err(_e) => Err(TruageCborLdError::UnexpectedFormat(format!(
                "unable to process string value according to cbor-ld rules: {}",
                val
            ))),
        }
    }
}
