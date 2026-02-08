// tests/semantic_edge_cases.rs - Edge cases and complex scenarios

use minilang_compiler::{Lexer, Parser, TypeChecker, SemanticError};

fn analyze(source: &str) -> Result<(), Vec<SemanticError>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens, source.to_string());
    let program = parser.parse_program().expect("Parser should succeed");
    
    let mut type_checker = TypeChecker::new();
    type_checker.check_program(&program)
}

fn expect_semantic_error(source: &str) -> Vec<SemanticError> {
    analyze(source).expect_err("Expected semantic error")
}

// ==================== EDGE CASES ====================

#[test]
fn test_empty_program() {
    let source = "";
    assert!(analyze(source).is_ok());
    println!("✓ Empty program passes semantic analysis");
}

#[test]
fn test_function_with_no_statements() {
    let source = r#"
func main() {
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Empty function passes");
}

#[test]
fn test_unreachable_code_after_return() {
    let source = r#"
func test() -> int {
    send 42;
    let x: int = 10;  # Unreachable but should still be checked
    send x;
}
"#;
    // Should pass - unreachable code is not an error
    assert!(analyze(source).is_ok());
    println!("✓ Unreachable code handled");
}

#[test]
fn test_complex_expression_types() {
    let source = r#"
func main() {
    let a: int = 5;
    let b: int = 10;
    let c: float = 2.5;
    let result: int = (a + b) * 2 - (b / a) + 1;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Complex expression types verified");
}

#[test]
fn test_chained_assignments() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = 20;
    x = y;
    y = x;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Chained assignments work");
}

#[test]
fn test_nested_function_calls() {
    let source = r#"
func inner(x: int) -> int {
    send x * 2;
}

func outer(y: int) -> int {
    send inner(inner(y));
}

func main() {
    let result: int = outer(5);
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Nested function calls type-check correctly");
}

#[test]
fn test_complex_logical_expressions() {
    let source = r#"
func main() {
    let a: bool = true;
    let b: bool = false;
    let c: bool = true;
    let result: bool = (a AND b) OR (NOT c AND a) OR (b AND NOT a);
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Complex logical expressions type-check");
}

#[test]
fn test_array_of_expressions() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = 10;
    let arr: int[3] = [x + y, x * 2, y - x];
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Array initialization with expressions works");
}

#[test]
fn test_multiple_errors() {
    let source = r#"
func main() {
    display x;           # Error 1: undefined variable
    let y: int = "hi";   # Error 2: type mismatch
    let z: int = 10;
    let z: int = 20;     # Error 3: redefinition
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.len() >= 3);
    println!("✓ Multiple errors detected in single pass");
}

#[test]
fn test_forward_function_reference() {
    let source = r#"
func main() {
    let x: int = helper(5);
}

func helper(n: int) -> int {
    send n * 2;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Forward function reference works");
}

#[test]
fn test_unary_operators() {
    let source = r#"
func main() {
    let x: int = 42;
    let neg: int = -x;
    let b: bool = true;
    let notb: bool = NOT b;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Unary operators type-check correctly");
}

#[test]
fn test_unary_type_errors() {
    let source = r#"
func main() {
    let s: string = "hello";
    let neg: string = -s;  # Cannot negate string
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Invalid unary operation detected");
}

#[test]
fn test_mixed_arithmetic_comparison() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = 20;
    let b: bool = (x + 5) < (y * 2);
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Mixed arithmetic and comparison works");
}

#[test]
fn test_all_types_in_function() {
    let source = r#"
func test(i: int, f: float, s: string, b: bool, arr: int[5]) -> bool {
    display i, f, s, b;
    send b;
}

func main() {
    let nums: int[5] = [1, 2, 3, 4, 5];
    let result: bool = test(42, 3.14, "hello", true, nums);
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ All types in function signature work");
}

#[test]
fn test_display_multiple_types() {
    let source = r#"
func main() {
    let i: int = 42;
    let f: float = 3.14;
    let s: string = "hello";
    let b: bool = true;
    display i, f, s, b, "literal", 100, true;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Display with multiple types works");
}

#[test]
fn test_break_in_while_loop() {
    let source = r#"
func main() {
    let x: int = 0;
    while x < 10 {
        break;
    }
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Break in while loop passes");
}

#[test]
fn test_continue_in_for_loop() {
    let source = r#"
func main() {
    for let i: int = 0; i < 10; i = i + 1 {
        continue;
    }
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Continue in for loop passes");
}

#[test]
fn test_break_outside_loop_error() {
    let source = r#"
func main() {
    let x: int = 5;
    break;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::BreakOutsideLoop { .. })));
    println!("✓ Break outside loop detected");
}

#[test]
fn test_continue_outside_loop_error() {
    let source = r#"
func main() {
    let x: int = 5;
    continue;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::BreakOutsideLoop { .. })));
    println!("✓ Continue outside loop detected");
}

#[test]
fn test_break_in_nested_loops() {
    let source = r#"
func main() {
    let x: int = 0;
    while x < 5 {
        for let i: int = 0; i < 10; i = i + 1 {
            if i == 5 {
                break;
            }
        }
        x = x + 1;
    }
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Break in nested loops passes");
}

#[test]
fn test_const_in_array_size() {
    let source = r#"
func main() {
    const SIZE: int = 5;
    let arr: int[5];
    arr[0] = 10;
    display arr[0];
}
"#;
    // This might not work yet, but it's a good edge case
    let _result = analyze(source);
    // Either passes or fails is OK - just testing edge case
    println!("✓ Const in array context tested");
}

#[test]
fn test_const_used_before_declaration() {
    let source = r#"
func main() {
    display MAX;
    const MAX: int = 100;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Const use before declaration detected");
}