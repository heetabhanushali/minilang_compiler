use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CompilationResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub c_code: Option<String>,
    pub stats: OptimizationStats,
    pub tokens: Option<Vec<TokenInfo>>,
    pub ast: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenInfo {
    pub token_type: String,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize, Deserialize)]
pub struct OptimizationStats {
    pub constants_folded: usize,
    pub dead_code_removed: usize,
    pub constants_propagated: usize,
    pub strength_reductions: usize,
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn compile(source: &str, opt_level: u8) -> String {
    use crate::{Lexer, Parser, TypeChecker, CodeGenerator, Optimizer};
    
    // Tokenize
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return serde_json::to_string(&CompilationResult {
                success: false,
                output: None,
                error: Some(format!("Lexer Error: {:?}", e)),
                c_code: None,
                stats: OptimizationStats {
                    constants_folded: 0,
                    dead_code_removed: 0,
                    constants_propagated: 0,
                    strength_reductions: 0,
                },
                tokens: None,
                ast: None,
            }).unwrap();
        }
    };
    
    // Convert tokens to TokenInfo for JSON
    let token_info: Vec<TokenInfo> = tokens.iter().enumerate().map(|(_idx, tok)| {
        let line = source[..tok.span.start].lines().count();
        let last_newline = source[..tok.span.start].rfind('\n').unwrap_or(0);
        let column = tok.span.start - last_newline;
        
        TokenInfo {
            token_type: format!("{:?}", tok.token).split('(').next().unwrap_or("Unknown").to_string(),
            value: source[tok.span.start..tok.span.end].to_string(),
            line,
            column,
        }
    }).collect();
    
    // Parse
    let mut parser = Parser::new(tokens, source.to_string());
    let mut program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            return serde_json::to_string(&CompilationResult {
                success: false,
                output: None,
                error: Some(format!("Parser Error: {:?}", e)),
                c_code: None,
                stats: OptimizationStats {
                    constants_folded: 0,
                    dead_code_removed: 0,
                    constants_propagated: 0,
                    strength_reductions: 0,
                },
                tokens: Some(token_info),
                ast: None,
            }).unwrap();
        }
    };
    
    // Generate AST display as JSON
    let ast_display = serde_json::to_string_pretty(&program)
        .unwrap_or_else(|_| format!("{:#?}", program));
    
    // Type check
    let mut type_checker = TypeChecker::new();
    if let Err(errors) = type_checker.check_program(&program) {
        let error_msg = errors.iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join("\n");
        
        return serde_json::to_string(&CompilationResult {
            success: false,
            output: None,
            error: Some(format!("Type Error:\n{}", error_msg)),
            c_code: None,
            stats: OptimizationStats {
                constants_folded: 0,
                dead_code_removed: 0,
                constants_propagated: 0,
                strength_reductions: 0,
            },
            tokens: Some(token_info),
            ast: Some(ast_display),
        }).unwrap();
    }
    
    // Optimize
    let mut optimizer = Optimizer::new(opt_level);
    let opt_stats = optimizer.optimize(&mut program);
    
    // Generate C code
    let mut codegen = CodeGenerator::new();
    let c_code = match codegen.generate(&program) {
        Ok(code) => code,
        Err(e) => {
            return serde_json::to_string(&CompilationResult {
                success: false,
                output: None,
                error: Some(format!("Code Generation Error: {}", e)),
                c_code: None,
                stats: OptimizationStats {
                    constants_folded: opt_stats.constants_folded,
                    dead_code_removed: opt_stats.dead_code_removed,
                    constants_propagated: opt_stats.constants_propagated,
                    strength_reductions: opt_stats.strength_reductions,
                },
                tokens: Some(token_info),
                ast: Some(ast_display),
            }).unwrap();
        }
    };
    
    // Success
    serde_json::to_string(&CompilationResult {
        success: true,
        output: Some(format!("Program compiled with optimization level {}\n\nYou can download the C code and run locally:\n   gcc program.c -o program\n   ./program", opt_level)),
        error: None,
        c_code: Some(c_code),
        stats: OptimizationStats {
            constants_folded: opt_stats.constants_folded,
            dead_code_removed: opt_stats.dead_code_removed,
            constants_propagated: opt_stats.constants_propagated,
            strength_reductions: opt_stats.strength_reductions,
        },
        tokens: Some(token_info),
        ast: Some(ast_display),
    }).unwrap()
}