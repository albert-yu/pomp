use std::borrow::Cow;
use std::fmt;

use markup_fmt::{Language, config::FormatOptions, format_text};

#[derive(Debug)]
pub enum HtmlError {
    FormatError(String),
}

impl fmt::Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FormatError(msg) => write!(f, "Invalid HTML - {}", msg),
        }
    }
}

pub fn html_format(buffer: &str) -> Result<String, HtmlError> {
    let mut options = FormatOptions::default();
    options.layout.indent_width = 2;
    options.layout.use_tabs = false;

    format_text(buffer, Language::Html, &options, |code, _| {
        Ok::<Cow<'_, str>, anyhow::Error>(Cow::Borrowed(code))
    })
    .map_err(|e| HtmlError::FormatError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_format_simple() {
        let input = "<div class=\"test\">\n<h1>Hello</h1>\n<p>World</p>\n</div>";
        let formatted = html_format(input).unwrap();
        assert_eq!(
            formatted,
            "<div class=\"test\">\n  <h1>Hello</h1>\n  <p>World</p>\n</div>\n"
        );
    }

    #[test]
    fn test_html_format_nested() {
        let input = "<html>\n<head>\n<title>Test</title>\n</head>\n<body>\n<h1>Title</h1>\n<p>Body</p>\n</body>\n</html>";
        let formatted = html_format(input).unwrap();
        assert_eq!(
            formatted,
            "<html>\n  <head>\n    <title>Test</title>\n  </head>\n  <body>\n    <h1>Title</h1>\n    <p>Body</p>\n  </body>\n</html>\n"
        );
    }

    #[test]
    fn test_html_format_indent_2() {
        let input = "<ul>\n<li>Item 1</li>\n<li>\n<span>Subitem</span>\n</li>\n</ul>";
        let formatted = html_format(input).unwrap();
        assert_eq!(
            formatted,
            "<ul>\n  <li>Item 1</li>\n  <li>\n    <span>Subitem</span>\n  </li>\n</ul>\n"
        );
    }

    #[test]
    fn test_html_format_invalid() {
        let input = "<div <invalid";
        let result = html_format(input);
        assert!(result.is_err());
    }
}
