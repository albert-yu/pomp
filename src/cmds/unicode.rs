use std::fmt;

#[derive(Debug)]
pub enum UnicodeEscapeError {
    InvalidEscapeSequence(String),
    InvalidCodePoint,
}

impl fmt::Display for UnicodeEscapeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidEscapeSequence(seq) => {
                write!(f, "invalid unicode escape sequence: {}", seq)
            }
            Self::InvalidCodePoint => {
                write!(f, "invalid unicode code point")
            }
        }
    }
}

pub fn unicode_escape_decode(buffer: &str) -> Result<String, UnicodeEscapeError> {
    let mut result = String::new();
    let mut chars = buffer.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('u') => {
                    // Collect the next 4 hex digits
                    let hex_digits: String = chars.by_ref().take(4).collect();

                    if hex_digits.len() != 4 {
                        return Err(UnicodeEscapeError::InvalidEscapeSequence(
                            format!("\\u{}", hex_digits)
                        ));
                    }

                    // Parse the hex value
                    let code_point = u32::from_str_radix(&hex_digits, 16)
                        .map_err(|_| UnicodeEscapeError::InvalidEscapeSequence(
                            format!("\\u{}", hex_digits)
                        ))?;

                    // Convert to char
                    let unicode_char = char::from_u32(code_point)
                        .ok_or(UnicodeEscapeError::InvalidCodePoint)?;

                    result.push(unicode_char);
                }
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

pub fn unicode_escape_encode(buffer: &str) -> String {
    let mut result = String::new();

    for ch in buffer.chars() {
        if ch.is_ascii() && !ch.is_control() {
            result.push(ch);
        } else {
            // Encode as \uXXXX
            let code_point = ch as u32;
            if code_point <= 0xFFFF {
                result.push_str(&format!("\\u{:04x}", code_point));
            } else {
                // For code points beyond BMP, we need to use surrogate pairs or \U notation
                // For simplicity, we'll use Rust's escape_unicode
                for escaped_ch in ch.escape_unicode() {
                    result.push(escaped_ch);
                }
            }
        }
    }

    result
}
