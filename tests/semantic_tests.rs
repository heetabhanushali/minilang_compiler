// tests/semantic_test.rs - Basic semantic analysis tests

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

// ==================== VARIABLE TESTS ====================

#[test]
fn test_valid_variable_declaration() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: float = 3.14;
    let s: string = "hello";
    let b: bool = true;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Valid variable declarations pass");
}

#[test]
fn test_undefined_variable() {
    let source = r#"
func main() {
    display x;  # x is not defined
}
"#;
    let errors = expect_semantic_error(source);
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        SemanticError::UndefinedVariable { name, .. } => {
            assert_eq!(name, "x");
        }
        _ => panic!("Expected UndefinedVariable error"),
    }
    println!("✓ Undefined variable detected");
}

#[test]
fn test_variable_redefinition() {
    let source = r#"
func main() {
    let x: int = 10;
    let x: int = 20;  # Redefinition in same scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::DuplicateDefinition { .. })));
    println!("✓ Variable redefinition detected");
}

#[test]
fn test_variable_shadowing_different_scopes() {
    let source = r#"
func main() {
    let x: int = 10;
    {
        let x: int = 20;  # OK - different scope
        display x;
    }
    display x;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Variable shadowing in different scopes allowed");
}

#[test]
fn test_variable_use_before_declaration() {
    let source = r#"
func main() {
    display y;
    let y: int = 10;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Use before declaration detected");
}

// ==================== TYPE CHECKING TESTS ====================

#[test]
fn test_type_mismatch_assignment() {
    let source = r#"
func main() {
    let x: int = "hello";  # String assigned to int
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Type mismatch in assignment detected");
}

#[test]
fn test_type_mismatch_different_types() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: string = x;  # Cannot assign int to string
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Type mismatch between different types detected");
}

#[test]
fn test_arithmetic_type_checking() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = 20;
    let z: int = x + y;  # OK
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Valid arithmetic operations pass");
}

#[test]
fn test_arithmetic_type_mismatch() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: string = "hello";
    let z: int = x + y;  # Cannot add int and string
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Arithmetic type mismatch detected");
}

#[test]
fn test_comparison_operators() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = 20;
    let b: bool = x < y;  # OK
    let c: bool = x == y;  # OK
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Valid comparison operations pass");
}

#[test]
fn test_comparison_type_mismatch() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: string = "hello";
    let b: bool = x < y;  # Cannot compare int and string
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Comparison type mismatch detected");
}

// ==================== LOGICAL OPERATOR TESTS ====================

#[test]
fn test_logical_operators_valid() {
    let source = r#"
func main() {
    let a: bool = true;
    let b: bool = false;
    let c: bool = a AND b;
    let d: bool = a OR b;
    let e: bool = NOT a;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Valid logical operations pass");
}

#[test]
fn test_logical_and_type_error() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: bool = true;
    let z: bool = x AND y;  # Cannot AND int and bool
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ AND operator type error detected");
}

#[test]
fn test_logical_or_type_error() {
    let source = r#"
func main() {
    let x: string = "hello";
    let y: bool = true;
    let z: bool = x OR y;  # Cannot OR string and bool
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ OR operator type error detected");
}

#[test]
fn test_logical_not_type_error() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: bool = NOT x;  # Cannot NOT an integer
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ NOT operator type error detected");
}

// ==================== CONTROL FLOW TESTS ====================

#[test]
fn test_if_condition_must_be_bool() {
    let source = r#"
func main() {
    let x: int = 42;
    if x {  # Condition must be boolean
        display "yes";
    }
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Non-boolean if condition detected");
}

#[test]
fn test_while_condition_must_be_bool() {
    let source = r#"
func main() {
    let x: string = "hello";
    while x {  # Condition must be boolean
        display x;
    }
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Non-boolean while condition detected");
}

#[test]
fn test_for_loop_condition_type() {
    let source = r#"
func main() {
    for i = 0; "not bool"; i = i + 1 {  # Condition must be boolean
        display i;
    }
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Non-boolean for condition detected");
}

// ==================== ARRAY TESTS ====================

#[test]
fn test_array_declaration_valid() {
    let source = r#"
func main() {
    let arr: int[5] = [1, 2, 3, 4, 5];
    let empty: float[10];
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Valid array declarations pass");
}

#[test]
fn test_array_index_must_be_int() {
    let source = r#"
func main() {
    let arr: int[5];
    let idx: float = 2.5;
    let val: int = arr[idx];  # Index must be integer
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Non-integer array index detected");
}

#[test]
fn test_array_element_type() {
    let source = r#"
func main() {
    let arr: int[5];
    let val: string = arr[0];  # Element is int, not string
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Array element type mismatch detected");
}

#[test]
fn test_indexing_non_array() {
    let source = r#"
func main() {
    let x: int = 42;
    let val: int = x[0];  # Cannot index non-array
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Indexing non-array detected");
}

// ==================== CONST TESTS ====================

#[test]
fn test_const_reassignment_error() {
    let source = r#"
func main() {
    const PI: float = 3.14159;
    PI = 3.0;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Const reassignment detected");
}

#[test]
fn test_const_redefinition_error() {
    let source = r#"
func main() {
    const MAX: int = 100;
    const MAX: int = 200;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::DuplicateDefinition { .. })));
    println!("✓ Const redefinition detected");
}

#[test]
fn test_const_type_mismatch() {
    let source = r#"
func main() {
    const PI: float = "not a float";
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Const type mismatch detected");
}

#[test]
fn test_interpolation_with_undefined_var() {
    let source = r#"
func main() {
    display "Hello, {undefined}!";
}
"#;
    
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Undefined variable in interpolation detected");
}

#[test]
fn test_interpolation_type_checking() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: float = 3.14;
    let b: bool = true;
    display "Int: {x}, Float: {y}, Bool: {b}";
}
"#;
    
    assert!(analyze(source).is_ok());
    println!("✓ Interpolation with different types passes");
}