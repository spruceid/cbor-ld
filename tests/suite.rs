mod common;
use cbor_ld::{tables::RegistryEntry, CompressionMode};
pub use common::*;

#[tokio::test]
async fn encode_note() {
    compression_test(
        include_str!("samples/note.jsonld"),
        include_str!("samples/note.cbor.hex"),
    )
    .await
}

#[tokio::test]
async fn decode_note() {
    decompression_test(
        include_str!("samples/note.cbor.hex"),
        include_str!("samples/note.jsonld"),
    )
    .await
}

#[tokio::test]
async fn encode_prc() {
    compression_test(
        include_str!("samples/prc.jsonld"),
        include_str!("samples/prc.cbor.hex"),
    )
    .await
}

#[tokio::test]
async fn decode_prc() {
    decompression_test(
        include_str!("samples/prc.cbor.hex"),
        include_str!("samples/prc.jsonld"),
    )
    .await
}

#[tokio::test]
async fn encode_truage() {
    compression_test(
        include_str!("samples/truage.jsonld"),
        include_str!("samples/truage.cbor.hex"),
    )
    .await
}

#[tokio::test]
async fn decode_truage() {
    decompression_test(
        include_str!("samples/truage.cbor.hex"),
        include_str!("samples/truage.jsonld"),
    )
    .await
}

#[tokio::test]
async fn encode_uncompressible() {
    let json: cbor_ld::JsonValue = include_str!("samples/uncompressible.jsonld")
        .parse()
        .unwrap();
    assert!(cbor_ld::encode(&json, create_context_loader())
        .await
        .is_err())
}

#[tokio::test]
async fn encode_vcb_aamva() {
    compression_test_with(
        include_str!("samples/vcb-aamva.jsonld"),
        include_str!("samples/vcb-aamva.cbor.hex"),
        CompressionMode::Compressed(RegistryEntry::VcBarcodes),
    )
    .await
}

#[tokio::test]
async fn decode_vcb_aamva() {
    decompression_test(
        include_str!("samples/vcb-aamva.cbor.hex"),
        include_str!("samples/vcb-aamva.jsonld"),
    )
    .await
}

#[tokio::test]
async fn encode_vcb_mrz() {
    compression_test_with(
        include_str!("samples/vcb-mrz.jsonld"),
        include_str!("samples/vcb-mrz.cbor.hex"),
        CompressionMode::Compressed(RegistryEntry::VcBarcodes),
    )
    .await
}

#[tokio::test]
async fn decode_vcb_mrz() {
    decompression_test(
        include_str!("samples/vcb-mrz.cbor.hex"),
        include_str!("samples/vcb-mrz.jsonld"),
    )
    .await
}
