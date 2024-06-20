mod common;
pub use common::*;

#[tokio::test]
async fn encode_trueage() {
    compression_test(
        include_str!("trueage.jsonld"),
        include_str!("trueage.cbor.hex"),
    )
    .await
}

#[tokio::test]
async fn decode_trueage() {
    decompression_test(
        include_str!("trueage.cbor.hex"),
        include_str!("trueage.jsonld"),
    )
    .await
}
