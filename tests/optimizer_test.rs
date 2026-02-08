// tests/optimizer_test.rs - Basic optimizer functionality tests

use minilang_compiler::{Lexer, OptimizationStats, Optimizer, Parser, Program, UnaryOp};
use minilang_compiler::ast::{Statement, Expression, Literal};
use pretty_assertions::assert_eq;

/// Helper to parse and optimize source code
fn optimize(source: &str, level: u8) -> (Program, OptimizationStats) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens, source.to_string());
    let mut program = parser.parse_program().expect("Parser failed");
    
    let mut optimizer = Optimizer::new(level);
    let stats = optimizer.optimize(&mut program);
    
    (program, stats)
}

/// Helper to get first statement from main function
fn get_first_statement(program: &Program) -> &Statement {
    &program.functions[0].body.statements[0]
}

// ==================== CONSTANT FOLDING TESTS ====================

#[test]
fn test_fold_integer_addition() {
    let source = r#"
func main() {
    let x: int = 10 + 20;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    // Check the value was folded to 30
    match get_first_statement(&program) {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(30));
                }
                _ => panic!("Expected literal after folding"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Integer addition folded: 10 + 20 = 30");
}

#[test]
fn test_fold_integer_subtraction() {
    let source = r#"
func main() {
    let x: int = 100 - 25;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    match get_first_statement(&program) {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(75));
                }
                _ => panic!("Expected literal after folding"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Integer subtraction folded: 100 - 25 = 75");
}

#[test]
fn test_fold_integer_multiplication() {
    let source = r#"
func main() {
    let x: int = 5 * 8;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    match get_first_statement(&program) {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(40));
                }
                _ => panic!("Expected literal after folding"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Integer multiplication folded: 5 * 8 = 40");
}

#[test]
fn test_fold_integer_division() {
    let source = r#"
func main() {
    let x: int = 50 / 5;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    match get_first_statement(&program) {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(10));
                }
                _ => panic!("Expected literal after folding"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Integer division folded: 50 / 5 = 10");
}

#[test]
fn test_fold_integer_modulo() {
    let source = r#"
func main() {
    let x: int = 27 % 5;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    match get_first_statement(&program) {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(2));
                }
                _ => panic!("Expected literal after folding"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Integer modulo folded: 27 % 5 = 2");
}

#[test]
fn test_fold_float_operations() {
    let source = r#"
func main() {
    let a: float = 3.5 + 2.5;
    let b: float = 10.0 - 3.5;
    let c: float = 2.5 * 4.0;
    let d: float = 9.0 / 3.0;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 4);
    
    println!("✓ Float operations folded");
}

#[test]
fn test_fold_boolean_and() {
    let source = r#"
func main() {
    let a: bool = true AND true;
    let b: bool = true AND false;
    let c: bool = false AND false;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 3);
    
    println!("✓ Boolean AND operations folded");
}

#[test]
fn test_fold_boolean_or() {
    let source = r#"
func main() {
    let a: bool = true OR false;
    let b: bool = false OR false;
    let c: bool = true OR true;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 3);
    
    println!("✓ Boolean OR operations folded");
}

#[test]
fn test_fold_not_operator() {
    let source = r#"
func main() {
    let a: bool = NOT true;
    let b: bool = NOT false;
    let c: bool = NOT (NOT true);
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // NOT operations should fold
    assert!(stats.constants_folded >= 3);
    
    println!("✓ NOT operations folded");
}

#[test]
fn test_fold_comparison_operators() {
    let source = r#"
func main() {
    let a: bool = 10 > 5;
    let b: bool = 5 < 10;
    let c: bool = 10 == 10;
    let d: bool = 5 != 10;
    let e: bool = 10 >= 10;
    let f: bool = 5 <= 10;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 6);
    
    println!("✓ Comparison operations folded");
}

#[test]
fn test_fold_nested_expressions() {
    let source = r#"
func main() {
    let x: int = (10 + 20) * (5 - 3);
}
"#;
    
    let (program, stats) = optimize(source, 1);
    // Should fold: 10+20=30, 5-3=2, then 30*2=60
    assert!(stats.constants_folded >= 3);
    
    match get_first_statement(&program) {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(60));
                }
                _ => panic!("Expected literal after folding"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Nested expression folded: (10 + 20) * (5 - 3) = 60");
}

#[test]
fn test_fold_complex_nested() {
    let source = r#"
func main() {
    let x: int = ((15 + 5) * 3) - (8 * 5);
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.constants_folded >= 4);
    
    match get_first_statement(&program) {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(20)); // (20 * 3) - 40 = 20
                }
                _ => panic!("Expected literal after folding"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Complex nested expression folded");
}

#[test]
fn test_fold_negate_operator() {
    let source = r#"
func main() {
    let a: int = -42;
    let b: int = -(10 + 5);
    let c: float = -3.14;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.constants_folded >= 1); // At least the (10 + 5) should fold
    
    println!("✓ Negation operations handled");
}

#[test]
fn test_no_fold_with_variables() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = x + 20;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Should NOT fold x + 20 because x is a variable
    assert_eq!(stats.constants_folded, 0);
    
    println!("✓ Expressions with variables not folded");
}

#[test]
fn test_partial_fold() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = x + (20 + 30);
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Should fold (20 + 30) to 50, but not x + 50
    assert_eq!(stats.constants_folded, 1);
    
    println!("✓ Partial folding: only constant subexpressions folded");
}

#[test]
fn test_fold_in_if_condition() {
    let source = r#"
func main() {
    if 10 > 5 {
        display "yes";
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    println!("✓ Constants in if conditions folded");
}

#[test]
fn test_fold_in_while_condition() {
    let source = r#"
func main() {
    while 5 < 10 {
        display "loop";
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    println!("✓ Constants in while conditions folded");
}

#[test]
fn test_fold_in_for_loop() {
    let source = r#"
func main() {
    for i = 2 + 3; i < 10 + 15; i = i + 1 {
        display i;
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.constants_folded >= 2); 
    
    println!("✓ Constants in for loop folded");
}

#[test]
fn test_fold_in_display() {
    let source = r#"
func main() {
    display "Result: ", 10 + 20;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    println!("✓ Constants in display statement folded");
}

#[test]
fn test_fold_in_function_args() {
    let source = r#"
func helper(x: int) -> int {
    send x * 2;
}

func main() {
    let result: int = helper(10 + 5);
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    println!("✓ Constants in function arguments folded");
}

#[test]
fn test_fold_in_array_index() {
    let source = r#"
func main() {
    let arr: int[10];
    let x: int = arr[2 + 3];
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 1);
    
    println!("✓ Constants in array index folded");
}

#[test]
fn test_fold_in_return() {
    let source = r#"
func calculate() -> int {
    send 10 * 5 + 30;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.constants_folded >= 2);
    
    println!("✓ Constants in return statement folded");
}

#[test]
fn test_no_optimization_level_0() {
    let source = r#"
func main() {
    let x: int = 10 + 20;
    let y: bool = true AND false;
}
"#;
    
    let (_, stats) = optimize(source, 0); // Level 0 = no optimization
    assert_eq!(stats.constants_folded, 0);
    assert_eq!(stats.dead_code_removed, 0);
    assert_eq!(stats.constants_propagated, 0);
    
    println!("✓ Level 0 optimization does nothing");
}

// ==================== DEAD CODE ELIMINATION TESTS ====================

#[test]
fn test_eliminate_if_true_dead_else() {
    let source = r#"
func main() {
    if true {
        display "kept";
    } else {
        display "dead code";
    }
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    // The else block should be removed
    match get_first_statement(&program) {
        Statement::If(if_stmt) => {
            assert!(if_stmt.else_block.is_none());
        }
        _ => panic!("Expected if statement"),
    }
    
    println!("✓ Dead else block eliminated when condition is true");
}

#[test]
fn test_eliminate_if_false_dead_then() {
    let source = r#"
func main() {
    if false {
        display "dead code";
    } else {
        display "kept";
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Dead then block eliminated when condition is false");
}

#[test]
fn test_eliminate_while_false() {
    let source = r#"
func main() {
    while false {
        display "never runs";
        let x: int = 100;
    }
    display "after loop";
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    // Should have only one statement left (the display after)
    assert_eq!(program.functions[0].body.statements.len(), 1);
    
    println!("✓ While false loop completely eliminated");
}

#[test]
fn test_no_eliminate_while_true() {
    let source = r#"
func main() {
    while true {
        display "infinite loop";
    }
}
"#;
    
    let (program, _) = optimize(source, 1);
    
    // While true should NOT be eliminated
    assert_eq!(program.functions[0].body.statements.len(), 1);
    
    println!("✓ While true loop kept (infinite loop)");
}

#[test]
fn test_eliminate_code_after_return() {
    let source = r#"
func test() -> int {
    display "before return";
    send 42;
    display "after return - dead";
    let x: int = 100;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    // Should have only 2 statements (display and return)
    assert_eq!(program.functions[0].body.statements.len(), 2);
    
    println!("✓ Code after return eliminated");
}

#[test]
fn test_eliminate_code_after_break() {
    let source = r#"
func main() {
    while true {
        display "before break";
        break;
        display "after break - dead";
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Code after break eliminated");
}

#[test]
fn test_eliminate_code_after_continue() {
    let source = r#"
func main() {
    for i = 0; i < 10; i = i + 1 {
        if i == 5 {
            continue;
            display "after continue - dead";
        }
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Code after continue eliminated");
}

// ==================== STRENGTH REDUCTION TESTS ====================

#[test]
fn test_strength_reduce_multiply_by_zero() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = x * 0;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    // Should be optimized to: let y: int = 0;
    match &program.functions[0].body.statements[1] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(0));
                }
                _ => panic!("Expected literal 0 after strength reduction"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction: x * 0 → 0");
}

#[test]
fn test_strength_reduce_multiply_by_one() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = x * 1;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    // Should be optimized to: let y: int = x;
    match &program.functions[0].body.statements[1] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Identifier(id) => {
                    assert_eq!(id.name, "x");
                }
                _ => panic!("Expected identifier x after strength reduction"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction: x * 1 → x");
}

#[test]
fn test_strength_reduce_multiply_by_negative_one() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = x * -1;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    // Should be optimized to: let y: int = -x;
    match &program.functions[0].body.statements[1] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Unary(unary) => {
                    assert!(matches!(unary.op, UnaryOp::Negate));
                }
                _ => panic!("Expected negation after strength reduction"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction: x * -1 → -x");
}

#[test]
fn test_strength_reduce_multiply_by_power_of_two() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = x * 8;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    println!("✓ Strength reduction: x * 8 marked for shift optimization");
}

#[test]
fn test_strength_reduce_divide_by_one() {
    let source = r#"
func main() {
    let x: int = 50;
    let y: int = x / 1;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    // Should be optimized to: let y: int = x;
    match &program.functions[0].body.statements[1] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Identifier(id) => {
                    assert_eq!(id.name, "x");
                }
                _ => panic!("Expected identifier x after strength reduction"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction: x / 1 → x");
}

#[test]
fn test_strength_reduce_divide_by_negative_one() {
    let source = r#"
func main() {
    let x: int = 50;
    let y: int = x / -1;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    // Should be optimized to: let y: int = -x;
    match &program.functions[0].body.statements[1] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Unary(unary) => {
                    assert!(matches!(unary.op, UnaryOp::Negate));
                }
                _ => panic!("Expected negation after strength reduction"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction: x / -1 → -x");
}

#[test]
fn test_strength_reduce_divide_by_power_of_two() {
    let source = r#"
func main() {
    let x: int = 100;
    let y: int = x / 4;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    println!("✓ Strength reduction: x / 4 marked for shift optimization");
}

#[test]
fn test_strength_reduce_add_zero() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: int = x + 0;
    let z: int = 0 + x;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.strength_reductions, 2);
    
    // Both should be optimized to just x
    println!("✓ Strength reduction: x + 0 → x and 0 + x → x");
}

#[test]
fn test_strength_reduce_subtract_zero() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: int = x - 0;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    // Should be optimized to: let y: int = x;
    match &program.functions[0].body.statements[1] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Identifier(id) => {
                    assert_eq!(id.name, "x");
                }
                _ => panic!("Expected identifier x after strength reduction"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction: x - 0 → x");
}

#[test]
fn test_strength_reduce_subtract_self() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: int = x - x;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    // Should be optimized to: let y: int = 0;
    match &program.functions[0].body.statements[1] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(0));
                }
                _ => panic!("Expected literal 0 after strength reduction"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction: x - x → 0");
}

#[test]
fn test_strength_reduce_modulo_power_of_two() {
    let source = r#"
func main() {
    let x: int = 100;
    let y: int = x % 16;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    println!("✓ Strength reduction: x % 16 marked for bitwise AND optimization");
}

#[test]
fn test_strength_reduce_multiple_in_expression() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = (x * 1) + 0 - (x - x);
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Should apply multiple strength reductions
    assert!(stats.strength_reductions >= 3);
    
    println!("✓ Multiple strength reductions in one expression");
}

#[test]
fn test_strength_reduce_in_nested_expressions() {
    let source = r#"
func main() {
    let a: int = 5;
    let b: int = ((a * 1) + (a * 0)) / 1;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.strength_reductions >= 3);
    
    println!("✓ Strength reduction in nested expressions");
}

#[test]
fn test_strength_reduce_level_0_disabled() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = x * 0;
}
"#;
    
    let (_, stats) = optimize(source, 0);
    assert_eq!(stats.strength_reductions, 0);
    
    println!("✓ Strength reduction disabled at level 0");
}

#[test]
fn test_strength_reduce_combined_with_constant_folding() {
    let source = r#"
func main() {
    let y: int = (10 + 20) * 1;
}
"#;
    
    let (program, stats) = optimize(source, 1);
    // Should first fold 10 + 20 to 30, then apply strength reduction 30 * 1 → 30
    assert!(stats.constants_folded > 0);
    assert!(stats.strength_reductions > 0);
    
    match &program.functions[0].body.statements[0] {
        Statement::Let(let_stmt) => {
            match let_stmt.value.as_ref().unwrap() {
                Expression::Literal(lit) => {
                    assert_eq!(lit.value, Literal::Integer(30));
                }
                _ => panic!("Expected fully optimized literal"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    
    println!("✓ Strength reduction combined with constant folding");
}
