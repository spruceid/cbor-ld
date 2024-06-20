mod common;
pub use common::*;

#[tokio::test]
async fn encode_trueage() {
    compression_test(
        include_str!("samples/trueage.jsonld"),
        include_str!("samples/trueage.cbor.hex"),
    )
    .await
}

#[tokio::test]
async fn decode_trueage() {
    decompression_test(
        include_str!("samples/trueage.cbor.hex"),
        include_str!("samples/trueage.jsonld"),
    )
    .await
}
