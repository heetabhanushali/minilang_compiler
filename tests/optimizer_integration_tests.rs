// tests/optimizer_integration_tests.rs - Real MiniLang program optimization tests

use minilang_compiler::{Lexer, Parser, Program, Optimizer, OptimizationStats};

fn optimize_program(source: &str, level: u8) -> (Program, OptimizationStats) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens, source.to_string());
    let mut program = parser.parse_program().expect("Parser failed");
    
    let mut optimizer = Optimizer::new(level);
    let stats = optimizer.optimize(&mut program);
    
    (program, stats)
}

// ==================== COMPLETE PROGRAM OPTIMIZATION TESTS ====================

#[test]
fn test_optimize_fibonacci() {
    let source = r#"
func fibonacci(n: int) -> int {
    if n <= 0 {
        send 0;
    } else if n == 1 {
        send 1;
    }
    send fibonacci(n - 1) + fibonacci(n - 2);
}

func main() {
    const N: int = 5 + 5;  # Should fold to 10
    let result: int = fibonacci(N);
    display "Fibonacci of ", N, " is ", result;
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded > 0); // Should fold 5 + 5
    
    println!("✓ Fibonacci program optimized");
}

#[test]
fn test_optimize_bubble_sort() {
    let source = r#"
func bubbleSort(arr: int[10], size: int) {
    let swapped: bool = true;
    let passes: int = 0;
    
    while swapped AND passes < size - 1 {
        swapped = false;
        
        for i = 0; i < size - passes - 1; i = i + 1 {
            if arr[i] > arr[i + 1] {
                let temp: int = arr[i];
                arr[i] = arr[i + 1];
                arr[i + 1] = temp;
                swapped = true;
            }
        }
        
        passes = passes + 1;
    }
}

func main() {
    const SIZE: int = 5 * 2;  # Should fold to 10
    let numbers: int[10] = [64, 34, 25, 12, 22, 11, 90, 88, 76, 1];
    bubbleSort(numbers, SIZE);
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded > 0);
    
    println!("✓ Bubble sort optimized");
}

#[test]
fn test_optimize_prime_checker() {
    let source = r#"
func isPrime(n: int) -> bool {
    if n <= 1 {
        send false;
    }
    
    if n == 2 {
        send true;
    }
    
    if n % 2 == 0 {
        send false;
    }
    
    for i = 3; i * i <= n; i = i + 2 {
        if n % i == 0 {
            send false;
        }
    }
    
    send true;
}

func main() {
    const START: int = 1 + 1;  # Should fold to 2
    const END: int = 10 * 10;  # Should fold to 100
    
    for n = START; n <= END; n = n + 1 {
        if isPrime(n) {
            display n;
        }
    }
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded >= 2); // Should fold START and END
    
    println!("✓ Prime checker optimized");
}

#[test]
fn test_optimize_calculator() {
    let source = r#"
func calculate(op: int, a: float, b: float) -> float {
    if op == 1 {
        send a + b;
    } else if op == 2 {
        send a - b;
    } else if op == 3 {
        send a * b;
    } else if op == 4 {
        if b != 0.0 {
            send a / b;
        }
        send 0.0;
    }
    send 0.0;
}

func main() {
    const OP_ADD: int = 1;
    const OP_MUL: int = 3;
    
    let x: float = 10.5;
    let y: float = 20.3;
    
    # These constant ops should propagate
    let sum: float = calculate(OP_ADD, x, y);
    let product: float = calculate(OP_MUL, x, y);
    
    # This should fold
    const ALWAYS_TRUE: bool = 10 > 5 AND 20 > 10;
    
    if ALWAYS_TRUE {  # Dead code elimination opportunity
        display "Results: ", sum, product;
    } else {
        display "Never happens";
    }
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded > 0);
    assert!(stats.constants_propagated > 0);
    
    println!("✓ Calculator program optimized");
}

#[test]
fn test_optimize_matrix_operations() {
    let source = r#"
func printMatrix(matrix: int[9], rows: int, cols: int) {
    for i = 0; i < rows; i = i + 1 {
        for j = 0; j < cols; j = j + 1 {
            let index: int = i * cols + j;
            display matrix[index];
        }
    }
}

func main() {
    const ROWS: int = 3;
    const COLS: int = 3;
    const SIZE: int = ROWS * COLS;  # Should fold to 9
    
    let matrix: int[9];
    
    # Initialize with pattern
    for i = 0; i < SIZE; i = i + 1 {
        matrix[i] = i + 1;
    }
    
    printMatrix(matrix, ROWS, COLS);
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded > 0); // Should fold ROWS * COLS
    
    println!("✓ Matrix operations optimized");
}

#[test]
fn test_optimize_factorial() {
    let source = r#"
func factorial(n: int) -> int {
    if n <= 1 {
        send 1;
    }
    send n * factorial(n - 1);
}

func main() {
    const BASE: int = 2 * 2 + 1;  # Should fold to 5
    
    # Test with constant folded value
    let result1: int = factorial(BASE);
    
    # Dead code that should be eliminated
    if false {
        display "This is impossible";
        let dead_var: int = 999;
    }
    
    # This should remain
    if true {
        display "Factorial of ", BASE, " is ", result1;
    }
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded > 0);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Factorial program optimized");
}

#[test]
fn test_optimize_game_logic() {
    let source = r#"
func checkWin(score: int) -> bool {
    const WIN_SCORE: int = 100;
    send score >= WIN_SCORE;
}

func main() {
    const MAX_ROUNDS: int = 5 + 5;  # Folds to 10
    const POINTS_PER_ROUND: int = 10 * 2;  # Folds to 20
    
    let score: int = 0;
    let round: int = 0;
    
    while round < MAX_ROUNDS AND NOT checkWin(score) {
        score = score + POINTS_PER_ROUND;
        round = round + 1;
        
        # Dead code if we know the condition
        if false {
            display "Cheat mode activated!";
            score = 1000;
        }
    }
    
    # Another optimization opportunity
    if true AND true {
        display "Game ended!";
    } else {
        display "This never prints";
    }
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded > 0);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Game logic optimized");
}

#[test]
fn test_optimize_string_processing() {
    let source = r#"
func processString(s: string) {
    const PREFIX: string = "Result: ";
    display PREFIX, s;
}

func main() {
    # Constants that can fold
    const COUNT: int = 2 + 3;  # Folds to 5
    const IS_VALID: bool = true AND NOT false;  # Folds to true
    
    if IS_VALID {
        for i = 0; i < COUNT; i = i + 1 {
            processString("test");
        }
    } else {
        # Dead code
        for i = 0; i < 1000; i = i + 1 {
            display "Never runs";
        }
    }
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded > 0);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ String processing optimized");
}

#[test]
fn test_optimize_nested_functions() {
    let source = r#"
func inner(x: int) -> int {
    const MULT: int = 2;
    send x * MULT;
}

func middle(y: int) -> int {
    const ADD: int = 5 + 5;  # Folds to 10
    send inner(y) + ADD;
}

func outer(z: int) -> int {
    const SUB: int = 3 * 4;  # Folds to 12
    send middle(z) - SUB;
}

func main() {
    const INPUT: int = 7 - 2;  # Folds to 5
    let result: int = outer(INPUT);
    
    # Optimization in conditionals
    if 100 > 50 {  # Folds to true
        display "Result: ", result;
    } else {
        display "Dead branch";
    }
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.constants_folded >= 4); // Multiple constants to fold
    assert!(stats.dead_code_removed > 0); // Dead else branch
    
    println!("✓ Nested functions optimized");
}

#[test]
fn test_optimize_all_features() {
    let source = r#"
func allFeatures(param: int) -> int {
    # Constant folding opportunities
    const BASE: int = 10 + 20;  # Folds to 30
    const MULTIPLIER: int = 2 * 3;  # Folds to 6
    
    # Dead code elimination opportunity
    if 5 > 10 {  # Folds to false
        display "Dead code";
        send 0;
    }
    
    # Constant propagation opportunity
    let result: int = BASE * MULTIPLIER;  # Could fold to 180
    
    # More dead code
    while false {
        display "Never runs";
        param = param + 1;
    }
    
    # Complex expression with constants
    let final: int = result + (100 / 10) - (2 * 2);  # Partially folds
    
    send final + param;
}

func main() {
    const TEST_VALUE: int = 15 - 10;  # Folds to 5
    
    # This condition folds and eliminates dead code
    if true OR false {  # Folds to true
        let answer: int = allFeatures(TEST_VALUE);
        display "Answer: ", answer;
    } else {
        display "This is dead code";
        let dead: int = 999;
    }
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    
    // Should have multiple optimizations
    assert!(stats.constants_folded > 5);
    assert!(stats.dead_code_removed > 2);
    
    println!("✓ All features program optimized");
    println!("   Constants folded: {}", stats.constants_folded);
    println!("   Dead code removed: {}", stats.dead_code_removed);
    println!("   Constants propagated: {}", stats.constants_propagated);
}


// ==================== STRENGTH REDUCTION INTEGRATION TESTS ====================

#[test]
fn test_strength_reduction_in_real_algorithm() {
    let source = r#"
func processData(data: int[10], size: int) -> int {
    let result: int = 0;
    
    for i = 0; i < size; i = i + 1 {
        # Strength reduction opportunities
        let doubled: int = data[i] * 2;  # Power of 2
        let halved: int = data[i] / 2;   # Power of 2
        let masked: int = data[i] % 8;   # Power of 2
        
        # Identity operations
        let same: int = data[i] * 1;
        let also_same: int = data[i] + 0;
        
        result = result + doubled + halved + masked + same + also_same;
    }
    
    send result;
}

func main() {
    let arr: int[10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let final: int = processData(arr, 10);
    display "Result: ", final;
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    // Should find many strength reduction opportunities
    assert!(stats.strength_reductions >= 5);
    
    println!("✓ Strength reduction in real algorithm");
    println!("   Strength reductions: {}", stats.strength_reductions);
}

#[test]
fn test_strength_reduction_with_constants() {
    let source = r#"
func calculate(x: int) -> int {
    const ZERO: int = 0;
    const ONE: int = 1;
    const POWER_TWO: int = 8;
    
    let a: int = x * ZERO;      # Should reduce to 0
    let b: int = x * ONE;       # Should reduce to x
    let c: int = x * POWER_TWO; # Should mark for shift
    
    send a + b + c;
}

func main() {
    let result: int = calculate(100);
    display result;
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    assert!(stats.strength_reductions >= 3);
    assert!(stats.constants_propagated > 0);
    
    println!("✓ Strength reduction with constant propagation");
}

#[test]
fn test_strength_reduction_full_optimization_pipeline() {
    let source = r#"
func optimizeMe() -> int {
    # Constant folding + strength reduction
    let a: int = (10 + 20) * 1;  # Folds to 30, then 30 * 1 → 30
    
    # Explicit dead code
    if false {
        send 999;
    }
    
    send a;
}

func main() {
    let value: int = optimizeMe();
    
    # More optimizations
    let doubled: int = value * 2;  # Power of 2
    let zero: int = value - value;  # Self subtraction
    
    display doubled + zero;  # zero is 0, so this is just doubled
}
"#;
    
    let (_, stats) = optimize_program(source, 2);
    
    assert!(stats.constants_folded > 0);
    assert!(stats.strength_reductions > 0);
    assert!(stats.dead_code_removed > 0);
    
    println!("✓ Full optimization pipeline with strength reduction");
    println!("   Constants folded: {}", stats.constants_folded);
    println!("   Strength reductions: {}", stats.strength_reductions);
    println!("   Dead code removed: {}", stats.dead_code_removed);
}
