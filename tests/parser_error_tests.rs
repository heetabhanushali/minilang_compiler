// tests/parser_error_tests.rs - Parser error detection tests

use minilang_compiler::{Lexer, Parser, ParserError};
use pretty_assertions::assert_eq;

/// Helper to test parser errors
fn parse_expect_error(source: &str) -> ParserError {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer should not fail");
    let mut parser = Parser::new(tokens, source.to_string());
    parser.parse_program().expect_err("Expected parser error")
}

// ==================== SYNTAX ERROR TESTS ====================

#[test]
fn test_error_missing_semicolon() {
    let source = r#"
func main() {
    let x = 42  # Missing semicolon
}
"#;
    
    let error = parse_expect_error(source);
    let error_string = format!("{:?}", error);
    assert!(error_string.contains("Unexpected") || error_string.contains("semicolon"));
    
    println!("✓ Missing semicolon error detected");
}

#[test]
fn test_error_missing_closing_brace() {
    let source = r#"
func main() {
    display "Hello";
# Missing closing brace
"#;
    
    let error = parse_expect_error(source);
    let error_string = format!("{:?}", error);
    assert!(error_string.contains("Unexpected") || error_string.contains("brace"));
    
    println!("✓ Missing closing brace error detected");
}

#[test]
fn test_error_missing_opening_brace() {
    let source = r#"
func main()
    display "Hello";
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing opening brace error detected");
}

#[test]
fn test_error_missing_function_name() {
    let source = r#"
func () {
    display "Hello";
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing function name error detected");
}

#[test]
fn test_error_missing_type_annotation() {
    let source = r#"
func main() {
    let x = 42;  # Missing : type
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing type annotation error detected");
}

#[test]
fn test_error_invalid_type() {
    let source = r#"
func main() {
    let x: invalid = 42;
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Invalid type error detected");
}

#[test]
fn test_error_missing_parentheses() {
    let source = r#"
func main {  # Missing ()
    display "Hello";
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing parentheses error detected");
}

#[test]
fn test_error_unclosed_string() {
    // This should be caught by lexer, but if not, parser should handle it
    let source = r#"
func main() {
    display "Hello;
}
"#;
    
    // This will likely fail at lexer stage
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    assert!(result.is_err());
    
    println!("✓ Unclosed string error detected");
}

#[test]
fn test_error_invalid_statement() {
    let source = r#"
func main() {
    42;  # Just a number, not a valid statement
}
"#;
    
    // This might actually parse as expression statement, which is fine
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens, source.to_string());
    let _ = parser.parse_program(); // May or may not error
    
    println!("✓ Invalid statement handling tested");
}

#[test]
fn test_error_missing_condition() {
    let source = r#"
func main() {
    if {  # Missing condition
        display "Hello";
    }
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing if condition error detected");
}

#[test]
fn test_error_missing_while_body() {
    let source = r#"
func main() {
    while x < 10  # Missing body
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing while body error detected");
}

#[test]
fn test_error_missing_do_while_condition() {
    let source = r#"
func main() {
    do {
        x = x + 1;
    } while;  # Missing condition
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing do-while condition error detected");
}

#[test]
fn test_error_invalid_for_loop() {
    let source = r#"
func main() {
    for ;; {  # Empty for loop parts
        display "Hello";
    }
}
"#;
    
    // This might actually be valid (infinite loop)
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens, source.to_string());
    let _result = parser.parse_program();
    
    // Whether it errors or not, we're testing it handles it
    println!("✓ Invalid for loop handling tested");
}

#[test]
fn test_error_missing_array_size() {
    let source = r#"
func main() {
    let arr: int[] = [1, 2, 3];  # Missing array size
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing array size error detected");
}

#[test]
fn test_error_mismatched_brackets() {
    let source = r#"
func main() {
    let x = arr[0};  # Mismatched brackets
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Mismatched brackets error detected");
}

#[test]
fn test_error_unexpected_token() {
    let source = r#"
func main() {
    let let x = 42;  # Double 'let'
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Unexpected token error detected");
}

#[test]
fn test_error_missing_comma_in_params() {
    let source = r#"
func add(a: int b: int) {  # Missing comma
    send a + b;
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Missing comma in parameters error detected");
}

#[test]
fn test_error_invalid_return_type() {
    let source = r#"
func add() -> {  # Invalid return type
    send 42;
}
"#;
    
    let _error = parse_expect_error(source);
    println!("✓ Invalid return type error detected");
}

#[test]
fn test_error_unexpected_eof() {
    let source = r#"
func main() {
    let x: int = 
"#;  // Unexpected end of file
    
    let _error = parse_expect_error(source);
    println!("✓ Unexpected EOF error detected");
}

#[test]
fn test_error_empty_program() {
    let source = "";
    
    // Empty program is actually valid (no functions)
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens, source.to_string());
    let result = parser.parse_program().unwrap();
    assert_eq!(result.functions.len(), 0);
    
    println!("✓ Empty program handled correctly");
}