use std::fmt;

#[derive(Debug)]
pub enum UrlDecodeError {
    InvalidEncoding,
}

impl fmt::Display for UrlDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidEncoding => {
                write!(f, "invalid URL encoding")
            }
        }
    }
}

pub fn url_decode(buffer: &str) -> Result<String, UrlDecodeError> {
    urlencoding::decode(buffer)
        .map(|s| s.into_owned())
        .map_err(|_| UrlDecodeError::InvalidEncoding)
}

pub fn url_encode(buffer: &str) -> String {
    urlencoding::encode(buffer).into_owned()
}
