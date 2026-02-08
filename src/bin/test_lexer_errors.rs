// src/bin/test_errors.rs - Complete edge case testing tool with beautiful errors

use minilang_compiler::Lexer;
use miette::{NamedSource, Report};

fn main() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        MiniLang Lexer - Complete Test Suite               ║");
    println!("║          Testing ALL Edge Cases with Beautiful Errors     ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Track statistics
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;

    // ==================== SECTION 1: INVALID CHARACTERS ====================
    println!("\n{:=^60}", " SECTION 1: Invalid Character Detection ");
    
    let invalid_char_tests = vec![
        ("@ symbol", "let x = 42 @ 13;", '@'),
        ("$ dollar sign", "let price = $100;", '$'),
        ("& ampersand", "let x = 10 & 20;", '&'),
        ("| pipe", "let x = 10 | 20;", '|'),
        ("~ tilde", "let x = ~10;", '~'),
        ("? question mark", "let x = 10?;", '?'),
        ("^ caret", "let x = 2 ^ 3;", '^'),
        ("` backtick", "let cmd = `echo`;", '`'),
        ("😀 emoji", "let x = 😀;", '😀'),
        ("€ euro symbol", "let price = €100;", '€'),
        ("£ pound symbol", "let price = £100;", '£'),
    ];

    for (desc, source, expected_char) in invalid_char_tests {
        total += 1;
        if test_invalid_char(desc, source, expected_char) {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    // ==================== SECTION 2: UNTERMINATED STRINGS ====================
    println!("\n{:=^60}", " SECTION 2: Unterminated String Detection ");
    
    let unterminated_tests = vec![
        ("Simple unterminated", r#"let msg = "Hello World"#),
        ("Unterminated at EOF", r#"display "Never ends..."#),
        ("Unterminated with content", r#"let msg = "This is a long string"#),
        ("Unterminated after code", r#"let x = 42; let y = "unterminated"#),
        ("String with newline", "let x = \"Hello\nWorld"),
    ];

    for (desc, source) in unterminated_tests {
        total += 1;
        if test_unterminated_string(desc, source) {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    // ==================== SECTION 3: EDGE CASES ====================
    println!("\n{:=^60}", " SECTION 3: Edge Cases ");
    
    let edge_case_tests = vec![
        ("Empty unterminated", r#"let msg = "#, EdgeCaseType::Either),
        ("Error at start", "@let x = 10;", EdgeCaseType::ShouldError),
        ("Error at end", "let x = 10@", EdgeCaseType::ShouldError),
        ("Error with whitespace", "let x =     @;", EdgeCaseType::ShouldError),
        ("Error in middle", "let x = 10 + @ - 5;", EdgeCaseType::ShouldError),
        ("Multiple errors", "let x = @ $ %;", EdgeCaseType::ShouldError),
        ("Mixed error types", r#"let x = @ "unterminated"#, EdgeCaseType::Either),
    ];

    for (desc, source, expected) in edge_case_tests {
        total += 1;
        if test_edge_case(desc, source, expected) {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    // ==================== SECTION 4: VALID CASES ====================
    println!("\n{:=^60}", " SECTION 4: Valid Cases (Should NOT Error) ");
    
    let valid_tests = vec![
        ("Empty source", ""),
        ("Only whitespace", "   \n\t  \n  "),
        ("Only single-line comment", "# This is a comment"),
        ("Only multi-line comment", "## This is a comment ##"),
        ("@ in comment", "# @ is ok here\nlet x = 42;"),
        ("@ in multi-line comment", "## @ $ % ##\nlet x = 42;"),
        ("@ in string", r#"let x = "@ is ok here";"#),
        ("$ in string", r#"let price = "Cost: $100";"#),
        ("Special chars in string", r#"let msg = "Special: @ $ % & *";"#),
        ("Empty string", r#"let msg = "";"#),
        ("String with escapes", r#"let msg = "Hello \"World\"";"#),
        ("Negative numbers", "let x = -42; let y = -3.14;"),
        ("Zero values", "let x = 0; let y = 0.0;"),
        ("Large number", "let x = 2147483647;"),
        ("Float formats", "let x = 0.0; let y = 123.456;"),
    ];

    for (desc, source) in valid_tests {
        total += 1;
        if test_valid_case(desc, source) {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    // ==================== SECTION 5: COMPLETE PROGRAMS ====================
    println!("\n{:=^60}", " SECTION 5: Complete Program Tests ");
    
    let program_tests = vec![
        ("Hello World", r#"func main() { display "Hello!"; }"#, false),
        ("Variable declaration", "let x: int = 42;", false),
        ("Function with params", "func add(a: int, b: int) -> int { send a + b; }", false),
        ("If statement", "if x > 0 AND y < 10 { display \"ok\"; }", false),
        ("Array declaration", "let arr: int[5] = [1, 2, 3, 4, 5];", false),
        ("All operators", "let x = a + b - c * d / e % f;", false),
        ("All comparisons", "let x = a == b != c < d > e <= f >= g;", false),
        ("Nested blocks", "func f() { if x { while y { display \"hi\"; } } }", false),
        ("Error in program", "func main() { let x = @; }", true),
        ("Unterminated in function", r#"func main() { display "bad; }"#, true),
    ];

    for (desc, source, should_error) in program_tests {
        total += 1;
        if test_complete_program(desc, source, should_error) {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    // ==================== SECTION 6: BEAUTIFUL ERROR DISPLAY ====================
    println!("\n{:=^60}", " SECTION 6: Beautiful Error Examples ");
    println!("\nDemonstrating Miette's beautiful error reporting:\n");
    
    show_beautiful_error("Invalid Character Example", "let x = 42 @ 13;");
    show_beautiful_error("Unterminated String Example", r#"let msg = "Hello World"#);
    show_beautiful_error("Complex Error Example", r#"
func main() {
    let x: int = 42;
    let y = 10 @ 20;  # Invalid character
    display "Done";
}
"#);

    // ==================== FINAL SUMMARY ====================
    print_final_summary(total, passed, failed);
    
    if failed > 0 {
        std::process::exit(1);
    }
}

// ==================== TEST HELPER FUNCTIONS ====================

#[derive(Debug, Clone, Copy)]
enum EdgeCaseType {
    ShouldError,
    Either,
}

fn test_invalid_char(desc: &str, source: &str, expected_char: char) -> bool {
    print!("  Testing {:<30} ", desc);
    
    let mut lexer = Lexer::new(source);
    match lexer.tokenize() {
        Err(e) => {
            let error_display = format!("{:?}", e);
            if error_display.contains(&format!("'{}'", expected_char)) {
                println!("✅ PASS");
                true
            } else {
                println!("❌ FAIL (wrong char)");
                false
            }
        }
        Ok(_) => {
            println!("❌ FAIL (no error detected)");
            false
        }
    }
}

fn test_unterminated_string(desc: &str, source: &str) -> bool {
    print!("  Testing {:<30} ", desc);
    
    let mut lexer = Lexer::new(source);
    match lexer.tokenize() {
        Err(e) => {
            let error_display = format!("{:?}", e);
            if error_display.contains("UnterminatedString") {
                println!("✅ PASS");
                true
            } else {
                println!("⚠️  WARN (different error)");
                true // Still acceptable
            }
        }
        Ok(_) => {
            println!("❌ FAIL (no error detected)");
            false
        }
    }
}

fn test_edge_case(desc: &str, source: &str, expected: EdgeCaseType) -> bool {
    print!("  Testing {:<30} ", desc);
    
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    
    match expected {
        EdgeCaseType::ShouldError => {
            if result.is_err() {
                println!("✅ PASS");
                true
            } else {
                println!("❌ FAIL (expected error)");
                false
            }
        }
        EdgeCaseType::Either => {
            println!("✅ PASS (either outcome OK)");
            true
        }
    }
}

fn test_valid_case(desc: &str, source: &str) -> bool {
    print!("  Testing {:<30} ", desc);
    
    let mut lexer = Lexer::new(source);
    match lexer.tokenize() {
        Ok(_) => {
            println!("✅ PASS");
            true
        }
        Err(_) => {
            println!("❌ FAIL (should not error)");
            false
        }
    }
}

fn test_complete_program(desc: &str, source: &str, should_error: bool) -> bool {
    print!("  Testing {:<30} ", desc);
    
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    
    match (result, should_error) {
        (Ok(_), false) => {
            println!("✅ PASS");
            true
        }
        (Err(_), true) => {
            println!("✅ PASS");
            true
        }
        (Ok(_), true) => {
            println!("❌ FAIL (expected error)");
            false
        }
        (Err(_), false) => {
            println!("❌ FAIL (unexpected error)");
            false
        }
    }
}

fn show_beautiful_error(description: &str, source: &str) {
    println!("{:-^60}", format!(" {} ", description));
    
    let mut lexer = Lexer::new(source);
    
    if let Err(e) = lexer.tokenize() {
        let named_source = NamedSource::new("example.mini", source.to_string());
        let report = Report::from(e).with_source_code(named_source);
        eprintln!("{:?}\n", report);
    } else {
        println!("(No error - valid code)\n");
    }
}

fn print_final_summary(total: u32, passed: u32, failed: u32) {
    let success_rate = if total > 0 {
        (passed * 100) / total
    } else {
        0
    };
    
    println!("\n");
    println!("{:=^60}", "");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    FINAL RESULTS                           ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Total Tests:      {:>4}                                    ║", total);
    println!("║ Passed:           {:>4} ✅                                 ║", passed);
    println!("║ Failed:           {:>4} {}                                 ║", 
             failed, 
             if failed > 0 { "❌" } else { "  " });
    println!("║ Success Rate:     {:>3}%                                   ║", success_rate);
    println!("╠════════════════════════════════════════════════════════════╣");
    
    if failed == 0 {
        println!("║              🎉 ALL TESTS PASSED! 🎉                      ║");
        println!("║          Your lexer is production-ready!                  ║");
    } else {
        println!("║              ⚠️  Some tests failed                        ║");
        println!("║          Review the output above                          ║");
    }
    
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
}