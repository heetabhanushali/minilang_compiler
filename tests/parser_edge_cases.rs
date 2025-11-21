// tests/parser_edge_cases.rs - Edge cases and corner scenarios

use minilang_compiler::{Lexer, Parser, Program};

fn parse(source: &str) -> Result<Program, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, source.to_string());
    Ok(parser.parse_program()?)
}

// ==================== EDGE CASES ====================

#[test]
fn test_empty_program() {
    let source = "";
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 0);
    println!("✓ Empty program parsed");
}

#[test]
fn test_function_with_only_comments() {
    let source = r#"
# Comment before
func main() {
    # Comment inside
    ## Multi-line
       comment ##
}
# Comment after
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].body.statements.len(), 0);
    println!("✓ Function with only comments parsed");
}

#[test]
fn test_deeply_nested_expressions() {
    let source = r#"
func main() {
    let x: int = ((((1 + 2) * 3) - 4) / 5) % 6;
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Deeply nested expressions parsed");
}

#[test]
fn test_deeply_nested_blocks() {
    let source = r#"
func main() {
    {
        {
            {
                {
                    let x: int = 1;
                }
            }
        }
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Deeply nested blocks parsed");
}

#[test]
fn test_multiple_array_dimensions_read() {
    let source = r#"
func main() {
    let matrix: int[10];
    let x: int = matrix[0];
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Array access parsed");
}

#[test]
fn test_function_call_as_argument() {
    let source = r#"
func helper(x: int) -> int {
    send x * 2;
}

func main() {
    let result: int = helper(helper(5));
}
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions.len(), 2);
    println!("✓ Nested function calls parsed");
}

#[test]
fn test_expression_in_array_index() {
    let source = r#"
func main() {
    let arr: int[10];
    let i: int = 5;
    let x: int = arr[i + 1];
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Expression in array index parsed");
}

#[test]
fn test_boolean_in_all_contexts() {
    let source = r#"
func main() {
    let a: bool = true;
    let b: bool = false;
    let c: bool = true AND false;
    let d: bool = NOT true;
}
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 4);
    println!("✓ Boolean literals and operations parsed");
}

#[test]
fn test_mixed_types_in_display() {
    let source = r#"
func main() {
    display "Number:", 42, "Float:", 3.14, "Bool:", true;
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Mixed types in display parsed");
}

#[test]
fn test_void_function_early_return() {
    let source = r#"
func check(x: int) {
    if x < 0 {
        send;
    }
    display "Valid";
}
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].name, "check");
    println!("✓ Void function with early return parsed");
}

#[test]
fn test_multiple_returns_in_function() {
    let source = r#"
func classify(x: int) -> string {
    if x < 0 {
        send "negative";
    } else if x == 0 {
        send "zero";
    } else {
        send "positive";
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Multiple return paths parsed");
}

#[test]
fn test_for_loop_no_init() {
    let source = r#"
func main() {
    for ; i < 10; i = i + 1 {
        display i;
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ For loop without init parsed");
}

#[test]
fn test_for_loop_no_condition() {
    let source = r#"
func main() {
    for i = 0; ; i = i + 1 {
        if i > 10 {
            send;
        }
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ For loop without condition parsed");
}

#[test]
fn test_for_loop_no_update() {
    let source = r#"
func main() {
    for i = 0; i < 10; {
        display i;
        i = i + 1;
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ For loop without update parsed");
}

#[test]
fn test_infinite_loops() {
    let source = r#"
func main() {
    while true {
        display "infinite";
    }
    
    for ; ; {
        display "also infinite";
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Infinite loops parsed");
}

#[test]
fn test_chained_comparisons() {
    let source = r#"
func main() {
    let a: bool = x < y;
    let b: bool = y < z;
    let c: bool = (x < y) AND (y < z);
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Chained comparisons parsed");
}

#[test]
fn test_negation_and_not() {
    let source = r#"
func main() {
    let x: int = -42;
    let y: int = -(-10);
    let b: bool = NOT true;
    let c: bool = NOT NOT false;
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Negation and NOT parsed");
}

#[test]
fn test_empty_array_literal() {
    // This might fail - arrays need size
    let source = r#"
func main() {
    let arr: int[0];
}
"#;
    let _result = parse(source);
    // Either succeeds or fails is OK - just testing edge case
    println!("✓ Empty array declaration tested");
}

#[test]
fn test_large_array() {
    let source = r#"
func main() {
    let big: int[1000];
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Large array declaration parsed");
}

#[test]
fn test_string_with_escapes() {
    let source = r#"
func main() {
    let s: string = "Line 1\nLine 2\tTabbed";
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ String with escape sequences parsed");
}

#[test]
fn test_parenthesized_expressions() {
    let source = r#"
func main() {
    let x: int = (1 + 2) * (3 + 4);
    let y: bool = (x > 5) AND (x < 10);
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Parenthesized expressions parsed");
}

#[test]
fn test_function_with_many_parameters() {
    let source = r#"
func many(a: int, b: int, c: int, d: int, e: int) -> int {
    send a + b + c + d + e;
}
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].params.len(), 5);
    println!("✓ Function with many parameters parsed");
}

#[test]
fn test_function_call_with_many_args() {
    let source = r#"
func main() {
    let result: int = calculate(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Function call with many arguments parsed");
}

#[test]
fn test_all_types_in_one_function() {
    let source = r#"
func allTypes(
    i: int,
    f: float,
    s: string,
    b: bool,
    arr: int[10]
) -> bool {
    send true;
}
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].params.len(), 5);
    println!("✓ All parameter types parsed");
}

#[test]
fn test_consecutive_statements() {
    let source = r#"
func main() {
    let a: int = 1;
    let b: int = 2;
    let c: int = 3;
    display a;
    display b;
    display c;
    a = 10;
    b = 20;
    c = 30;
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions[0].body.statements.len() >= 9);
    println!("✓ Many consecutive statements parsed");
}

#[test]
fn test_assignment_with_complex_expression() {
    let source = r#"
func main() {
    let x: int = 10;
    x = x * 2 + 5 - 3;
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Assignment with complex expression parsed");
}

#[test]
fn test_display_with_no_arguments() {
    // This should fail - display needs at least one argument
    let source = r#"
func main() {
    display;
}
"#;
    let result = parse(source);
    assert!(result.is_err(), "Display with no args should fail");
    println!("✓ Display with no arguments correctly rejected");
}

#[test]
fn test_deeply_nested_if_else() {
    let source = r#"
func main() {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                display "all positive";
            }
        }
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Deeply nested if statements parsed");
}

#[test]
fn test_else_if_chain_long() {
    let source = r#"
func main() {
    if x == 1 {
        display "one";
    } else if x == 2 {
        display "two";
    } else if x == 3 {
        display "three";
    } else if x == 4 {
        display "four";
    } else {
        display "other";
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Long else-if chain parsed");
}

#[test]
fn test_loop_in_loop_in_loop() {
    let source = r#"
func main() {
    while x > 0 {
        while y > 0 {
            for i = 0; i < 10; i = i + 1 {
                display i;
            }
        }
    }
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Triply nested loops parsed");
}

#[test]
fn test_operators_all_precedence_levels() {
    let source = r#"
func main() {
    let result: bool = a OR b AND c == d != e < f > g <= h >= i + j - k * l / m % n;
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ All precedence levels in one expression parsed");
}


#[test]
fn test_multiple_consts() {
    let source = r#"
func main() {
    const A: int = 1;
    const B: int = 2;
    const C: int = 3;
    const D: int = 4;
    const E: int = 5;
    display A, B, C, D, E;
}
"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.functions[0].body.statements.len(), 6); // 5 consts + 1 display
    println!("✓ Multiple const declarations parsed");
}

#[test]
fn test_const_with_expression() {
    let source = r#"
func main() {
    const SUM: int = 10 + 20 + 30;
    const PRODUCT: int = 5 * 6;
    display SUM, PRODUCT;
}
"#;
    let ast = parse(source).unwrap();
    assert!(ast.functions.len() > 0);
    println!("✓ Const with expression parsed");
}