// tests/semantic_scope_tests.rs - Scope-related semantic tests

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

// ==================== SCOPE TESTS ====================

#[test]
fn test_nested_scope_visibility() {
    let source = r#"
func main() {
    let x: int = 10;
    {
        display x;  # x is visible in inner scope
        let y: int = 20;
        display y;
    }
    display x;  # x still visible
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Nested scope visibility works correctly");
}

#[test]
fn test_inner_scope_not_visible_outside() {
    let source = r#"
func main() {
    {
        let x: int = 10;
    }
    display x;  # x is out of scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Inner scope variable not visible outside detected");
}

#[test]
fn test_if_scope() {
    let source = r#"
func main() {
    if true {
        let x: int = 10;
    }
    display x;  # x is out of scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ If block scope isolation detected");
}

#[test]
fn test_else_scope() {
    let source = r#"
func main() {
    if false {
        let x: int = 10;
    } else {
        let y: int = 20;
    }
    display y;  # y is out of scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Else block scope isolation detected");
}

#[test]
fn test_while_scope() {
    let source = r#"
func main() {
    while true {
        let x: int = 10;
    }
    display x;  # x is out of scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ While loop scope isolation detected");
}

#[test]
fn test_for_loop_init_scope() {
    let source = r#"
func main() {
    for let i: int = 0; i < 10; i = i + 1 {
        display i;
    }
    display i;  # i is out of scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ For loop variable scope isolation detected");
}

#[test]
fn test_for_loop_body_scope() {
    let source = r#"
func main() {
    for i = 0; i < 10; i = i + 1 {
        let x: int = i * 2;
    }
    display x;  # x is out of scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ For loop body scope isolation detected");
}

#[test]
fn test_deeply_nested_scopes() {
    let source = r#"
func main() {
    let a: int = 1;
    {
        let b: int = 2;
        {
            let c: int = 3;
            {
                display a;  # OK
                display b;  # OK
                display c;  # OK
            }
            display a;  # OK
            display b;  # OK
        }
        display a;  # OK
    }
    display a;  # OK
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Deeply nested scopes work correctly");
}

#[test]
fn test_shadowing_in_nested_scope() {
    let source = r#"
func main() {
    let x: int = 10;
    {
        let x: string = "hello";  # OK - different scope
        display x;  # Uses inner x
    }
    display x;  # Uses outer x
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Variable shadowing in nested scope allowed");
}

#[test]
fn test_function_scope_isolation() {
    let source = r#"
func f1() {
    let x: int = 10;
}

func f2() {
    display x;  # x from f1 is not visible
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Function scope isolation detected");
}

#[test]
fn test_parameter_scope() {
    let source = r#"
func test(x: int, y: int) {
    display x;  # OK
    display y;  # OK
    {
        display x;  # OK - parameters visible in inner scope
        display y;  # OK
    }
}

func main() {
    display x;  # x from test is not visible
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Parameter scope correctly limited to function");
}

#[test]
fn test_do_while_scope() {
    let source = r#"
func main() {
    do {
        let x: int = 10;
    } while false;
    display x;  # x is out of scope
}
"#;
    let errors = expect_semantic_error(source);
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { .. })));
    println!("✓ Do-while loop scope isolation detected");
}


#[test]
fn test_const_scope() {
    let source = r#"
func main() {
    const X: int = 10;
    display X;
    let y: int = X;
    display y;
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Const scope visibility works");
}

#[test]
fn test_const_shadowing() {
    let source = r#"
func outer() {
    const X: int = 10;
    display X;
}

func inner() {
    const X: int = 20;
    display X;
}

func main() {
    outer();
    inner();
}
"#;
    assert!(analyze(source).is_ok());
    println!("✓ Const in different functions allowed");
}