// src/bin/test_semantic.rs - Interactive semantic analyzer testing tool

use minilang_compiler::{Lexer, Parser, TypeChecker};
use miette::{NamedSource, Report};

fn main() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     MiniLang Semantic Analyzer - Interactive Test Tool    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let test_cases = vec![
        // Valid programs
        ("Valid variable declaration", r#"
func main() {
    let x: int = 42;
    display x;
}"#),
        
        ("Valid function call", r#"
func add(a: int, b: int) -> int {
    send a + b;
}
func main() {
    let result: int = add(5, 3);
}"#),
        
        ("Valid control flow", r#"
func main() {
    let x: int = 10;
    if x > 0 {
        display "positive";
    }
}"#),
        
        // Error cases - undefined variable
        ("ERROR: Undefined variable", r#"
func main() {
    display x;
}"#),
        
        // Error cases - type mismatch
        ("ERROR: Type mismatch", r#"
func main() {
    let x: int = "hello";
}"#),
        
        // Error cases - redefinition
        ("ERROR: Variable redefinition", r#"
func main() {
    let x: int = 10;
    let x: int = 20;
}"#),
        
        // Error cases - function errors
        ("ERROR: Undefined function", r#"
func main() {
    let x: int = add(1, 2);
}"#),
        
        ("ERROR: Wrong argument count", r#"
func add(a: int, b: int) -> int {
    send a + b;
}
func main() {
    let x: int = add(1, 2, 3);
}"#),
        
        ("ERROR: Wrong argument type", r#"
func add(a: int, b: int) -> int {
    send a + b;
}
func main() {
    let x: int = add("hello", 2);
}"#),
        
        // Error cases - control flow
        ("ERROR: Non-boolean condition", r#"
func main() {
    let x: int = 42;
    if x {
        display "yes";
    }
}"#),
        
        // Error cases - arrays
        ("ERROR: Non-integer array index", r#"
func main() {
    let arr: int[5];
    let val: int = arr["hello"];
}"#),
        
        ("ERROR: Indexing non-array", r#"
func main() {
    let x: int = 42;
    let val: int = x[0];
}"#),
        
        // Error cases - logical operators
        ("ERROR: AND with non-boolean", r#"
func main() {
    let x: int = 10;
    let b: bool = x AND true;
}"#),
        
        // Error cases - return statements
        ("ERROR: Missing return", r#"
func getValue() -> int {
    let x: int = 42;
}"#),
        
        ("ERROR: Return type mismatch", r#"
func getValue() -> int {
    send "hello";
}"#),
        
        // Complex valid program
        ("Valid complex program", r#"
func factorial(n: int) -> int {
    if n <= 1 {
        send 1;
    }
    send n * factorial(n - 1);
}

func main() {
    let result: int = factorial(5);
    display "Factorial:", result;
}"#),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut false_positives = 0;
    let mut false_negatives = 0;

    for (i, (description, source)) in test_cases.iter().enumerate() {
        println!("\n{:=^60}", format!(" Test {} ", i + 1));
        println!("Description: {}", description);
        println!("{:-^60}", "");

        // Tokenize
        let mut lexer = Lexer::new(source);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                println!("❌ Lexer error (unexpected):");
                let named_source = NamedSource::new("test.mini", source.to_string());
                let report = Report::from(e).with_source_code(named_source);
                eprintln!("{:?}", report);
                failed += 1;
                continue;
            }
        };

        // Parse
        let mut parser = Parser::new(tokens, source.to_string());
        let program = match parser.parse_program() {
            Ok(ast) => ast,
            Err(e) => {
                println!("❌ Parser error (unexpected):");
                let named_source = NamedSource::new("test.mini", source.to_string());
                let report = Report::from(e).with_source_code(named_source);
                eprintln!("{:?}", report);
                failed += 1;
                continue;
            }
        };

        // Semantic Analysis
        let mut type_checker = TypeChecker::new();
        let is_error_expected = description.starts_with("ERROR:");
        
        match type_checker.check_program(&program) {
            Ok(()) => {
                if is_error_expected {
                    println!("❌ FALSE NEGATIVE: Expected semantic error but passed");
                    false_negatives += 1;
                    failed += 1;
                } else {
                    println!("✅ PASSED: Program is semantically valid");
                    passed += 1;
                }
            }
            Err(errors) => {
                if is_error_expected {
                    println!("✅ ERROR DETECTED (expected)");
                    println!("   Found {} semantic error(s):", errors.len());
                    for error in errors.iter().take(2) {
                        let named_source = NamedSource::new("test.mini", source.to_string());
                        let report = Report::from(error.clone()).with_source_code(named_source);
                        eprintln!("{:?}", report);
                    }
                    passed += 1;
                } else {
                    println!("❌ FALSE POSITIVE: Unexpected semantic error");
                    for error in errors {
                        let named_source = NamedSource::new("test.mini", source.to_string());
                        let report = Report::from(error).with_source_code(named_source);
                        eprintln!("{:?}", report);
                    }
                    false_positives += 1;
                    failed += 1;
                }
            }
        }
    }

    println!("\n");
    println!("{:=^60}", "");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    FINAL RESULTS                           ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Total Tests:       {:>4}                                    ║", test_cases.len());
    println!("║ Passed:            {:>4} ✅                                 ║", passed);
    println!("║ Failed:            {:>4} {}                                 ║", 
             failed, 
             if failed > 0 { "❌" } else { "  " });
    println!("║ False Positives:   {:>4}                                    ║", false_positives);
    println!("║ False Negatives:   {:>4}                                    ║", false_negatives);
    println!("╚════════════════════════════════════════════════════════════╝");
    
    if failed == 0 {
        println!("\n🎉 All semantic tests passed!");
    } else {
        println!("\n⚠️  Some tests failed. Review the errors above.");
    }
}