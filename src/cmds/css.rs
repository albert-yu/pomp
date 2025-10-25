use std::fmt;

use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};

#[derive(Debug)]
pub enum CssError {
    ParseError(String),
    MinifyError(String),
    FormatError(()),
}

impl fmt::Display for CssError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseError(msg) => {
                write!(f, "Invalid CSS - {}", msg)
            }
            Self::MinifyError(msg) => {
                write!(f, "Failed to minify CSS - {}", msg)
            }
            Self::FormatError(..) => {
                write!(f, "Failed to format CSS")
            }
        }
    }
}

pub fn css_format(buffer: &str) -> Result<String, CssError> {
    let stylesheet = StyleSheet::parse(buffer, ParserOptions::default())
        .map_err(|e| CssError::ParseError(e.to_string()))?;

    let printer_options = PrinterOptions {
        minify: false,
        ..Default::default()
    };

    stylesheet
        .to_css(printer_options)
        .map(|result| result.code)
        .map_err(|_| CssError::FormatError(()))
}

pub fn css_minify(buffer: &str) -> Result<String, CssError> {
    let mut stylesheet = StyleSheet::parse(buffer, ParserOptions::default())
        .map_err(|e| CssError::ParseError(e.to_string()))?;

    stylesheet
        .minify(MinifyOptions::default())
        .map_err(|e| CssError::MinifyError(e.to_string()))?;

    let printer_options = PrinterOptions {
        minify: true,
        ..Default::default()
    };

    stylesheet
        .to_css(printer_options)
        .map(|result| result.code)
        .map_err(|_| CssError::FormatError(()))
}
