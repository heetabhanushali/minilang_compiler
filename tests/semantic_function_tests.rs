// tests/semantic_function_tests.rs - Function-related semantic tests

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

// ==================== FUNCTION DECLARATION TESTS ====================

#[test]
fn test_function_duplicate_definition() {
    let source = r#"
func add(x: int, y: int) -> int {
    send x + y;
}

func add(a: int, b: int) -> int {  # Duplicate function
    send a + b;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::DuplicateDefinition { .. })));
    println!("✓ Duplicate function definition detected");
}

#[test]
fn test_function_parameter_duplicate() {
    let source = r#"
func add(x: int, x: int) -> int {  # Duplicate parameter name
    send x;
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::DuplicateDefinition { .. })));
    println!("✓ Duplicate parameter name detected");
}

// ==================== FUNCTION CALL TESTS ====================

#[test]
fn test_undefined_function_call() {
    let source = r#"
func main() {
    let result: int = add(1, 2);  # add is not defined
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedFunction { .. })));
    println!("✓ Undefined function call detected");
}

#[test]
fn test_function_argument_count_mismatch() {
    let source = r#"
func add(x: int, y: int) -> int {
    send x + y;
}

func main() {
    let result: int = add(1, 2, 3);  # Too many arguments
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::ArgumentCountMismatch { .. })));
    println!("✓ Argument count mismatch detected");
}

#[test]
fn test_function_argument_type_mismatch() {
    let source = r#"
func add(x: int, y: int) -> int {
    send x + y;
}

func main() {
    let result: int = add("hello", 2);  # Wrong argument type
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Argument type mismatch detected");
}

#[test]
fn test_function_return_type_usage() {
    let source = r#"
func getValue() -> int {
    send 42;
}

func main() {
    let x: string = getValue();  # Return type mismatch
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Return type mismatch in assignment detected");
}

#[test]
fn test_void_function_in_expression() {
    let source = r#"
func printValue(x: int) {
    display x;
}

func main() {
    let result: int = printValue(5);  # Void function used in expression
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Void function in expression detected");
}

// ==================== RETURN STATEMENT TESTS ====================

#[test]
fn test_missing_return_statement() {
    let source = r#"
func getValue() -> int {
    let x: int = 42;
    # Missing return statement
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::MissingReturn { .. })));
    println!("✓ Missing return statement detected");
}

#[test]
fn test_return_type_mismatch() {
    let source = r#"
func getValue() -> int {
    send "hello";  # Should return int, not string
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Return type mismatch detected");
}

#[test]
fn test_void_function_returning_value() {
    let source = r#"
func printValue(x: int) {
    display x;
    send x;  # Void function should not return value
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Void function returning value detected");
}

#[test]
fn test_non_void_function_returning_void() {
    let source = r#"
func getValue() -> int {
    send;  # Should return int, not void
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Non-void function returning void detected");
}

#[test]
fn test_conditional_return_paths() {
    let source = r#"
func getValue(x: int) -> int {
    if x > 0 {
        send x;
    }
    # Missing return for else case
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::MissingReturn { .. })));
    println!("✓ Missing return on all paths detected");
}

#[test]
fn test_valid_conditional_returns() {
    let source = r#"
func getValue(x: int) -> int {
    if x > 0 {
        send x;
    } else {
        send -x;
    }
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Valid conditional returns pass");
}

// ==================== RECURSION TESTS ====================

#[test]
fn test_recursive_function_valid() {
    let source = r#"
func factorial(n: int) -> int {
    if n <= 1 {
        send 1;
    }
    send n * factorial(n - 1);
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Valid recursive function passes");
}

#[test]
fn test_mutual_recursion() {
    let source = r#"
func isEven(n: int) -> bool {
    if n == 0 {
        send true;
    }
    send isOdd(n - 1);
}

func isOdd(n: int) -> bool {
    if n == 0 {
        send false;
    }
    send isEven(n - 1);
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Mutual recursion passes");
}

// ==================== PARAMETER TESTS ====================

#[test]
fn test_parameter_scope() {
    let source = r#"
func test(x: int) {
    let x: int = 10;  # Shadows parameter - should error
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::DuplicateDefinition { .. })));
    println!("✓ Parameter shadowing in same scope detected");
}

#[test]
fn test_parameter_usage() {
    let source = r#"
func add(x: int, y: int) -> int {
    send x + y + z;  # z is undefined
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Undefined variable in function detected");
}

#[test]
fn test_array_parameter() {
    let source = r#"
func sum(arr: int[5]) -> int {
    let total: int = 0;
    let i: int;
    for i = 0; i < 5; i = i + 1 {
        total = total + arr[i];
    }
    send total;
}

func main() {
    let nums: int[5] = [1, 2, 3, 4, 5];
    let result: int = sum(nums);
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Array parameter passing passes");
}

#[test]
fn test_array_parameter_size_mismatch() {
    let source = r#"
func process(arr: int[5]) {
    display "Processing";
}

func main() {
    let nums: int[10];
    process(nums);  # Array size mismatch
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    println!("✓ Array parameter size mismatch detected");
}


#[test]
fn test_const_in_function() {
    let source = r#"
func calculate() -> int {
    const MULTIPLIER: int = 10;
    let x: int = 5;
    send x * MULTIPLIER;
}

func main() {
    let result: int = calculate();
    display result;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Const in function works");
}

#[test]
fn test_const_as_function_argument() {
    let source = r#"
func double(x: int) -> int {
    send x * 2;
}

func main() {
    const VALUE: int = 21;
    let result: int = double(VALUE);
    display result;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Const as function argument works");
}