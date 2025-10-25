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

pub fn base64_decode(buffer: &str) -> Result<String, DecodeError> {
    let decoded_bytes = general_purpose::STANDARD.decode(buffer)?;
    let decoded_str = String::from_utf8(decoded_bytes)?;
    Ok(decoded_str)
}
