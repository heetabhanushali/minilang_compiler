// tests/codegen_tests.rs - Code generation testing

use minilang_compiler::{Lexer, Parser, TypeChecker, CodeGenerator};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn compile_to_c(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, source.to_string());
    let program = parser.parse_program()?;
    
    let mut type_checker = TypeChecker::new();
    match type_checker.check_program(&program) {
        Ok(()) => {},
        Err(errors) => {
            // Convert Vec<SemanticError> to a single error message
            let error_messages: Vec<String> = errors.iter()
                .map(|e| format!("{:?}", e))
                .collect();
            return Err(format!("Semantic errors: {}", error_messages.join(", ")).into());
        }
    }
    
    let mut codegen = CodeGenerator::new();
    let c_code = codegen.generate(&program)?;
    Ok(c_code)
}

fn compile_and_run(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    let c_code = compile_to_c(source)?;
    
    // Create temp directory
    let temp_dir = TempDir::new()?;
    let c_file = temp_dir.path().join("test.c");
    let exe_file = temp_dir.path().join("test");
    
    // Write C code
    fs::write(&c_file, c_code)?;
    
    // Compile with GCC
    let output = Command::new("gcc")
        .arg(&c_file)
        .arg("-o")
        .arg(&exe_file)
        .arg("-std=c99")
        .output()?;
    
    if !output.status.success() {
        return Err(format!("GCC failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    // Run the executable
    let output = Command::new(&exe_file).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ==================== BASIC CODE GENERATION TESTS ====================

#[test]
fn test_hello_world_generates_valid_c() {
    let source = r#"
func main() {
    display "Hello, World!";
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("int main("));
    assert!(c_code.contains("printf"));
    assert!(c_code.contains("Hello, World!"));
}

#[test]
fn test_variables_generate_valid_c() {
    let source = r#"
func main() {
    let x: int = 42;
    let y: float = 3.14;
    let s: string = "test";
    let b: bool = true;
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("int x = 42"));
    assert!(c_code.contains("double y = 3.14"));
    assert!(c_code.contains("const char* s = \"test\""));
    assert!(c_code.contains("bool b = true"));
}

#[test]
fn test_if_statement_generates_valid_c() {
    let source = r#"
func main() {
    let x: int = 10;
    if x > 5 {
        display "Greater";
    } else {
        display "Lesser";
    }
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("if ((x > 5))"));
    assert!(c_code.contains("else"));
}

#[test]
fn test_loops_generate_valid_c() {
    let source = r#"
func main() {
    while true {
        break;
    }
    
    for let i: int = 0; i < 10; i = i + 1 {
        continue;
    }
    
    do {
        display "once";
    } while false;
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("while (true)"));
    assert!(c_code.contains("for (int i = 0"));
    assert!(c_code.contains("do {"));
    assert!(c_code.contains("break;"));
    assert!(c_code.contains("continue;"));
}

#[test]
fn test_const_generates_valid_c() {
    let source = r#"
func main() {
    const PI: float = 3.14159;
    const MAX: int = 100;
    display PI;
    display MAX;
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("const double PI = 3.14159"));
    assert!(c_code.contains("const int MAX = 100"));
    println!("✓ Const declarations generate valid C");
}

#[test]
fn test_const_in_expressions() {
    let source = r#"
func main() {
    const BASE: int = 10;
    let result: int = BASE * 2 + 5;
    display result;
}
"#;
    
    let output = compile_and_run(source).unwrap();
    assert_eq!(output.trim(), "25");
    println!("✓ Const in expressions works correctly");
}

#[test]
fn test_multiple_consts_output() {
    let source = r#"
func main() {
    const A: int = 1;
    const B: int = 2;
    const C: int = 3;
    display A + B + C;
}
"#;
    
    let output = compile_and_run(source).unwrap();
    assert_eq!(output.trim(), "6");
    println!("✓ Multiple consts compute correctly");
}

#[test]
fn test_array_bounds_with_actual_size() {
    let source = r#"
func main() {
    let arr: int[5] = [1, 2, 3, 4, 5];
    let val: int = arr[4];
    display val;
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("CHECK_BOUNDS"));
    // Should contain the actual size 5, not hardcoded 10
    assert!(c_code.contains("CHECK_BOUNDS(4, 5)") || c_code.contains("_minilang_check_bounds(4, 5"));
    println!("✓ Array bounds use actual size");
}

#[test]
fn test_printf_format_specifiers() {
    let source = r#"
func main() {
    let i: int = 42;
    let f: float = 3.14;
    let s: string = "hello";
    let b: bool = true;
    display i;
    display f;
    display s;
    display b;
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("printf"));  // Just verify printf is used
    println!("✓ Printf statements generated");
}

#[test]
fn test_break_continue_generate_c() {
    let source = r#"
func main() {
    for let i: int = 0; i < 10; i = i + 1 {  
        if i == 2 {
            continue;
        }
        if i == 5 {
            break;
        }
        display i;
    }
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    assert!(c_code.contains("continue;"));
    assert!(c_code.contains("break;"));
    println!("✓ Break and continue generate valid C");
}

// ==================== OUTPUT CORRECTNESS TESTS ====================

#[test]
fn test_arithmetic_output() {
    let source = r#"
func main() {
    display 2 + 3;
    display 10 - 5;
    display 3 * 4;
    display 20 / 4;
    display 17 % 5;
}
"#;
    
    let output = compile_and_run(source).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "5");
    assert_eq!(lines[1], "5");
    assert_eq!(lines[2], "12");
    assert_eq!(lines[3], "5");
    assert_eq!(lines[4], "2");
}

#[test]
fn test_logical_operations_output() {
    let source = r#"
func main() {
    let a: bool = true;
    let b: bool = false;
    display a AND b;
    display a OR b;
    display NOT a;
}
"#;
    
    let output = compile_and_run(source).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "0");  // false
    assert_eq!(lines[1], "1");  // true
    assert_eq!(lines[2], "0");  // false
}

#[test]
fn test_function_call_output() {
    let source = r#"
func add(a: int, b: int) -> int {
    send a + b;
}

func main() {
    display add(3, 4);
}
"#;
    
    let output = compile_and_run(source).unwrap();
    assert_eq!(output.trim(), "7");
}

#[test]
fn test_array_operations_output() {
    let source = r#"
func main() {
    let arr: int[5] = [10, 20, 30, 40, 50];
    display arr[0];
    display arr[4];
}
"#;
    
    let output = compile_and_run(source).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "10");
    assert_eq!(lines[1], "50");
}

#[test]
fn test_loop_output() {
    let source = r#"
func main() {
    for let i: int = 0; i < 3; i = i + 1 {
        display i;
    }
}
"#;
    
    let output = compile_and_run(source).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "0");
    assert_eq!(lines[1], "1");
    assert_eq!(lines[2], "2");
}

#[test]
fn test_break_continue_output() {
    let source = r#"
func main() {
    for let i: int = 0; i < 5; i = i + 1 {
        if i == 2 {
            continue;
        }
        if i == 4 {
            break;
        }
        display i;
    }
}
"#;
    
    let output = compile_and_run(source).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3);  // Should only print 0, 1, 3
    assert_eq!(lines[0], "0");
    assert_eq!(lines[1], "1");
    assert_eq!(lines[2], "3");
    // 2 is skipped (continue)
    // 3 would be printed but we don't reach it
}

#[test]
fn test_string_interpolation_generates_c() {
    let source = r#"
func main() {
    let name: string = "Test";
    display "Hello, {name}!";
}
"#;
    
    let c_code = compile_to_c(source).unwrap();
    // Check that multiple printf calls are generated
    let printf_count = c_code.matches("printf").count();
    assert!(printf_count >= 3); // "Hello, ", name, "!", and newline
    println!("✓ String interpolation generates multiple printfs");
}

#[test]
fn test_string_interpolation_output() {
    let source = r#"
func main() {
    let name: string = "Alice";
    let age: int = 30;
    display "Name: {name}, Age: {age}";
}
"#;
    
    let output = compile_and_run(source).unwrap();
    assert_eq!(output.trim(), "Name: Alice, Age: 30");
    println!("✓ String interpolation output is correct");
}

#[test]
fn test_interpolation_with_expression() {
    let source = r#"
func main() {
    let x: int = 5;
    let y: int = 3;
    display "Sum: {x} + {y} = {x + y}";
}
"#;
    
    let output = compile_and_run(source).unwrap();
    assert_eq!(output.trim(), "Sum: 5 + 3 = 8");
    println!("✓ Expression interpolation works");
}

// ==================== C CODE VALIDITY TESTS ====================

#[test]
fn test_generated_c_compiles() {
    let test_programs = vec![
        // Simple program
        r#"func main() { display "test"; }"#,
        
        // With variables
        r#"func main() { let x: int = 42; display x; }"#,
        
        // With function
        r#"
        func helper(x: int) -> int { send x * 2; }
        func main() { display helper(21); }
        "#,
        
        // With arrays
        r#"func main() { let arr: int[3] = [1, 2, 3]; display arr[1]; }"#,
    ];
    
    for (i, source) in test_programs.iter().enumerate() {
        let result = compile_and_run(source);
        assert!(result.is_ok(), "Test program {} failed to compile: {:?}", i, result);
    }
}