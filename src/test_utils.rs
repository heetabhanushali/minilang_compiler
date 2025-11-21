// test_utils.rs - Helper functions for testing

use crate::{Token, Lexer};

/// Quick tokenize helper for tests
pub fn tokenize(input: &str) -> Result<Vec<Token>, crate::LexerError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    Ok(tokens.into_iter().map(|t| t.token).collect())
}

/// Check that tokenization fails
pub fn expect_error(input: &str) -> crate::LexerError {
    let mut lexer = Lexer::new(input);
    lexer.tokenize().expect_err("Expected an error")
}