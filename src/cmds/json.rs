use std::fmt;

use serde_json::Value;

#[derive(Debug)]
pub enum JsonError {
    ParseError(String),
    FormatError(()),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseError(msg) => {
                write!(f, "Invalid JSON - {}", msg)
            }
            Self::FormatError(..) => {
                write!(f, "Failed to format JSON")
            }
        }
    }
}

impl From<serde_json::Error> for JsonError {
    fn from(err: serde_json::Error) -> JsonError {
        JsonError::ParseError(err.to_string())
    }
}

pub fn json_format(buffer: &str) -> Result<String, JsonError> {
    let json_value: Value = serde_json::from_str(buffer)?;
    serde_json::to_string_pretty(&json_value).map_err(|_| JsonError::FormatError(()))
}

pub fn json_minify(buffer: &str) -> Result<String, JsonError> {
    let json_value: Value = serde_json::from_str(buffer)?;
    serde_json::to_string(&json_value).map_err(|_| JsonError::FormatError(()))
}
