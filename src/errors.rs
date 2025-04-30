/// A structured error type for parsing failures.
///
/// Contains detailed information about where and why parsing failed.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
    pub expected: Vec<String>,
}

impl ParseError {
    /// Create a new ParseError with a message and position.
    pub fn new(message: String, position: usize) -> Self {
        ParseError {
            message,
            position,
            expected: Vec::new(),
        }
    }

    /// Add an expected token to the error.
    pub fn add_expected(&mut self, token: String) {
        self.expected.push(token);
    }
}
