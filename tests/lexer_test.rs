// tests/lexer_test.rs - Comprehensive lexer tests

use minilang_compiler::{Token, test_utils::tokenize};
use pretty_assertions::assert_eq;

#[test]
fn test_integers() {
    assert_eq!(tokenize("42").unwrap(), vec![Token::Integer(42)]);
    assert_eq!(tokenize("-13").unwrap(), vec![Token::Integer(-13)]);
    assert_eq!(tokenize("0").unwrap(), vec![Token::Integer(0)]);
}

#[test]
fn test_floats() {
    assert_eq!(tokenize("3.14").unwrap(), vec![Token::Float(3.14)]);
    assert_eq!(tokenize("-2.5").unwrap(), vec![Token::Float(-2.5)]);
}

#[test]
fn test_strings() {
    assert_eq!(
        tokenize(r#""Hello""#).unwrap(),
        vec![Token::String("Hello".to_string())]
    );
    
    assert_eq!(
        tokenize(r#""Hello, World!""#).unwrap(),
        vec![Token::String("Hello, World!".to_string())]
    );
}

#[test]
fn test_keywords() {
    let tokens = tokenize("let func if else while for const break continue").unwrap();
    assert_eq!(tokens, vec![
        Token::Let,
        Token::Func,
        Token::If,
        Token::Else,
        Token::While,
        Token::For,
        Token::Const,
        Token::Break,
        Token::Continue,
    ]);
}

#[test]
fn test_types() {
    let tokens = tokenize("int float string bool").unwrap();
    assert_eq!(tokens, vec![
        Token::TypeInt,
        Token::TypeFloat,
        Token::TypeString,
        Token::TypeBool,
    ]);
}

#[test]
fn test_identifiers() {
    let tokens = tokenize("myVar x _test var123").unwrap();
    assert_eq!(tokens, vec![
        Token::Identifier("myVar".to_string()),
        Token::Identifier("x".to_string()),
        Token::Identifier("_test".to_string()),
        Token::Identifier("var123".to_string()),
    ]);
}

#[test]
fn test_operators() {
    let tokens = tokenize("+ - * / % =").unwrap();
    assert_eq!(tokens, vec![
        Token::Plus,
        Token::Minus,
        Token::Star,
        Token::Slash,
        Token::Percent,
        Token::Assign,
    ]);
}

#[test]
fn test_comparison() {
    let tokens = tokenize("== != < > <= >=").unwrap();
    assert_eq!(tokens, vec![
        Token::Equal,
        Token::NotEqual,
        Token::LessThan,
        Token::GreaterThan,
        Token::LessEqual,
        Token::GreaterEqual,
    ]);
}

#[test]
fn test_logical() {
    let tokens = tokenize("AND OR NOT").unwrap();
    assert_eq!(tokens, vec![
        Token::And,
        Token::Or,
        Token::Not,
    ]);
}

#[test]
fn test_punctuation() {
    let tokens = tokenize("( ) { } [ ] , : ; ->").unwrap();
    assert_eq!(tokens, vec![
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBrace,
        Token::RightBrace,
        Token::LeftBracket,
        Token::RightBracket,
        Token::Comma,
        Token::Colon,
        Token::Semicolon,
        Token::Arrow,
    ]);
}

#[test]
fn test_booleans() {
    let tokens = tokenize("true false").unwrap();
    assert_eq!(tokens, vec![
        Token::True,
        Token::False,
    ]);
}

#[test]
fn test_comments_ignored() {
    // Single line comment
    let tokens = tokenize("42 # comment\n13").unwrap();
    assert_eq!(tokens, vec![
        Token::Integer(42),
        Token::Integer(13),
    ]);
    
    // Multi-line comment
    let tokens = tokenize("42 ## comment ## 13").unwrap();
    assert_eq!(tokens, vec![
        Token::Integer(42),
        Token::Integer(13),
    ]);
}

#[test]
fn test_complete_statement() {
    let code = r#"let x: int = 42;"#;
    let tokens = tokenize(code).unwrap();
    assert_eq!(tokens, vec![
        Token::Let,
        Token::Identifier("x".to_string()),
        Token::Colon,
        Token::TypeInt,
        Token::Assign,
        Token::Integer(42),
        Token::Semicolon,
    ]);
}

#[test]
fn test_function_declaration() {
    let code = r#"func main() { display "Hello"; }"#;
    let tokens = tokenize(code).unwrap();
    assert_eq!(tokens, vec![
        Token::Func,
        Token::Identifier("main".to_string()),
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBrace,
        Token::Display,
        Token::String("Hello".to_string()),
        Token::Semicolon,
        Token::RightBrace,
    ]);
}

#[test]
fn test_invalid_character() {
    use minilang_compiler::test_utils::expect_error;
    use minilang_compiler::LexerError;
    
    let err = expect_error("42 @ 13");
    match err {
        LexerError::UnexpectedChar { char, .. } => {
            assert_eq!(char, '@');
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_string_with_braces() {
    // Make sure lexer passes strings with {} through
    let tokens = tokenize(r#""Hello {world}""#).unwrap();
    assert_eq!(tokens, vec![Token::String("Hello {world}".to_string())]);
    println!("✓ String with braces tokenizes");
}

#[test]
fn test_unterminated_string() {
    use minilang_compiler::test_utils::expect_error;
    use minilang_compiler::LexerError;
    
    let err = expect_error(r#""hello"#);
    match err {
        LexerError::UnterminatedString { .. } => {
            // Correct error type
        }
        _ => panic!("Wrong error type"),
    }
}