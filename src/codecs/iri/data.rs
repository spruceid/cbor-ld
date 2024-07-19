use multibase::Base;

use super::IriCodec;
use crate::{CborValue, DecodeError, EncodeError};

pub struct DataUrlCodec;

impl IriCodec for DataUrlCodec {
    fn encode(&self, suffix: &str) -> Result<Vec<CborValue>, EncodeError> {
        if let Some(url) = DataUrl::new(suffix) {
            if url.base_64 {
                if let Ok(data) = Base::Base64.decode(url.data) {
                    return Ok(vec![
                        CborValue::Text(url.media_type.to_owned()),
                        CborValue::Bytes(data),
                    ]);
                }
            }
        }

        Ok(vec![CborValue::Text(suffix.to_owned())])
    }

    fn decode(&self, array: &[CborValue]) -> Result<String, DecodeError> {
        match array.len() {
            1 => {
                // no base64
                array[0]
                    .as_text()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| DecodeError::Codec("data", "expected text".to_string()))
            }
            2 => {
                // base64
                let media_type = array[0]
                    .as_text()
                    .ok_or_else(|| DecodeError::Codec("data", "expected text".to_string()))?;

                let data = array[1]
                    .as_bytes()
                    .ok_or_else(|| DecodeError::Codec("data", "expected bytes".to_string()))?;

                let base64_data = Base::Base64.encode(data);

                Ok(format!("{media_type};base64,{base64_data}"))
            }
            _ => Err(DecodeError::Codec(
                "data",
                "invalid array length".to_string(),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DataUrl<'a> {
    media_type: &'a str,
    base_64: bool,
    data: &'a str,
}

impl<'a> DataUrl<'a> {
    fn new(url: &'a str) -> Option<Self> {
        let mut chars = url.char_indices();
        // Parse media type.
        loop {
            match chars.next() {
                Some((i, ',')) => {
                    // no base64
                    break Some(Self {
                        media_type: &url[..i],
                        base_64: false,
                        data: &url[(i + 1)..],
                    });
                }
                Some((i, ';')) => {
                    // base64
                    let j = i + 8;
                    break if url.len() >= j && &url[(i + 1)..j] == "base64," {
                        Some(Self {
                            media_type: &url[..i],
                            base_64: true,
                            data: &url[j..],
                        })
                    } else {
                        None
                    };
                }
                Some((_, c)) if is_media_type_char(c) => (),
                _ => break None,
            }
        }
    }
}

fn is_media_type_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(c, '/' | '!' | '#' | '$' | '&' | '-' | '+' | '^' | '_' | '.')
}

#[cfg(test)]
mod tests {
    use super::DataUrl;

    #[test]
    fn parse_data_url_1() {
        assert!(DataUrl::new("invalid").is_none())
    }

    #[test]
    fn parse_data_url_2() {
        let found = DataUrl::new(",valid");
        let expected = DataUrl {
            media_type: "",
            base_64: false,
            data: "valid",
        };

        assert_eq!(found, Some(expected));
    }

    #[test]
    fn parse_data_url_3() {
        let found = DataUrl::new(";base64,");
        let expected = DataUrl {
            media_type: "",
            base_64: true,
            data: "",
        };

        assert_eq!(found, Some(expected));
    }

    #[test]
    fn parse_data_url_4() {
        let found = DataUrl::new(";base64,data");
        let expected = DataUrl {
            media_type: "",
            base_64: true,
            data: "data",
        };

        assert_eq!(found, Some(expected));
    }

    #[test]
    fn parse_data_url_5() {
        let found = DataUrl::new("image/jpeg,data");
        let expected = DataUrl {
            media_type: "image/jpeg",
            base_64: false,
            data: "data",
        };

        assert_eq!(found, Some(expected));
    }

    #[test]
    fn parse_data_url_6() {
        let found = DataUrl::new("image/jpeg;base64,data");
        let expected = DataUrl {
            media_type: "image/jpeg",
            base_64: true,
            data: "data",
        };

        assert_eq!(found, Some(expected));
    }

    #[test]
    fn parse_data_url_7() {
        let found = DataUrl::new("image/jpeg;base64,");
        let expected = DataUrl {
            media_type: "image/jpeg",
            base_64: true,
            data: "",
        };

        assert_eq!(found, Some(expected));
    }
}
