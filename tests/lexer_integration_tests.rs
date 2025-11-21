// tests/lexer_integration_tests.rs - Real-world tokenization tests

use minilang_compiler::{Token, Lexer};
use pretty_assertions::assert_eq;

/// Helper to tokenize and extract just the token types
fn tokenize(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
        .expect("Tokenization should succeed")
        .into_iter()
        .map(|t| t.token)
        .collect()
}

// ==================== REAL PROGRAMS ====================

#[test]
fn test_hello_world_program() {
    let source = r#"
func main() {
    display "Hello, World!";
}
"#;
    
    let tokens = tokenize(source);
    
    assert_eq!(tokens[0], Token::Func);
    assert_eq!(tokens[1], Token::Identifier("main".to_string()));
    assert_eq!(tokens[2], Token::LeftParen);
    assert_eq!(tokens[3], Token::RightParen);
    assert_eq!(tokens[4], Token::LeftBrace);
    assert_eq!(tokens[5], Token::Display);
    assert_eq!(tokens[6], Token::String("Hello, World!".to_string()));
    assert_eq!(tokens[7], Token::Semicolon);
    assert_eq!(tokens[8], Token::RightBrace);
    
    println!("✓ Hello World program tokenizes correctly");
}

#[test]
fn test_variable_declaration() {
    let source = "let x: int = 42;";
    let tokens = tokenize(source);
    
    assert_eq!(tokens, vec![
        Token::Let,
        Token::Identifier("x".to_string()),
        Token::Colon,
        Token::TypeInt,
        Token::Assign,
        Token::Integer(42),
        Token::Semicolon,
    ]);
    
    println!("✓ Variable declaration tokenizes correctly");
}

#[test]
fn test_function_with_parameters() {
    let source = "func add(a: int, b: int) -> int { send a + b; }";
    let tokens = tokenize(source);
    
    assert_eq!(tokens[0], Token::Func);
    assert_eq!(tokens[1], Token::Identifier("add".to_string()));
    assert_eq!(tokens[2], Token::LeftParen);
    assert_eq!(tokens[3], Token::Identifier("a".to_string()));
    assert_eq!(tokens[4], Token::Colon);
    assert_eq!(tokens[5], Token::TypeInt);
    assert_eq!(tokens[6], Token::Comma);
    assert_eq!(tokens[7], Token::Identifier("b".to_string()));
    
    println!("✓ Function with parameters tokenizes correctly");
}

#[test]
fn test_if_statement_with_and_or() {
    let source = r#"
if x > 10 AND y < 20 OR NOT z {
    display "Complex condition";
}
"#;
    
    let tokens = tokenize(source);
    
    // Check that AND, OR, NOT are recognized
    assert!(tokens.contains(&Token::And));
    assert!(tokens.contains(&Token::Or));
    assert!(tokens.contains(&Token::Not));
    
    println!("✓ Complex condition with AND/OR/NOT tokenizes correctly");
}

#[test]
fn test_array_declaration() {
    let source = "let arr: int[5] = [1, 2, 3, 4, 5];";
    let tokens = tokenize(source);
    
    assert_eq!(tokens[0], Token::Let);
    assert_eq!(tokens[1], Token::Identifier("arr".to_string()));
    assert_eq!(tokens[2], Token::Colon);
    assert_eq!(tokens[3], Token::TypeInt);
    assert_eq!(tokens[4], Token::LeftBracket);
    assert_eq!(tokens[5], Token::Integer(5));
    assert_eq!(tokens[6], Token::RightBracket);
    
    println!("✓ Array declaration tokenizes correctly");
}

#[test]
fn test_all_data_types() {
    let source = r#"
let i: int = 42;
let f: float = 3.14;
let s: string = "hello";
let b: bool = true;
"#;
    
    let tokens = tokenize(source);
    
    // Verify all type keywords appear
    assert!(tokens.contains(&Token::TypeInt));
    assert!(tokens.contains(&Token::TypeFloat));
    assert!(tokens.contains(&Token::TypeString));
    assert!(tokens.contains(&Token::TypeBool));
    
    // Verify all literal types
    assert!(tokens.contains(&Token::Integer(42)));
    assert!(tokens.contains(&Token::Float(3.14)));
    assert!(tokens.contains(&Token::String("hello".to_string())));
    assert!(tokens.contains(&Token::True));
    
    println!("✓ All data types tokenize correctly");
}

#[test]
fn test_all_operators() {
    let source = "x + y - z * w / v % m";
    let tokens = tokenize(source);
    
    assert!(tokens.contains(&Token::Plus));
    assert!(tokens.contains(&Token::Minus));
    assert!(tokens.contains(&Token::Star));
    assert!(tokens.contains(&Token::Slash));
    assert!(tokens.contains(&Token::Percent));
    
    println!("✓ All arithmetic operators tokenize correctly");
}

#[test]
fn test_all_comparisons() {
    let source = "x == y != z < w > v <= m >= n";
    let tokens = tokenize(source);
    
    assert!(tokens.contains(&Token::Equal));
    assert!(tokens.contains(&Token::NotEqual));
    assert!(tokens.contains(&Token::LessThan));
    assert!(tokens.contains(&Token::GreaterThan));
    assert!(tokens.contains(&Token::LessEqual));
    assert!(tokens.contains(&Token::GreaterEqual));
    
    println!("✓ All comparison operators tokenize correctly");
}

#[test]
fn test_nested_structures() {
    let source = r#"
func outer() {
    if x > 0 {
        while y < 10 {
            for i = 0; i < 5; i = i + 1 {
                display "nested";
            }
        }
    }
}
"#;
    
    let tokens = tokenize(source);
    
    // Should have multiple opening and closing braces
    let lbrace_count = tokens.iter().filter(|t| **t == Token::LeftBrace).count();
    let rbrace_count = tokens.iter().filter(|t| **t == Token::RightBrace).count();
    
    assert_eq!(lbrace_count, rbrace_count);
    assert!(lbrace_count >= 4, "Should have at least 4 pairs of braces");
    
    println!("✓ Nested structures tokenize correctly");
}

#[test]
fn test_comments_are_ignored() {
    let source = r#"
# This is a comment
let x = 42;  # inline comment
## Multi-line
   comment here ##
let y = 13;
"#;
    
    let tokens = tokenize(source);
    
    // Comments should not appear in tokens
    // We should only have tokens for the actual code
    assert!(tokens.contains(&Token::Let));
    assert!(tokens.contains(&Token::Integer(42)));
    assert!(tokens.contains(&Token::Integer(13)));
    
    // No comment-related tokens
    assert!(!tokens.iter().any(|t| matches!(t, Token::Identifier(s) if s.contains("comment"))));
    
    println!("✓ Comments are correctly ignored");
}

#[test]
fn test_whitespace_handling() {
    let source = "let    x   :   int   =   42  ;";
    let tokens = tokenize(source);
    
    // Should tokenize the same as without extra whitespace
    assert_eq!(tokens, vec![
        Token::Let,
        Token::Identifier("x".to_string()),
        Token::Colon,
        Token::TypeInt,
        Token::Assign,
        Token::Integer(42),
        Token::Semicolon,
    ]);
    
    println!("✓ Extra whitespace is correctly handled");
}

#[test]
fn test_negative_numbers() {
    let source = "let x = -42; let y = -3.14;";
    let tokens = tokenize(source);
    
    assert!(tokens.contains(&Token::Integer(-42)));
    assert!(tokens.contains(&Token::Float(-3.14)));
    
    println!("✓ Negative numbers tokenize correctly");
}

#[test]
fn test_empty_program() {
    let source = "";
    let tokens = tokenize(source);
    
    assert_eq!(tokens.len(), 0);
    println!("✓ Empty program tokenizes correctly");
}

#[test]
fn test_only_comments() {
    let source = r#"
# Just a comment
## Just a multi-line
   comment ##
"#;
    
    let tokens = tokenize(source);
    
    assert_eq!(tokens.len(), 0);
    println!("✓ Program with only comments tokenizes correctly");
}


#[test]
fn test_const_in_complete_program() {
    let source = r#"
const PI: float = 3.14159;
const MAX_SIZE: int = 100;

func calculateArea(radius: float) -> float {
    send PI * radius * radius;
}

func main() {
    let area: float = calculateArea(5.0);
    display "Area:", area;
}
"#;
    
    let tokens = tokenize(source);
    
    // Verify const keyword appears
    assert!(tokens.contains(&Token::Const));
    
    // Count const occurrences (should be 2)
    let const_count = tokens.iter().filter(|t| **t == Token::Const).count();
    assert_eq!(const_count, 2);
    
    println!("✓ Complete program with const tokenizes correctly");
}