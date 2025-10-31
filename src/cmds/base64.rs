use std::fmt;
use std::string::FromUtf8Error;

use base64::{Engine as _, engine::general_purpose};

#[derive(Debug)]
pub enum DecodeError {
    Base64DecodeError(()),
    FromUtf8Error(()),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Base64DecodeError(..) => {
                write!(f, "invalid base64 input")
            }
            Self::FromUtf8Error(..) => {
                write!(f, "decoded base64 is not valid UTF-8")
            }
        }
    }
}

impl From<base64::DecodeError> for DecodeError {
    fn from(_err: base64::DecodeError) -> DecodeError {
        DecodeError::Base64DecodeError(())
    }
}

impl From<FromUtf8Error> for DecodeError {
    fn from(_err: FromUtf8Error) -> DecodeError {
        DecodeError::FromUtf8Error(())
    }
}

fn add_base64_padding(input: &str) -> String {
    let trimmed = input.trim();
    let padding_needed = (4 - (trimmed.len() % 4)) % 4;
    if padding_needed == 0 {
        trimmed.to_string()
    } else {
        format!("{}{}", trimmed, "=".repeat(padding_needed))
    }
}

pub fn base64_decode(buffer: &str) -> Result<String, DecodeError> {
    let padded = add_base64_padding(buffer);
    let decoded_bytes = general_purpose::STANDARD.decode(&padded)?;
    let decoded_str = String::from_utf8(decoded_bytes)?;
    Ok(decoded_str)
}

pub fn base64_encode(buffer: &str) -> String {
    let encoded = general_purpose::STANDARD.encode(buffer.as_bytes());
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode_with_padding() {
        let input = "SGVsbG8gV29ybGQ=";
        let result = base64_decode(input).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_base64_decode_without_padding() {
        // "Hello World" in base64 is "SGVsbG8gV29ybGQ=" but we test without padding
        let input = "SGVsbG8gV29ybGQ";
        let result = base64_decode(input).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_base64_decode_missing_one_pad() {
        // "Hello!" in base64 is "SGVsbG8h" (no padding needed)
        // "Hello" in base64 is "SGVsbG8=" (1 padding needed)
        let input = "SGVsbG8";
        let result = base64_decode(input).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_base64_decode_missing_two_pads() {
        // "Hi" in base64 is "SGk=" normally, test without padding
        let input = "SGk";
        let result = base64_decode(input).unwrap();
        assert_eq!(result, "Hi");
    }

    #[test]
    fn test_base64_encode() {
        let input = "Hello World";
        let result = base64_encode(input);
        assert_eq!(result, "SGVsbG8gV29ybGQ=");
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = "Test string with special chars: !@#$%^&*()";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
