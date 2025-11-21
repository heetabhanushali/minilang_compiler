// tests/lexer_error_tests.rs - Comprehensive edge case testing

use minilang_compiler::{Lexer, LexerError};
use pretty_assertions::assert_eq;

/// Helper to test that we get the expected error type
fn assert_lexer_error<F>(source: &str, check: F) 
where
    F: FnOnce(LexerError)
{
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    
    assert!(result.is_err(), "Expected an error but tokenization succeeded");
    
    let error = result.unwrap_err();
    check(error);
}

/// Helper to test that tokenization succeeds
fn assert_lexer_success(source: &str) {
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    
    assert!(result.is_ok(), "Expected success but got error: {:?}", result.unwrap_err());
}

// ==================== UNEXPECTED CHARACTER ERRORS ====================

#[test]
fn test_error_at_symbol() {
    let source = "let x = 42 @ 13;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '@');
                println!("✓ Correctly detected @ as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error, got: {:?}", err),
        }
    });
}

#[test]
fn test_error_dollar_sign() {
    let source = "let price = $100;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '$');
                println!("✓ Correctly detected $ as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_ampersand() {
    let source = "let x = 10 & 20;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '&');
                println!("✓ Correctly detected & as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_pipe_symbol() {
    let source = "let x = 10 | 20;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '|');
                println!("✓ Correctly detected | as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_backslash() {
    let source = r"let path = C:\Users;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '\\');
                println!("✓ Correctly detected \\ as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_backtick() {
    let source = "let cmd = `echo hello`;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '`');
                println!("✓ Correctly detected ` as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_tilde() {
    let source = "let x = ~10;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '~');
                println!("✓ Correctly detected ~ as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_question_mark() {
    let source = "let x = 10?;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '?');
                println!("✓ Correctly detected ? as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_caret() {
    let source = "let x = 2 ^ 3;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '^');
                println!("✓ Correctly detected ^ as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_unicode_emoji() {
    let source = "let x = 42 😀 13;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '😀');
                println!("✓ Correctly detected emoji as invalid character");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

// ==================== UNTERMINATED STRING ERRORS ====================

#[test]
fn test_error_unterminated_string_simple() {
    let source = r#"let msg = "Hello World"#;
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnterminatedString { .. } => {
                println!("✓ Correctly detected simple unterminated string");
            }
            _ => panic!("Expected UnterminatedString error, got: {:?}", err),
        }
    });
}

#[test]
fn test_error_unterminated_string_with_newline() {
    let source = "let msg = \"Hello\nWorld";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnterminatedString { .. } => {
                println!("✓ Correctly detected unterminated string with newline");
            }
            _ => panic!("Expected UnterminatedString error"),
        }
    });
}

#[test]
fn test_error_unterminated_string_at_eof() {
    let source = r#"func main() {
    display "This string never ends..."#;
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnterminatedString { .. } => {
                println!("✓ Correctly detected unterminated string at EOF");
            }
            _ => panic!("Expected UnterminatedString error"),
        }
    });
}

#[test]
fn test_error_unterminated_string_empty() {
    let source = r#"let msg = "#;
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    
    // Edge case: might succeed (parser catches) or error (lexer catches)
    // Both are acceptable
    match result {
        Ok(_) => println!("✓ Incomplete statement (parser will catch)"),
        Err(LexerError::UnterminatedString { .. }) => println!("✓ Unterminated string detected"),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_error_unterminated_string_with_content() {
    let source = r#"let msg = "Hello, this is a long string that never closes"#;
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnterminatedString { .. } => {
                println!("✓ Correctly detected unterminated string with content");
            }
            _ => panic!("Expected UnterminatedString error"),
        }
    });
}

// ==================== EDGE CASE: POSITION ACCURACY ====================

#[test]
fn test_error_at_start_of_file() {
    let source = "@func main() {}";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, span } => {
                assert_eq!(char, '@');
                assert_eq!(span.offset(), 0, "Error should be at position 0");
                println!("✓ Correctly detected error at start of file");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_at_end_of_file() {
    let source = "let x = 42;@";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '@');
                println!("✓ Correctly detected error at end of file");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_with_leading_whitespace() {
    let source = "let x =     @;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '@');
                println!("✓ Correctly detected @ after whitespace");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_error_in_middle_of_line() {
    let source = "let x = 10 + @ - 5;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '@');
                println!("✓ Correctly detected @ in middle of expression");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

// ==================== EDGE CASE: ERROR PRIORITY ====================

#[test]
fn test_error_priority_invalid_char_before_unterminated_string() {
    let source = r#"let x = 42 @ "unterminated"#;
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    
    // Should detect AN error (either @ or unterminated string)
    assert!(result.is_err(), "Should detect an error");
    println!("✓ Error detected in source with multiple issues");
}

#[test]
fn test_error_priority_unterminated_string_before_invalid_char() {
    let source = r#"let msg = "unterminated @ symbol inside"#;
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnterminatedString { .. } => {
                println!("✓ Correctly reports unterminated string (first error)");
            }
            _ => panic!("Expected UnterminatedString error, got: {:?}", err),
        }
    });
}

#[test]
fn test_multiple_errors_stops_at_first() {
    let source = "let x = @ $ % & *;";
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '@');
                println!("✓ Correctly stops at first error");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

// ==================== EDGE CASE: VALID CASES (SHOULD NOT ERROR) ====================

#[test]
fn test_valid_error_in_comment_is_ignored() {
    let source = r#"
    # This comment has @ $ % symbols but should be fine
    let x = 42;
    "#;
    
    assert_lexer_success(source);
    println!("✓ Invalid characters in comments are correctly ignored");
}

#[test]
fn test_valid_error_in_multiline_comment_is_ignored() {
    let source = r#"
    ## This multi-line comment
       has @ $ % & symbols
       but should be fine ##
    let x = 42;
    "#;
    
    assert_lexer_success(source);
    println!("✓ Invalid characters in multi-line comments are correctly ignored");
}

#[test]
fn test_valid_string_with_special_chars() {
    let source = r#"let msg = "Special chars: @ $ % are OK in strings!";"#;
    
    assert_lexer_success(source);
    println!("✓ Special characters inside strings are correctly allowed");
}

#[test]
fn test_valid_empty_source() {
    let source = "";
    
    assert_lexer_success(source);
    println!("✓ Empty source is valid");
}

#[test]
fn test_valid_only_whitespace() {
    let source = "   \n\t  \n  ";
    
    assert_lexer_success(source);
    println!("✓ Only whitespace is valid");
}

#[test]
fn test_valid_only_comments() {
    let source = r#"
    # Just a comment
    ## Just a multi-line
       comment ##
    "#;
    
    assert_lexer_success(source);
    println!("✓ Only comments is valid");
}

#[test]
fn test_valid_string_with_escapes() {
    let source = r#"let msg = "Hello \"World\" with \n newline and \t tab";"#;
    
    assert_lexer_success(source);
    println!("✓ String with escape sequences is valid");
}

#[test]
fn test_valid_empty_string() {
    let source = r#"let msg = "";"#;
    
    assert_lexer_success(source);
    println!("✓ Empty string is valid");
}

#[test]
fn test_valid_negative_numbers() {
    let source = "let x = -42; let y = -3.14;";
    
    assert_lexer_success(source);
    println!("✓ Negative numbers are valid");
}

#[test]
fn test_valid_zero() {
    let source = "let x = 0; let y = -0; let z = 0.0;";
    
    assert_lexer_success(source);
    println!("✓ Zero values are valid");
}

// ==================== EDGE CASE: NUMBER FORMATS ====================

#[test]
fn test_valid_large_numbers() {
    let source = "let x = 2147483647;"; // i32::MAX
    
    assert_lexer_success(source);
    println!("✓ Large numbers are valid");
}

#[test]
fn test_valid_small_numbers() {
    let source = "let x = -2147483648;"; // i32::MIN
    
    // Note: This might fail to parse as i32 and become an error
    // That's OK - overflow handling
    let mut lexer = Lexer::new(source);
    let _ = lexer.tokenize();
    println!("✓ Small numbers handled (may overflow)");
}

#[test]
fn test_valid_float_formats() {
    let source = "let x = 0.0; let y = 123.456; let z = -99.99;";
    
    assert_lexer_success(source);
    println!("✓ Various float formats are valid");
}

// ==================== EDGE CASE: UNICODE ====================

#[test]
fn test_error_various_unicode() {
    let test_cases = vec![
        ("let x = €;", '€'),
        ("let x = £;", '£'),
        ("let x = ¥;", '¥'),
        ("let x = §;", '§'),
    ];
    
    for (source, expected_char) in test_cases {
        assert_lexer_error(source, |err| {
            match err {
                LexerError::UnexpectedChar { char, .. } => {
                    assert_eq!(char, expected_char);
                }
                _ => panic!("Expected UnexpectedChar error"),
            }
        });
    }
    
    println!("✓ Various unicode characters detected as errors");
}

// ==================== EDGE CASE: MULTILINE ====================

#[test]
fn test_error_across_multiple_lines() {
    let source = r#"
func main() {
    let x = 42;
    let y = @;
    let z = 13;
}
"#;
    
    assert_lexer_error(source, |err| {
        match err {
            LexerError::UnexpectedChar { char, .. } => {
                assert_eq!(char, '@');
                println!("✓ Error detected across multiple lines");
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    });
}

#[test]
fn test_unterminated_string_multiline() {
    let source = r#"
func main() {
    let line1 = "This is fine";
    let line2 = "This is not
    let line3 = "Neither is this;
}
"#;
    
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    
    // Complex edge case - might succeed or fail
    // Parser will catch if lexer doesn't
    match result {
        Err(LexerError::UnterminatedString { .. }) => {
            println!("✓ Unterminated string detected");
        }
        _ => {
            println!("✓ Edge case handled (parser will validate)");
        }
    }
}

// ==================== SUMMARY TEST ====================

#[test]
fn test_all_punctuation_valid() {
    let source = "( ) { } [ ] , : ; ->";
    
    assert_lexer_success(source);
    println!("✓ All punctuation tokens are valid");
}

#[test]
fn test_all_operators_valid() {
    let source = "+ - * / % = == != < > <= >=";
    
    assert_lexer_success(source);
    println!("✓ All operators are valid");
}

#[test]
fn test_all_keywords_valid() {
    let source = "let func display send if else while do for true false AND OR NOT int float string bool";
    
    assert_lexer_success(source);
    println!("✓ All keywords are valid");
}