# CBOR-LD implementation for Rust

<!-- cargo-rdme start -->

This library provides a Rust implementation of [CBOR-LD], a compression
format for [JSON-LD] based on the [Concise Binary Object Representation
(CBOR)][CBOR].

[CBOR-LD]: <https://json-ld.github.io/cbor-ld-spec/>
[JSON-LD]: <https://www.w3.org/TR/json-ld/>
[CBOR]: <https://www.rfc-editor.org/rfc/rfc8949.html>

## Usage

```rust
// Parse an input JSON-LD document.
let json: cbor_ld::JsonValue = include_str!("../tests/samples/note.jsonld").parse().unwrap();

// Create a JSON-LD context loader.
let mut context_loader = json_ld::loader::ReqwestLoader::new();

// Encode (compress) the JSON-LD document into CBOR-LD.
let encoded: cbor_ld::CborValue = cbor_ld::encode(&json, &mut context_loader).await.unwrap();

// Decode (decompress) the CBOR-LD document back into JSON-LD.
let decoded: cbor_ld::JsonValue = cbor_ld::decode(&encoded, &mut context_loader).await.unwrap();

// The input and decoded JSON values should be equal
// (modulo objects entries ordering and some compact IRI expansions).
use json_syntax::BorrowUnordered;
assert_eq!(json.as_unordered(), decoded.as_unordered())
```

<!-- cargo-rdme end -->
