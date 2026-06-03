use std::fmt::{Display, Formatter, Result};

// Error Handling for config parser: line number, error message

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        ParseError {
            line,
            message: message.into(),
        }
    }
}

impl Display for ParseError { // implement Display trait for better error messages display
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message) // "Parse error at line 42: Invalid port"
    }
}