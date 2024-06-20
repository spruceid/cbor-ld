use std::str::FromStr;

use cbor_ld::{decode_from_bytes, encode_to_bytes, JsonValue};
use iref::IriBuf;
use json_ld::{FsLoader, Print};
use json_syntax::BorrowUnordered;
use static_iref::iri;

pub fn create_context_loader() -> FsLoader<IriBuf> {
    let mut loader = FsLoader::new();

    loader.mount(
        iri!("https://w3id.org/security").to_owned(),
        "tests/contexts/security",
    );
    loader.mount(
        iri!("https://www.w3.org/2018/credentials").to_owned(),
        "tests/contexts/credentials",
    );
    loader.mount(
        iri!("https://w3id.org/age").to_owned(),
        "tests/contexts/age",
    );
    loader.mount(
        iri!("https://w3id.org/cit").to_owned(),
        "tests/contexts/cit",
    );

    loader
}

pub async fn compression_test(input: &str, expected_hex: &str) {
    let json = cbor_ld::JsonValue::from_str(input).unwrap();
    let expected_bytes = hex::decode(expected_hex).unwrap();
    let bytes = encode_to_bytes(&json, create_context_loader())
        .await
        .unwrap();

    eprint!("output   = ");
    diff(&bytes, &expected_bytes);
    eprint!("expected = ");
    diff(&expected_bytes, &bytes);

    assert_eq!(bytes, expected_bytes)
}

pub async fn decompression_test(input_hex: &str, expected: &str) {
    let bytes = hex::decode(input_hex).unwrap();

    let json = decode_from_bytes(&bytes, create_context_loader())
        .await
        .unwrap();

    let expected_json: JsonValue = expected.parse().unwrap();

    eprintln!("output   = {}", json.pretty_print());
    eprintln!("expected = {}", expected_json.pretty_print());

    assert_eq!(json.as_unordered(), expected_json.as_unordered())
}

pub fn diff(value: &[u8], expected: &[u8]) {
    use yansi::Paint;

    let mut bytes = value.iter();
    let mut expected_bytes = expected.iter();

    #[derive(PartialEq)]
    enum State {
        None,
        Eq,
        Neq,
        Missing,
        Added,
    }

    impl State {
        fn apply(&self) {
            match self {
                Self::None => (),
                Self::Eq => eprint!("{}", "".green().linger()),
                Self::Neq => eprint!("{}", "".red().linger()),
                Self::Missing => eprint!("{}", "".white().strike().on_red()),
                Self::Added => eprintln!("{}", "".white().on_red()),
            }
        }

        fn print(&mut self, value: u8, next_state: State) {
            if *self != next_state {
                next_state.apply();
                *self = next_state;
            }

            eprint!("{value:02x}");
        }
    }

    let mut state = State::None;

    loop {
        match (bytes.next(), expected_bytes.next()) {
            (Some(a), Some(b)) => state.print(*a, if a == b { State::Eq } else { State::Neq }),
            (Some(a), None) => state.print(*a, State::Added),
            (None, Some(b)) => state.print(*b, State::Missing),
            (None, None) => break,
        }
    }

    eprintln!("{}", "".resetting())
}
