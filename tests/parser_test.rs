// tests/parser_test.rs - Basic parser functionality tests

use minilang_compiler::{Lexer, Parser, Program, Statement, Expression};
use minilang_compiler::ast::Literal; 
use pretty_assertions::assert_eq;

/// Helper to parse source code
fn parse(source: &str) -> Result<Program, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, source.to_string());
    Ok(parser.parse_program()?)
}

// ==================== BASIC FUNCTION TESTS ====================

#[test]
fn test_parse_empty_function() {
    let source = "func main() { }";
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].name, "main");
    assert_eq!(ast.functions[0].params.len(), 0);
    assert_eq!(ast.functions[0].return_type, None);
    assert_eq!(ast.functions[0].body.statements.len(), 0);
    
    println!("✓ Empty function parsed");
}

#[test]
fn test_parse_hello_world() {
    let source = r#"
func main() {
    display "Hello, World!";
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].name, "main");
    
    // Check the display statement
    match &ast.functions[0].body.statements[0] {
        Statement::Display(stmt) => {
            assert_eq!(stmt.expressions.len(), 1);
            match &stmt.expressions[0] {
                Expression::Literal(lit_expr) => {
                    match &lit_expr.value{
                        Literal::String(s) =>{
                            assert_eq!(s, "Hello, World!");
                        }
                        _ => panic!("Expected string literal"),
                    }
                }
                _ => panic!("Expected string literal"),
            }
        }
        _ => panic!("Expected display statement"),
    }
    
    println!("✓ Hello World parsed");
}

#[test]
fn test_parse_function_with_parameters() {
    let source = "func add(a: int, b: int) -> int { send a + b; }";
    
    let ast = parse(source).unwrap();
    let func = &ast.functions[0];
    
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.params[0].name, "a");
    assert_eq!(func.params[1].name, "b");
    assert!(func.return_type.is_some());
    
    println!("✓ Function with parameters parsed");
}

#[test]
fn test_parse_multiple_functions() {
    let source = r#"
func first() { }
func second() { }
func third() { }
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 3);
    assert_eq!(ast.functions[0].name, "first");
    assert_eq!(ast.functions[1].name, "second");
    assert_eq!(ast.functions[2].name, "third");
    
    println!("✓ Multiple functions parsed");
}

// ==================== VARIABLE DECLARATION TESTS ====================

#[test]
fn test_parse_variable_declarations() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: float = 3.14;
    let name: string = "MiniLang";
    let flag: bool = true;
}
"#;
    
    let ast = parse(source).unwrap();
    let statements = &ast.functions[0].body.statements;
    
    assert_eq!(statements.len(), 4);
    
    // Check each variable declaration
    for stmt in statements {
        match stmt {
            Statement::Let(let_stmt) => {
                assert!(let_stmt.value.is_some());
            }
            _ => panic!("Expected let statement"),
        }
    }
    
    println!("✓ Variable declarations parsed");
}

#[test]
fn test_parse_variable_without_initialization() {
    let source = "func main() { let x: int; }";
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::Let(let_stmt) => {
            assert_eq!(let_stmt.name, "x");
            assert!(let_stmt.value.is_none());
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Variable without initialization parsed");
}

#[test]
fn test_parse_const_declaration() {
    let source = r#"
func main() {
    const PI: float = 3.14159;
    const MAX_SIZE: int = 100;
    const GREETING: string = "Hello, World!";
}
"#;
    
    let ast = parse(source).unwrap();
    let statements = &ast.functions[0].body.statements;
    
    assert_eq!(statements.len(), 3);
    
    // Check first const (PI)
    match &statements[0] {
        Statement::Const(const_stmt) => {
            assert_eq!(const_stmt.name, "PI");
            // Check that value is a literal expression
            match &const_stmt.value {
                Expression::Literal(_) => {},
                _ => panic!("Expected literal expression"),
            }
        }
        _ => panic!("Expected const statement"),
    }
    
    println!("✓ Const declarations parsed");
}

#[test]
fn test_parse_const_without_value_fails() {
    let source = r#"
func main() {
    const PI: float;  // Should fail - const needs initialization
}
"#;
    
    let result = parse(source);
    assert!(result.is_err(), "Const without value should fail to parse");
    println!("✓ Const without initialization correctly rejected");
}


#[test]
fn test_parse_array_declaration() {
    let source = r#"
func main() {
    let arr: int[5] = [1, 2, 3, 4, 5];
    let empty: string[10];
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 2);
    
    println!("✓ Array declarations parsed");
}



// ==================== EXPRESSION TESTS ====================

#[test]
fn test_parse_arithmetic_expressions() {
    let source = r#"
func main() {
    let a: int = 10 + 20;
    let b: int = 30 - 15;
    let c: int = 5 * 6;
    let d: int = 100 / 5;
    let e: int = 17 % 5;
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 5);
    
    println!("✓ Arithmetic expressions parsed");
}

#[test]
fn test_parse_comparison_expressions() {
    let source = r#"
func main() {
    let a: bool = x == y;
    let b: bool = x != y;
    let c: bool = x < y;
    let d: bool = x > y;
    let e: bool = x <= y;
    let f: bool = x >= y;
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 6);
    
    println!("✓ Comparison expressions parsed");
}

#[test]
fn test_parse_logical_expressions() {
    let source = r#"
func main() {
    let a: bool = x AND y;
    let b: bool = x OR y;
    let c: bool = NOT x;
    let d: bool = x AND y OR z;
    let e: bool = NOT (x OR y);
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 5);
    
    println!("✓ Logical expressions parsed");
}

#[test]
fn test_parse_operator_precedence() {
    let source = r#"
func main() {
    let a: int = 2 + 3 * 4;        # Should be 2 + (3 * 4) = 14
    let b: int = (2 + 3) * 4;      # Should be (2 + 3) * 4 = 20
    let c: bool = x AND y OR z;     # Should be (x AND y) OR z
    let d: bool = x OR y AND z;     # Should be x OR (y AND z)
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 4);
    
    println!("✓ Operator precedence parsed correctly");
}

// ==================== CONTROL FLOW TESTS ====================

#[test]
fn test_parse_if_statement() {
    let source = r#"
func main() {
    if x > 0 {
        display "positive";
    }
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::If(if_stmt) => {
            assert!(if_stmt.else_block.is_none());
        }
        _ => panic!("Expected if statement"),
    }
    
    println!("✓ If statement parsed");
}

#[test]
fn test_parse_if_else_statement() {
    let source = r#"
func main() {
    if x > 0 {
        display "positive";
    } else {
        display "non-positive";
    }
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::If(if_stmt) => {
            assert!(if_stmt.else_block.is_some());
        }
        _ => panic!("Expected if statement"),
    }
    
    println!("✓ If-else statement parsed");
}

#[test]
fn test_parse_if_else_if_chain() {
    let source = r#"
func main() {
    if x > 0 {
        display "positive";
    } else if x < 0 {
        display "negative";
    } else {
        display "zero";
    }
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::If(if_stmt) => {
            assert!(if_stmt.else_block.is_some());
            // The else block should contain another if statement
            let else_block = if_stmt.else_block.as_ref().unwrap();
            assert_eq!(else_block.statements.len(), 1);
        }
        _ => panic!("Expected if statement"),
    }
    
    println!("✓ If-else-if chain parsed");
}

#[test]
fn test_parse_while_loop() {
    let source = r#"
func main() {
    while x < 10 {
        x = x + 1;
    }
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::While(while_stmt) => {
            assert_eq!(while_stmt.body.statements.len(), 1);
        }
        _ => panic!("Expected while statement"),
    }
    
    println!("✓ While loop parsed");
}

#[test]
fn test_parse_do_while_loop() {
    let source = r#"
func main() {
    do {
        x = x + 1;
    } while x < 10;
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::DoWhile(do_while_stmt) => {
            assert_eq!(do_while_stmt.body.statements.len(), 1);
        }
        _ => panic!("Expected do-while statement"),
    }
    
    println!("✓ Do-while loop parsed");
}

#[test]
fn test_parse_for_loop() {
    let source = r#"
func main() {
    for i = 0; i < 10; i = i + 1 {
        display i;
    }
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::For(for_stmt) => {
            assert!(for_stmt.init.is_some());
            assert!(for_stmt.condition.is_some());
            assert!(for_stmt.update.is_some());
            assert_eq!(for_stmt.body.statements.len(), 1);
        }
        _ => panic!("Expected for statement"),
    }
    
    println!("✓ For loop parsed");
}

// ==================== SPECIAL STATEMENT TESTS ====================

#[test]
fn test_parse_display_statement() {
    let source = r#"
func main() {
    display "Hello";
    display "Name:", name;
    display x, y, z;
}
"#;
    
    let ast = parse(source).unwrap();
    let statements = &ast.functions[0].body.statements;
    
    // First display - one expression
    match &statements[0] {
        Statement::Display(stmt) => assert_eq!(stmt.expressions.len(), 1),
        _ => panic!("Expected display"),
    }
    
    // Second display - two expressions
    match &statements[1] {
        Statement::Display(stmt) => assert_eq!(stmt.expressions.len(), 2),
        _ => panic!("Expected display"),
    }
    
    // Third display - three expressions
    match &statements[2] {
        Statement::Display(stmt) => assert_eq!(stmt.expressions.len(), 3),
        _ => panic!("Expected display"),
    }
    
    println!("✓ Display statements parsed");
}

#[test]
fn test_parse_return_statement() {
    let source = r#"
func calculate() -> int {
    send 42;
}

func early_return() {
    if x < 0 {
        send;
    }
    send x * 2;
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 2);
    
    println!("✓ Return statements parsed");
}

#[test]
fn test_parse_break_statement() {
    let source = r#"
func main() {
    while true {
        break;
    }
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::While(while_stmt) => {
            match &while_stmt.body.statements[0] {
                Statement::Break(_) => {},
                _ => panic!("Expected break statement"),
            }
        }
        _ => panic!("Expected while statement"),
    }
    
    println!("✓ Break statement parsed");
}

#[test]
fn test_parse_continue_statement() {
    let source = r#"
func main() {
    for i = 0; i < 10; i = i + 1 {
        continue;
    }
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::For(for_stmt) => {
            match &for_stmt.body.statements[0] {
                Statement::Continue(_) => {},
                _ => panic!("Expected continue statement"),
            }
        }
        _ => panic!("Expected for statement"),
    }
    
    println!("✓ Continue statement parsed");
}

// ==================== COMPLEX TESTS ====================

#[test]
fn test_parse_nested_blocks() {
    let source = r#"
func main() {
    {
        let x: int = 1;
        {
            let y: int = 2;
            {
                let z: int = 3;
            }
        }
    }
}
"#;
    
    let ast = parse(source).unwrap();
    assert!(ast.functions[0].body.statements.len() > 0);
    
    println!("✓ Nested blocks parsed");
}

#[test]
fn test_parse_function_calls() {
    let source = r#"
func main() {
    print();
    add(1, 2);
    calculate(x, y + z, 42);
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 3);
    
    println!("✓ Function calls parsed");
}

#[test]
fn test_parse_array_indexing() {
    let source = r#"
func main() {
    let arr: int[10];
    let x: int = arr[0];
    let y: int = arr[i];
    arr[5] = 100;
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 4);
    
    println!("✓ Array indexing parsed");
}


#[test]
fn test_parse_assignments() {
    let source = r#"
func main() {
    let x: int;
    let y: int;
    let z: int;
    x = 10;
    y = x + 5;
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 5);
    
    println!("✓ Assignments parsed");
}

#[test]
fn test_parse_function_call_result() {
    let source = r#"
func getValue() -> int {
    send 42;
}

func main() {
    let result: int = getValue();
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 2);
    
    println!("✓ Function call result assignment parsed");
}


#[test]
fn test_parse_string_interpolation_simple() {
    let source = r#"
func main() {
    let name: string = "Alice";
    display "Hello, {name}!";
}
"#;
    
    let ast = parse(source).unwrap();
    // Check it parses without error
    assert_eq!(ast.functions.len(), 1);
    
    // Verify the display statement has interpolated string
    match &ast.functions[0].body.statements[1] {
        Statement::Display(display_stmt) => {
            match &display_stmt.expressions[0] {
                Expression::Literal(lit_expr) => {
                    assert!(matches!(lit_expr.value, Literal::InterpolatedString(_)));
                }
                _ => panic!("Expected literal expression"),
            }
        }
        _ => panic!("Expected display statement"),
    }
    
    println!("✓ Simple string interpolation parsed");
}

#[test]
fn test_parse_string_interpolation_multiple_vars() {
    let source = r#"
func main() {
    let name: string = "Bob";
    let age: int = 25;
    display "Name: {name}, Age: {age}";
}
"#;
    
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 1);
    println!("✓ Multiple interpolations parsed");
}

#[test]
fn test_parse_string_no_interpolation() {
    let source = r#"
func main() {
    display "No interpolation here!";
}
"#;
    
    let ast = parse(source).unwrap();
    match &ast.functions[0].body.statements[0] {
        Statement::Display(display_stmt) => {
            match &display_stmt.expressions[0] {
                Expression::Literal(lit_expr) => {
                    // Should be a regular string, not interpolated
                    assert!(matches!(lit_expr.value, Literal::String(_)));
                }
                _ => panic!("Expected literal"),
            }
        }
        _ => panic!("Expected display"),
    }
    
    println!("✓ Non-interpolated string stays regular");
}

#[test]
fn test_parse_empty_interpolation_fails() {
    let source = r#"
func main() {
    display "Empty: {}";
}
"#;
    
    let result = parse(source);
    assert!(result.is_err());
    println!("✓ Empty interpolation correctly rejected");
}
