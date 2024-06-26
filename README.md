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

## Command-line interface

A command-line interface is provided to easily encode and decode CBOR-LD
documents from the terminal.

### Install & run

You can install the command-line interface using the `bin` feature:
```console
cargo install --path . --features=bin
```

This will install a `cbor-ld` executable:
```console
cbor-ld <args>
```

Alternatively you can directly run the command-line interface without
installing it:
```console
cargo run --features=bin -- <args>
```

### Usage

Use the `-h` (`--help`) flag to display all the available commands and
options:
```console
cbor-ld -h
```

The executable provides two commands `encode` and `decode` to compress a
JSON-LD document into CBOR-LD, and back.
```console
cbor-ld encode path/to/input.jsonld > path/to/output.cbor
```

If no input file is given, the standard input will be used.
Using the `-x` (`--hexadecimal`) option the CBOR input/output will be
decoded/encoded as hexadecimal.
```console
cbor-ld decode -x path/to/input.cbor.hex > path/to/output.jsonld
```

By default remote JSON-LD contexts will be fetched online. You can change
this behavior by adding file-system endpoints for some URLs using the
`-m` (`--mount`) option, and/or disable HTTP queries alltogether using the
`-o` (`--offline`) flag.
```console
cbor-ld --offline -m "https://www.w3.org/ns/credentials=tests/contexts/credentials" decode path/to/input.cbor > path/to/output.jsonld
```

These options can also be provided using a TOML configuration files using
the `-f` (`--config`) option.
```console
cbor-ld -f path/to/config.toml encode path/to/input.jsonld > path/to/output.cbor
```

An example configuration file is provided at `tests/config.toml`.

<!-- cargo-rdme end -->
