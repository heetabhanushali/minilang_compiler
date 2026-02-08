// src/bin/test_parser.rs - Interactive parser testing tool

use minilang_compiler::{Lexer, Parser};
use miette::{NamedSource, Report};

fn main() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        MiniLang Parser - Interactive Test Tool            ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let test_cases = vec![
        // Basic programs
        ("Empty function", "func main() { }"),
        ("Hello world", r#"func main() { display "Hello!"; }"#),
        ("Variable declaration", "func main() { let x: int = 42; }"),
        
        // Control flow
        ("If statement", "func main() { if x > 0 { display x; } }"),
        ("While loop", "func main() { while x < 10 { x = x + 1; } }"),
        ("For loop", "func main() { for i = 0; i < 10; i = i + 1 { display i; } }"),
        
        // Functions
        ("Function with params", "func add(a: int, b: int) -> int { send a + b; }"),
        ("Recursive function", r#"
func factorial(n: int) -> int {
    if n <= 1 {
        send 1;
    }
    send n * factorial(n - 1);
}
"#),
        
        // Arrays
        ("Array declaration", "func main() { let arr: int[5] = [1,2,3,4,5]; }"),
        ("Array indexing", "func main() { let x: int = arr[0]; }"),
        
        // Expressions
        ("Arithmetic", "func main() { let x: int = 2 + 3 * 4; }"),
        ("Logical", "func main() { let b: bool = x > 0 AND y < 10; }"),
        
        // Error cases
        ("Missing semicolon", "func main() { let x: int = 42 }"),
        ("Missing brace", "func main() { display x;"),
        ("Missing type", "func main() { let x = 42; }"),
        ("Invalid syntax", "func { }"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (i, (description, source)) in test_cases.iter().enumerate() {
        println!("\n{:=^60}", format!(" Test {} ", i + 1));
        println!("Description: {}", description);
        println!("Source: {}", if source.len() > 50 { 
            format!("{}...", &source[..50]) 
        } else { 
            source.to_string() 
        });
        println!("{:-^60}", "");

        // Tokenize
        let mut lexer = Lexer::new(source);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                println!("❌ Lexer error:");
                let named_source = NamedSource::new("test.mini", source.to_string());
                let report = Report::from(e).with_source_code(named_source);
                eprintln!("{:?}", report);
                failed += 1;
                continue;
            }
        };

        println!("✓ Tokenized: {} tokens", tokens.len());

        // Parse
        let mut parser = Parser::new(tokens, source.to_string());
        match parser.parse_program() {
            Ok(ast) => {
                println!("✅ PARSED SUCCESSFULLY");
                println!("   Functions: {}", ast.functions.len());
                for func in &ast.functions {
                    println!("     - {} ({} params, {} statements)", 
                        func.name, 
                        func.params.len(),
                        func.body.statements.len()
                    );
                }
                passed += 1;
            }
            Err(e) => {
                if description.starts_with("Missing") || description.starts_with("Invalid") {
                    println!("✅ ERROR DETECTED (expected)");
                    let named_source = NamedSource::new("test.mini", source.to_string());
                    let report = Report::from(e).with_source_code(named_source);
                    eprintln!("{:?}", report);
                    passed += 1;
                } else {
                    println!("❌ UNEXPECTED ERROR");
                    let named_source = NamedSource::new("test.mini", source.to_string());
                    let report = Report::from(e).with_source_code(named_source);
                    eprintln!("{:?}", report);
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
    println!("║ Total Tests:      {:>4}                                    ║", test_cases.len());
    println!("║ Passed:           {:>4} ✅                                 ║", passed);
    println!("║ Failed:           {:>4} {}                                 ║", 
             failed, 
             if failed > 0 { "❌" } else { "  " });
    println!("╚════════════════════════════════════════════════════════════╝");
}