use crate::encoders::{encode_string_value, encode_vocab_term};
use crate::get_keywordsmap;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const DID_KEY: &str = "did:key:";
pub const DID_V1: &str = "did:v1:nym";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TruageCborLdError {
    UnexpectedFormat(String),
}

impl From<TruageCborLdError> for Error {
    fn from(value: TruageCborLdError) -> Self {
        Error::msg(format!("{:?}", value))
    }
}

impl From<serde_json::Error> for TruageCborLdError {
    fn from(value: serde_json::Error) -> Self {
        TruageCborLdError::UnexpectedFormat(value.to_string())
    }
}

impl From<serde_cbor::Error> for TruageCborLdError {
    fn from(value: serde_cbor::Error) -> Self {
        TruageCborLdError::UnexpectedFormat(value.to_string())
    }
}

impl From<bs58::decode::Error> for TruageCborLdError {
    fn from(value: bs58::decode::Error) -> Self {
        TruageCborLdError::UnexpectedFormat(value.to_string())
    }
}

impl From<base64::DecodeError> for TruageCborLdError {
    fn from(value: base64::DecodeError) -> Self {
        TruageCborLdError::UnexpectedFormat(value.to_string())
    }
}

impl From<uuid::Error> for TruageCborLdError {
    fn from(value: uuid::Error) -> Self {
        TruageCborLdError::UnexpectedFormat(value.to_string())
    }
}

impl From<std::time::SystemTimeError> for TruageCborLdError {
    fn from(value: std::time::SystemTimeError) -> Self {
        TruageCborLdError::UnexpectedFormat(value.to_string())
    }
}

impl From<chrono::ParseError> for TruageCborLdError {
    fn from(value: chrono::ParseError) -> Self {
        TruageCborLdError::UnexpectedFormat(value.to_string())
    }
}

// this function recursively collects cbor-ld encoded objects into an array of transform_maps
// to obtain the compressed cbor-ld, the encoder must know something about the document structure to reconstruct the propper mapping tags
// this algorithm is incomplete, but covers the scope of TruAge credentials.
// To make a generic jsonld-cborld converter:
// TODO: extend truage_jsonld_to_cborld to cover all types
// TODO: attach coordinates to each transform_map to always allow for reconstruction of the cbor-ld document
pub fn truage_jsonld_to_cborld(
    document: Value,
    mut transform_maps: Vec<BTreeMap<u8, Vec<u8>>>,
) -> Result<Vec<BTreeMap<u8, Vec<u8>>>, TruageCborLdError> {
    let doc = match document {
        Value::Object(o) => o,
        _ => {
            return Err(TruageCborLdError::UnexpectedFormat(
                "Invalid document structure".to_string(),
            ))
        }
    };

    let key_map = get_keywordsmap();
    let mut transform_map = BTreeMap::<u8, Vec<u8>>::new();
    let mut results = vec![];

    for (key, value) in doc {
        let Some(key_encoded) = key_map.get(&key) else {
            return Err(TruageCborLdError::UnexpectedFormat(format!("unknown key: {}", key)))
        };

        match value {
            Value::Array(array) => {
                let mut value_array: Vec<u8> = vec![];
                // if a known cbor-ld key has a value of type array, the plural key encoding is used,
                // which is always the next number from the key encoding
                let key_encoded_plural = key_encoded + 1;
                let map_indicator = array.len() as u8 + 128;
                value_array.append(&mut vec![map_indicator]);

                let encoding_result: Result<(), TruageCborLdError> =
                    array.into_iter().try_for_each(|v| {
                        match v {
                            Value::String(s) => {
                                if let Some(_known_key_word) = key_map.get(&s) {
                                    let mut value_encoded =
                                        encode_vocab_term(s.clone(), key_map.clone())?;
                                    //cbor tag 24 for an independent integer
                                    value_encoded.insert(0, 24);
                                    value_array.append(&mut value_encoded);
                                } else {
                                    let mut value_encoded = encode_string_value(s.to_string())?;
                                    value_array.append(&mut value_encoded);
                                }
                            }
                            //TruAge credentials only have array values containing strings
                            _ => {
                                return Err(TruageCborLdError::UnexpectedFormat(format!(
                                    "unexpected value type for TruAge: {}",
                                    v
                                )))
                            }
                        }
                        transform_map.insert(key_encoded_plural, value_array.clone());
                        Ok(())
                    });

                // if anything went wrong encoding the array values, stop encoding entirely
                match encoding_result {
                    Ok(_result) => {}
                    Err(e) => return Err(e),
                }
            }
            Value::Object(o) => {
                let mut embedded_transform_map =
                    truage_jsonld_to_cborld(Value::Object(o), transform_maps.clone())?;
                transform_maps.append(&mut embedded_transform_map);
            }
            Value::String(s) => {
                if let Some(_known_key_word) = key_map.get(&s) {
                    let value_encoded = encode_vocab_term(s.clone(), key_map.clone())?;
                    transform_map.insert(*key_encoded, value_encoded);
                } else {
                    let value_encoded = encode_string_value(s)?;
                    transform_map.insert(*key_encoded, value_encoded);
                }
            }
            Value::Number(number) => {
                let Some(num) = number.as_u64() else {
                    return Err(TruageCborLdError::UnexpectedFormat("Integer value can't be parsed as a number".to_string()))
                };
                transform_map.insert(*key_encoded, vec![num as u8]);
            }
            _ => {
                return Err(TruageCborLdError::UnexpectedFormat(
                    "Unexpected value type for TruAge".to_string(),
                ))
            }
        }
    }
    transform_maps.push(transform_map.clone());
    results.push(transform_map);

    Ok(transform_maps)
}

pub fn compress(map: BTreeMap<u8, Vec<u8>>) -> Result<Vec<u8>, TruageCborLdError> {
    let mut compressed_array: Vec<u8> = vec![];
    let map_indicator2: u8 = map.len() as u8 + 160;
    compressed_array.push(map_indicator2);
    for (key, value) in map {
        let mut compressed_element = serde_cbor::to_vec(&key)?;
        let mut val = value.clone();
        if value.len() == 1 {
            let Some(v) = value.clone().pop() else { return Err(TruageCborLdError::UnexpectedFormat("unexpected empty value".to_string()))};
            compressed_element.append(&mut serde_cbor::to_vec(&v)?);
        } else {
            compressed_element.append(&mut val);
        }
        compressed_array.append(&mut compressed_element);
    }
    Ok(compressed_array)
}

pub fn encode_truage(document: Value) -> Result<Vec<u8>, TruageCborLdError> {
    let key_words_map = get_keywordsmap();
    let transform_maps = vec![];
    let mut result = truage_jsonld_to_cborld(document, transform_maps)?;

    let Some(mut truage_credential) = result.pop() else { return Err(TruageCborLdError::UnexpectedFormat("failure encoding truage credential".to_string()))};
    let Some(mut verifiable_credential) = result.pop() else { return Err(TruageCborLdError::UnexpectedFormat("failure encoding verifiable_credential".to_string()))};
    let Some(proof) =result.pop() else { return Err(TruageCborLdError::UnexpectedFormat("failure encoding proof".to_string()))};
    let Some(credential_subject) = result.pop() else { return Err(TruageCborLdError::UnexpectedFormat("failure encoding credential_subject".to_string()))};

    //reconstructing the truage credential
    let compressed_credential_subject_array: Vec<u8> = compress(credential_subject)?;
    let compressed_proof_array: Vec<u8> = compress(proof)?;

    let Some(proof_indicator) = key_words_map.get("proof") else {
        return Err(TruageCborLdError::UnexpectedFormat("current mapping does not know how to encode proof parameter".to_string()))
    };
    let Some(credential_subject_indicator) = key_words_map.get("credentialSubject") else {
        return Err(TruageCborLdError::UnexpectedFormat("current mapping does not know how to encode credentialSubject parameter".to_string()))
    };

    verifiable_credential.insert(proof_indicator.to_owned(), compressed_proof_array);
    verifiable_credential.insert(
        credential_subject_indicator.to_owned(),
        compressed_credential_subject_array,
    );

    let compressed_vc_array = compress(verifiable_credential)?;
    let Some(vc_indicator) = key_words_map.get("verifiableCredential") else {
        return Err(TruageCborLdError::UnexpectedFormat("current mapping does not know how to encode verifiableCredential parameter".to_string()))
    };
    truage_credential.insert(vc_indicator.to_owned(), compressed_vc_array);

    //tag indicating cbor encoding
    let mut final_result: Vec<u8> = vec![217, 5, 1];

    let mut truage_credential_cborld_compressed = compress(truage_credential)?;
    final_result.append(&mut truage_credential_cborld_compressed);

    Ok(final_result)
}

// tests will fail without the correct json_ld document input
#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_truage_cborld_compression() {
        let cmp_hex = hex::decode("d90501a300111874186e187ca801831116141870820350188e8450269e11ebb545d3692cf353981872a51874189618b61a610efcda18be18c418c058417abc243faceeb32327cf8afe87f7ef7d743983c588ef3d06c59f14914f0ea096d836f6fe8202c07f79c8aff0f664d276d37f170eeb742e425334fd0824af26e60c18c2831904015822ed01597d5ac5de5cdb08efcc29850df0d3fc935190b86eabbeb5eb06884db6a3aeec5822ed01597d5ac5de5cdb08efcc29850df0d3fc935190b86eabbeb5eb06884db6a3aeec187582186c1882189ea2188a58537ad90501a401150904074a7ad90501a2011605184108583b7a0000abd6420d628c532176ef0dd720df748248b808d4b9425c8f45ab44b5029feaca278d4fcd2d48cdf617fdac0f99681757fc8de74afda52e2418941518a21a60d4e4f718a41a605b9af718a8821904015822ed01597d5ac5de5cdb08efcc29850df0d3fc935190b86eabbeb5eb06884db6a3aeec").unwrap();

        let doc: Value = serde_json::json!({
          "@context": "https://www.w3.org/2018/credentials/v1",
          "type": "VerifiablePresentation",
          "verifiableCredential": {
            "@context": [
              "https://www.w3.org/2018/credentials/v1",
              "https://w3id.org/age/v1",
              "https://w3id.org/security/suites/ed25519-2020/v1"
            ],
            "id": "urn:uuid:188e8450-269e-11eb-b545-d3692cf35398",
            "type": [
              "VerifiableCredential",
              "OverAgeTokenCredential"
            ],
            "issuer": "did:key:z6MkkUbCFazdoducKf8SUye7cAxuicMdDBhXKWuTEuGA3jQF",
            "issuanceDate": "2021-03-24T20:03:03Z",
            "expirationDate": "2021-06-24T20:03:03Z",
            "credentialSubject": {
              "overAge": 21,
              "concealedIdToken": "zo58FV8vqzY2ZqLT4fSaVhe7CsdBKsUikBMbKridqSyc7LceLmgWcNTeHm2gfvgjuNjrVif1G2A5EKx2eyNkSu5ZBc6gNnjF8ZkV3P8dPrX8o46SF"
            },
            "proof": {
              "type": "Ed25519Signature2020",
              "created": "2021-08-07T21:36:26Z",
              "verificationMethod": "did:key:z6MkkUbCFazdoducKf8SUye7cAxuicMdDBhXKWuTEuGA3jQF#z6MkkUbCFazdoducKf8SUye7cAxuicMdDBhXKWuTEuGA3jQF",
              "proofPurpose": "assertionMethod",
              "proofValue": "z4mAs9uHU16jR4xwPcbhHyRUc6BbaiJQE5MJwn3PCWkRXsriK9AMrQQMbjzG9XXFPNgngmQXHKUz23WRSu9jSxPCF"
            }
          }
        });
        let cborld_encoded = encode_truage(doc).unwrap();
        assert_eq!(cborld_encoded, cmp_hex);
    }

    #[async_std::test]
    async fn test_incomplete_truage() {
        let _cmp_hex = hex::decode("d90501a300111874186e187ca8018311161418708203509f5ff197d6d44a3da68234fc799667431872a51874189618b61a637bbeed18be18c418c058417a1d84ac2b75c2ccfaf8e57cc7bc94df9b7314291a43eb0d57ead4dfdf2f0575a76046c510e889196afef3e949692262fde8dbfc5b525f72f9126c4e0fec1b460418c2831904015822ed0191fb716a4a661ec5fb6436b3f6225ebfc10a7eda60d2b4bb5e6fb3ecfbb1207f5822ed0191fb716a4a661ec5fb6436b3f6225ebfc10a7eda60d2b4bb5e6fb3ecfbb1207f187582186c1882189ea2188a58587ad90501a40015186a1864186c4b7ad90501a20016187c1841186e583b7a00009a3e130c62d700e40a8388a3a3ac4f8df1aed0e596d32ebdc528e223443e9a1ccb24fbc19480ec9ce03937e4548b5cc12a9c2d0634ed3fc01894184118a21a63f508ec18a41a637bbeec18a8821904015822ed0191fb716a4a661ec5fb6436b3f6225ebfc10a7eda60d2b4bb5e6fb3ecfbb1207f").unwrap();

        let doc: Value = serde_json::json!({
            "@context": "https://www.w3.org/2018/credentials/v1",
            "type": "VerifiablePresentation",
            "verifiableCredential": {
                "@context": [
                    "https://www.w3.org/2018/credentials/v1",
                    "https://w3id.org/age/v1",
                    "https://w3id.org/security/suites/ed25519-2020/v1"
                ],
                "id": "urn:uuid:9f5ff197-d6d4-4a3d-a682-34fc79966743",
                "type": [
                    "VerifiableCredential",
                    "OverAgeTokenCredential"
                ],
                "issuer": "did:key:z6MkpH7YDw3LBmqTmUzifCBe999t8DatvWnpxSgYQn9UEeyc",
                "issuanceDate": "2022-11-21T18:09:48Z",
                "expirationDate": "2023-02-21T18:09:48Z",
                // "credentialSubject": {
                //     "overAge": 65,
                //     "concealedIdToken": "zPwe8eWs7Gv9pfQ2UL6y17BfCNYFx2fiHGqChnf4jK5wdtH6EgeBM6jNshNYvBYkZjudjGWyEyi5zjBVBkMtdgN7V7AKnL5BSGcxi25KpGk6KQDP9mRKHfWw"
                // },
                "proof": {
                    "type": "Ed25519Signature2020",
                    "created": "2022-11-21T18:09:49Z",
                    "verificationMethod": "did:key:z6MkpH7YDw3LBmqTmUzifCBe999t8DatvWnpxSgYQn9UEeyc#z6MkpH7YDw3LBmqTmUzifCBe999t8DatvWnpxSgYQn9UEeyc",
                    "proofPurpose": "assertionMethod",
                    "proofValue": "zbEKA8bqX3cYWJ5cYEQztNy9m3pR1L3QyDxoKcu7jXWyWz8NzKvbtpeWFnzReLAoWD2exshvto1fxbf7H7H6Vfsh"
                }
            }
        });

        let cborld_encoded = encode_truage(doc);
        match cborld_encoded {
            Ok(_v) => {
                panic!()
            }
            Err(e) => {
                assert_eq!(
                    e,
                    TruageCborLdError::UnexpectedFormat(
                        "failure encoding credential_subject".to_string()
                    )
                );
            }
        }
    }

    #[async_std::test]
    async fn test_age_over_for_underaged() {
        let cmp_hex = hex::decode("d90501a300111874186e187ca8018311161418708203509f5ff197d6d44a3da68234fc799667431872a51874189618b61a637bbeed18be18c418c058417a1d84ac2b75c2ccfaf8e57cc7bc94df9b7314291a43eb0d57ead4dfdf2f0575a76046c510e889196afef3e949692262fde8dbfc5b525f72f9126c4e0fec1b460418c2831904015822ed0191fb716a4a661ec5fb6436b3f6225ebfc10a7eda60d2b4bb5e6fb3ecfbb1207f5822ed0191fb716a4a661ec5fb6436b3f6225ebfc10a7eda60d2b4bb5e6fb3ecfbb1207f187582186c1882189ea2188a58587ad90501a40015186a1864186c4b7ad90501a20016187c1841186e583b7a00009a3e130c62d700e40a8388a3a3ac4f8df1aed0e596d32ebdc528e223443e9a1ccb24fbc19480ec9ce03937e4548b5cc12a9c2d0634ed3fc018940018a21a63f508ec18a41a637bbeec18a8821904015822ed0191fb716a4a661ec5fb6436b3f6225ebfc10a7eda60d2b4bb5e6fb3ecfbb1207f").unwrap();
        let doc: Value = serde_json::json!({
            "@context": "https://www.w3.org/2018/credentials/v1",
            "type": "VerifiablePresentation",
            "verifiableCredential": {
                "@context": [
                    "https://www.w3.org/2018/credentials/v1",
                    "https://w3id.org/age/v1",
                    "https://w3id.org/security/suites/ed25519-2020/v1"
                ],
                "id": "urn:uuid:9f5ff197-d6d4-4a3d-a682-34fc79966743",
                "type": [
                    "VerifiableCredential",
                    "OverAgeTokenCredential"
                ],
                "issuer": "did:key:z6MkpH7YDw3LBmqTmUzifCBe999t8DatvWnpxSgYQn9UEeyc",
                "issuanceDate": "2022-11-21T18:09:48Z",
                "expirationDate": "2023-02-21T18:09:48Z",
                "credentialSubject": {
                    "overAge": 0,
                    "concealedIdToken": "zPwe8eWs7Gv9pfQ2UL6y17BfCNYFx2fiHGqChnf4jK5wdtH6EgeBM6jNshNYvBYkZjudjGWyEyi5zjBVBkMtdgN7V7AKnL5BSGcxi25KpGk6KQDP9mRKHfWw"
                },
                "proof": {
                    "type": "Ed25519Signature2020",
                    "created": "2022-11-21T18:09:49Z",
                    "verificationMethod": "did:key:z6MkpH7YDw3LBmqTmUzifCBe999t8DatvWnpxSgYQn9UEeyc#z6MkpH7YDw3LBmqTmUzifCBe999t8DatvWnpxSgYQn9UEeyc",
                    "proofPurpose": "assertionMethod",
                    "proofValue": "zbEKA8bqX3cYWJ5cYEQztNy9m3pR1L3QyDxoKcu7jXWyWz8NzKvbtpeWFnzReLAoWD2exshvto1fxbf7H7H6Vfsh"
                }
            }
        });

        let cborld_encoded = encode_truage(doc).unwrap();

        assert_eq!(cborld_encoded, cmp_hex);
    }
}
