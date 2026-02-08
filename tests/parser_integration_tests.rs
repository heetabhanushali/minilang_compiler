// tests/parser_integration_tests.rs - Real MiniLang program parsing tests

use minilang_compiler::{Lexer, Parser, Program};

fn parse_program(source: &str) -> Program {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens, source.to_string());
    parser.parse_program().expect("Parser failed")
}

// ==================== COMPLETE PROGRAM TESTS ====================

#[test]
fn test_parse_bubble_sort() {
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
    let numbers: int[10] = [64, 34, 25, 12, 22, 11, 90, 88, 76, 1];
    bubbleSort(numbers, 10);
    display "Sorted!";
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 2);
    assert_eq!(ast.functions[0].name, "bubbleSort");
    assert_eq!(ast.functions[1].name, "main");
    
    println!("✓ Bubble sort program parsed");
}

#[test]
fn test_parse_factorial() {
    let source = r#"
func factorial(n: int) -> int {
    if n <= 1 {
        send 1;
    }
    send n * factorial(n - 1);
}

func main() {
    let result: int = factorial(5);
    display "Factorial of 5 is:", result;
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 2);
    assert_eq!(ast.functions[0].name, "factorial");
    assert!(ast.functions[0].return_type.is_some());
    
    println!("✓ Factorial program parsed");
}

#[test]
fn test_parse_fibonacci() {
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
    for i = 0; i < 10; i = i + 1 {
        let result: int = fibonacci(i);
        display "Fib(", i, ") =", result;
    }
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 2);
    
    println!("✓ Fibonacci program parsed");
}

#[test]
fn test_parse_prime_checker() {
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
    let count: int = 0;
    for n = 2; n <= 100; n = n + 1 {
        if isPrime(n) {
            display n, "is prime";
            count = count + 1;
        }
    }
    display "Total primes found:", count;
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 2);
    
    println!("✓ Prime checker program parsed");
}

#[test]
fn test_parse_nested_loops() {
    let source = r#"
func printMatrix() {
    for i = 0; i < 3; i = i + 1 {
        for j = 0; j < 3; j = j + 1 {
            display "Matrix[", i, "][", j, "]";
        }
    }
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 1);
    
    println!("✓ Nested loops program parsed");
}

#[test]
fn test_parse_complex_expressions() {
    let source = r#"
func calculate() -> float {
    let a: float = 10.5;
    let b: float = 20.3;
    let c: float = 5.7;
    
    let result: float = (a + b) * c / 2.0 - (a * b) + (c * c);
    send result;
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 1);
    
    println!("✓ Complex expressions program parsed");
}

#[test]
fn test_parse_all_statements() {
    let source = r#"
func testAll() {
    # Variable declaration
    let x: int = 42;
    
    # Display
    display "Value:", x;
    
    # If-else
    if x > 0 {
        display "Positive";
    } else {
        display "Non-positive";
    }
    
    # While loop
    while x > 0 {
        x = x - 1;
    }
    
    # Do-while loop
    do {
        x = x + 1;
    } while x < 10;
    
    # For loop
    for i = 0; i < 5; i = i + 1 {
        display i;
    }
    
    # Return
    send x;
}
"#;
    
    let ast = parse_program(source);
    let statements = &ast.functions[0].body.statements;
    assert!(statements.len() >= 7);
    
    println!("✓ All statement types program parsed");
}

#[test]
fn test_parse_array_operations() {
    let source = r#"
func arrayOps() {
    let arr: int[10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum: int = 0;
    
    for i = 0; i < 10; i = i + 1 {
        sum = sum + arr[i];
    }
    
    arr[0] = 100;
    let first: int = arr[0];
    let last: int = arr[9];
    
    display "Sum:", sum;
    display "First:", first;
    display "Last:", last;
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 1);
    
    println!("✓ Array operations program parsed");
}

#[test]
fn test_parse_logical_operations() {
    let source = r#"
func checkConditions(x: int, y: int, z: bool) -> bool {
    if x > 0 AND y > 0 {
        send true;
    }
    
    if x < 0 OR y < 0 {
        send false;
    }
    
    if NOT z {
        send x == y;
    }
    
    send (x > y) AND (y > 0) OR NOT z;
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 1);
    
    println!("✓ Logical operations program parsed");
}

#[test]
fn test_parse_multiple_functions_complex() {
    let source = r#"
func helper(x: int) -> int {
    send x * 2;
}

func process(arr: int[5]) {
    for i = 0; i < 5; i = i + 1 {
        arr[i] = helper(arr[i]);
    }
}

func display_array(arr: int[5]) {
    for i = 0; i < 5; i = i + 1 {
        display "arr[", i, "] =", arr[i];
    }
}

func main() {
    let data: int[5] = [1, 2, 3, 4, 5];
    process(data);
    display_array(data);
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 4);
    assert_eq!(ast.functions[0].name, "helper");
    assert_eq!(ast.functions[1].name, "process");
    assert_eq!(ast.functions[2].name, "display_array");
    assert_eq!(ast.functions[3].name, "main");
    
    println!("✓ Multiple complex functions program parsed");
}


#[test]
fn test_parse_program_with_const() {
    let source = r#"
func checkSpeed(speed: int) -> bool {
    const MAX_SPEED: int = 100;
    if speed > MAX_SPEED {
        send false;
    }
    send true;
}

func main() {
    const GRAVITY: float = 9.8;
    display "Gravity:", GRAVITY;
    let safe: bool = checkSpeed(80);
    display "Safe:", safe;
}
"#;
    
    let ast = parse_program(source);
    assert_eq!(ast.functions.len(), 2);
    assert_eq!(ast.functions[0].name, "checkSpeed");
    assert_eq!(ast.functions[1].name, "main");
    
    println!("✓ Program with function-local constants parsed");
}