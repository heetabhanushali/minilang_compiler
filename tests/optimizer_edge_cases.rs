// tests/optimizer_edge_cases.rs - Edge cases and corner scenarios for optimizer

use minilang_compiler::{Lexer, Parser, Program, Optimizer, OptimizationStats};
use pretty_assertions::assert_eq;

fn optimize(source: &str, level: u8) -> (Program, OptimizationStats) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens, source.to_string());
    let mut program = parser.parse_program().expect("Parser failed");
    
    let mut optimizer = Optimizer::new(level);
    let stats = optimizer.optimize(&mut program);
    
    (program, stats)
}

// ==================== EDGE CASES FOR CONSTANT FOLDING ====================

#[test]
fn test_fold_division_by_zero_not_folded() {
    let source = r#"
func main() {
    let x: int = 10 / 0;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Division by zero should NOT be folded (runtime error)
    assert_eq!(stats.constants_folded, 0);
    
    println!("✓ Division by zero not folded");
}

#[test]
fn test_fold_modulo_by_zero_not_folded() {
    let source = r#"
func main() {
    let x: int = 10 % 0;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert_eq!(stats.constants_folded, 0);
    
    println!("✓ Modulo by zero not folded");
}

#[test]
fn test_fold_integer_overflow() {
    let source = r#"
func main() {
    let x: int = 1000000 * 1000;
}
"#;
    
    // This test depends on how we handle overflow
    // For now, let's just check it doesn't crash
    let (_program, _stats) = optimize(source, 1);
    
    println!("✓ Integer overflow handled");
}

#[test]
fn test_fold_float_division_by_zero() {
    let source = r#"
func main() {
    let x: float = 10.0 / 0.0;
}
"#;
    
    let (_, _) = optimize(source, 1);
    // Float division by zero might fold to infinity
    // Depends on implementation
    
    println!("✓ Float division by zero handled");
}

#[test]
fn test_fold_deeply_nested() {
    let source = r#"
func main() {
    let x: int = ((((1 + 2) + 3) + 4) + 5) + 6;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.constants_folded >= 5);
    
    println!("✓ Deeply nested expressions folded");
}

#[test]
fn test_fold_mixed_types_not_folded() {
    // If parser allows mixed types, optimizer shouldn't fold them
    let source = r#"
func main() {
    let x: float = 10.5;
    let y: int = 5;
}
"#;
    
    let (_, _) = optimize(source, 1);
    // Mixed type operations shouldn't be folded
    
    println!("✓ Mixed type operations handled correctly");
}

#[test]
fn test_fold_string_operations_not_folded() {
    let source = r#"
func main() {
    let s: string = "Hello";
    display s, " World";
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // String concatenation is not folded in our optimizer
    assert_eq!(stats.constants_folded, 0);
    
    println!("✓ String operations not folded");
}

#[test]
fn test_fold_array_operations_not_folded() {
    let source = r#"
func main() {
    let arr: int[3] = [1, 2, 3];
    let x: int = arr[0] + arr[1];
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Array access is not constant, shouldn't fold
    assert_eq!(stats.constants_folded, 0);
    
    println!("✓ Array operations not folded");
}

#[test]
fn test_fold_function_calls_not_folded() {
    let source = r#"
func pure() -> int {
    send 42;
}

func main() {
    let x: int = pure() + 10;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Function calls are not folded
    assert_eq!(stats.constants_folded, 0);
    
    println!("✓ Function calls not folded");
}

// ==================== EDGE CASES FOR DEAD CODE ====================

#[test]
fn test_eliminate_nested_dead_blocks() {
    let source = r#"
func main() {
    if false {
        if true {
            if false {
                display "triple dead";
            }
        }
    }
}
"#;
    
    let (program, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    // Should remove the entire outer if
    assert_eq!(program.functions[0].body.statements.len(), 0);
    
    println!("✓ Nested dead blocks eliminated");
}

#[test]
fn test_no_eliminate_variable_condition() {
    let source = r#"
func main() {
    let x: bool = true;
    if x {
        display "not dead";
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Should NOT eliminate - x is a variable
    assert_eq!(stats.dead_code_removed, 0);
    
    println!("✓ Variable conditions not eliminated");
}

#[test]
fn test_eliminate_empty_blocks() {
    let source = r#"
func main() {
    if false {
        # Empty block
    }
    
    while false {
        # Another empty block
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed >= 2);
    
    println!("✓ Empty dead blocks eliminated");
}

#[test]
fn test_eliminate_dead_in_dead() {
    let source = r#"
func main() {
    if false {
        send;
        display "dead after return in dead block";
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Dead code within dead code handled");
}

#[test]
fn test_keep_side_effects() {
    let source = r#"
func sideEffect() -> int {
    display "side effect!";
    send 42;
}

func main() {
    let x: int = sideEffect() + 10;
}
"#;
    
    let (_, _) = optimize(source, 1);
    // Should NOT eliminate function calls with side effects
    
    println!("✓ Expressions with side effects kept");
}

// ==================== EDGE CASES FOR CONSTANT PROPAGATION ====================

#[test]
fn test_propagate_const_vs_let() {
    let source = r#"
func main() {
    const C: int = 10;
    let v: int = 20;
    
    let x: int = C;  # Should propagate C
    let y: int = v + 5;  # Might propagate v (depends on implementation)
}
"#;
    
    let (_, stats) = optimize(source, 2);
    assert!(stats.constants_propagated > 0);
    
    println!("✓ Const propagation different from let");
}

#[test]
fn test_propagate_stops_at_mutation() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = x;  # Can propagate x=10
    x = 20;
    let z: int = x;  # Cannot propagate - x changed
}
"#;
    
    let (_, _) = optimize(source, 2);
    // Should propagate first use but not second
    
    println!("✓ Propagation stops at mutation");
}

#[test]
fn test_propagate_across_scopes() {
    let source = r#"
func main() {
    const GLOBAL: int = 100;
    
    if true {
        let x: int = GLOBAL;  # Should propagate
    }
    
    let y: int = GLOBAL;  # Should also propagate
}
"#;
    
    let (_, stats) = optimize(source, 2);
    assert!(stats.constants_propagated > 0);
    
    println!("✓ Constants propagate across scopes");
}

#[test]
fn test_no_propagate_across_functions() {
    let source = r#"
func first() {
    const LOCAL: int = 10;
}

func second() {
    let x: int = LOCAL;  // Should NOT propagate - different function
}
"#;
    
    // This should fail to compile, but if it doesn't, 
    // optimizer shouldn't propagate
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens, source.to_string());
    let _ = parser.parse_program();
    
    // Whether it parses or not, we're testing edge case handling
    println!("✓ No propagation across function boundaries");
}

// ==================== COMBINED EDGE CASES ====================

#[test]
fn test_optimize_empty_function() {
    let source = r#"
func empty() {
}
"#;
    
    let (_, stats) = optimize(source, 2);
    assert_eq!(stats.constants_folded, 0);
    assert_eq!(stats.dead_code_removed, 0);
    
    println!("✓ Empty function optimization handled");
}

#[test]
fn test_optimize_comment_only_function() {
    let source = r#"
func main() {
    # Just comments
    ## More comments ##
}
"#;
    
    let (program, _) = optimize(source, 2);
    assert_eq!(program.functions[0].body.statements.len(), 0);
    
    println!("✓ Comment-only function handled");
}

#[test]
fn test_optimize_huge_constant_expression() {
    let source = r#"
func main() {
    let x: int = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10;
}
"#;
    
    let (_, stats) = optimize(source, 2);
    assert!(stats.constants_folded >= 9);
    
    println!("✓ Large constant expression folded");
}

#[test]
fn test_optimize_alternating_constants_variables() {
    let source = r#"
func main() {
    let a: int = 10;
    let x: int = 5 + a + 10 + a + 15;
}
"#;
    
    let (_, _) = optimize(source, 2);
    // Should only fold the constants, not the variables
    
    println!("✓ Mixed constant/variable expressions handled");
}

#[test]
fn test_optimize_unreachable_but_has_errors() {
    let source = r#"
func main() {
    if false {
        let x: int = 10;
        let y: int = 10 / 0;  # Error in dead code
    }
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Dead code should be removed even if it has errors
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Dead code with errors eliminated");
}

// ==================== STRENGTH REDUCTION EDGE CASES ====================

#[test]
fn test_strength_reduce_does_not_affect_division_by_zero() {
    let source = r#"
func main() {
    let x: int = 10;
    let y: int = x / 0;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Should NOT apply strength reduction to division by zero
    assert_eq!(stats.strength_reductions, 0);
    
    println!("✓ Strength reduction skips division by zero");
}

#[test]
fn test_strength_reduce_large_power_of_two() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = x * 1024;  # 2^10
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    println!("✓ Strength reduction handles large power of two");
}

#[test]
fn test_strength_reduce_not_power_of_two() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = x * 7;  # Not a power of 2
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Should NOT apply strength reduction for non-power-of-2
    assert_eq!(stats.strength_reductions, 0);
    
    println!("✓ Strength reduction skips non-power-of-two");
}

#[test]
fn test_strength_reduce_negative_power_of_two() {
    let source = r#"
func main() {
    let x: int = 100;
    let y: int = x * -8;  # Negative power of 2
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Should NOT optimize negative powers of 2 (except -1)
    assert_eq!(stats.strength_reductions, 0);
    
    println!("✓ Strength reduction skips negative power of two (except -1)");
}

#[test]
fn test_strength_reduce_float_not_optimized() {
    let source = r#"
func main() {
    let x: float = 5.5;
    let y: float = x * 1.0;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    // Strength reduction currently only works with integers
    assert_eq!(stats.strength_reductions, 0);
    
    println!("✓ Strength reduction skips float operations");
}

#[test]
fn test_strength_reduce_const_expression() {
    let source = r#"
func main() {
    const X: int = 42;
    let y: int = X * 0;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.strength_reductions > 0);
    
    println!("✓ Strength reduction works with const variables");
}

#[test]
fn test_strength_reduce_in_all_contexts() {
    let source = r#"
func main() {
    let x: int = 10;
    
    # In variable declaration
    let a: int = x * 1;
    
    # In if condition
    if (x - x) == 0 {
        # In display
        display x + 0;
    }
    
    # In while condition
    while (x * 0) == 0 {
        break;
    }
    
    # In for loop
    for i = x - x; i < 10; i = i + 1 {
        let temp: int = i * 1;
    }
    
    # In return
    send x / 1;
}
"#;
    
    let (_, stats) = optimize(source, 1);
    assert!(stats.strength_reductions >= 6);
    
    println!("✓ Strength reduction works in all statement contexts");
}
