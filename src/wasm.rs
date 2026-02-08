use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use miette::{NamedSource, GraphicalReportHandler, GraphicalTheme};

use crate::{Lexer, Parser, TypeChecker, CodeGenerator, Optimizer};
use crate::errors::{LexerError, ParserError, SemanticError};

#[derive(Serialize, Deserialize)]
pub struct CompilationResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub error_ansi: Option<String>,
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

#[derive(Serialize, Deserialize, Default)]
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

/// Wrapper that attaches source code to any miette Diagnostic
/// This is needed because our error types don't carry source code themselves
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("{title}")]
struct DiagnosticWithSource {
    title: String,

    #[source_code]
    source_code: NamedSource<String>,

    #[label("{label_text}")]
    span: miette::SourceSpan,

    label_text: String,

    #[help]
    help_text: Option<String>,

    #[diagnostic(code)]
    code_str: Option<String>,
}

/// Extract span, label, help, and title from a LexerError
fn lexer_to_diagnostic(error: &LexerError, source: &str) -> DiagnosticWithSource {
    let (title, span, label, help, code) = match error {
        LexerError::UnexpectedChar { char, span } => (
            format!("unexpected character '{}'", char),
            *span,
            "unexpected character here".to_string(),
            Some(format!("Valid characters include letters, digits, and operators (+, -, *, /, etc.)")),
            "minilang::lexer::unexpected_char",
        ),
        LexerError::UnterminatedString { start } => (
            "unterminated string literal".to_string(),
            *start,
            "string starts here but never ends".to_string(),
            Some("Strings must be closed with a matching double quote (\")".to_string()),
            "minilang::lexer::unterminated_string",
        ),
        LexerError::InvalidNumber { span } => (
            "invalid number format".to_string(),
            *span,
            "invalid number format here".to_string(),
            Some("Numbers should be integers (42) or floats (3.14)".to_string()),
            "minilang::lexer::invalid_number",
        ),
        LexerError::IntegerOverflow { value, span } => (
            format!("integer literal out of range: '{}'", value),
            *span,
            "this number is too large".to_string(),
            Some("Integer literals must be between -2147483648 and 2147483647".to_string()),
            "minilang::lexer::integer_overflow",
        ),
    };

    DiagnosticWithSource {
        title,
        source_code: NamedSource::new("main.mini", source.to_string()),
        span,
        label_text: label,
        help_text: help,
        code_str: Some(code.to_string()),
    }
}

/// Extract span, label, help, and title from a ParserError
fn parser_to_diagnostic(error: &ParserError, source: &str) -> DiagnosticWithSource {
    let (title, span, label, help, code) = match error {
        ParserError::UnexpectedToken { expected, found, span } => (
            "unexpected token".to_string(),
            *span,
            "unexpected token here".to_string(),
            Some(format!("Expected {}, but found {}", expected, found)),
            "minilang::parser::unexpected_token",
        ),
        ParserError::MissingSemicolon { span } => (
            "missing semicolon".to_string(),
            *span,
            "semicolon expected here".to_string(),
            Some("Add a semicolon ';' at the end of the statement".to_string()),
            "minilang::parser::missing_semicolon",
        ),
        ParserError::MissingClosingBrace { span } => (
            "missing closing brace".to_string(),
            *span,
            "closing brace expected".to_string(),
            Some("Add a closing brace '}' to match the opening brace".to_string()),
            "minilang::parser::missing_brace",
        ),
        ParserError::InvalidExpression { span } => (
            "invalid expression".to_string(),
            *span,
            "invalid expression".to_string(),
            Some("This doesn't look like a valid expression".to_string()),
            "minilang::parser::invalid_expression",
        ),
        ParserError::MissingType { span } => (
            "missing type annotation".to_string(),
            *span,
            "type annotation expected here".to_string(),
            Some("Variables require type annotations: let name: type = value;".to_string()),
            "minilang::parser::missing_type",
        ),
        ParserError::UnexpectedEof { expected } => (
            "unexpected end of input".to_string(),
            miette::SourceSpan::from(0..0),
            format!("expected {}", expected),
            Some("The program ended unexpectedly. Check for missing closing braces.".to_string()),
            "minilang::parser::unexpected_eof",
        ),
    };

    DiagnosticWithSource {
        title,
        source_code: NamedSource::new("main.mini", source.to_string()),
        span,
        label_text: label,
        help_text: help,
        code_str: Some(code.to_string()),
    }
}

/// Extract span, label, help, and title from a SemanticError
fn semantic_to_diagnostic(error: &SemanticError, source: &str) -> DiagnosticWithSource {
    let (title, span, label, help, code) = match error {
        SemanticError::UndefinedVariable { name, span, suggestion, context } => {
            let t = match context {
                Some(ctx) => format!("undefined variable '{}' in {}", name, ctx),
                None => format!("undefined variable '{}'", name),
            };
            (t, *span, "undefined here".to_string(), Some(suggestion.clone()), "minilang::semantic::undefined_variable")
        }
        SemanticError::TypeMismatch { expected, found, span } => (
            "type mismatch".to_string(),
            *span,
            "type mismatch here".to_string(),
            Some(format!("Expected type {}, but found {}", expected, found)),
            "minilang::semantic::type_mismatch",
        ),
        SemanticError::DuplicateDefinition { name, span, .. } => (
            "variable already defined".to_string(),
            *span,
            "redefined here".to_string(),
            Some(format!("Variable '{}' was already defined in this scope", name)),
            "minilang::semantic::duplicate_definition",
        ),
        SemanticError::UndefinedFunction { name, span, suggestion, context } => {
            let t = match context {
                Some(ctx) => format!("undefined function '{}' in {}", name, ctx),
                None => format!("undefined function '{}'", name),
            };
            (t, *span, "undefined function".to_string(), Some(suggestion.clone()), "minilang::semantic::undefined_function")
        }
        SemanticError::ArgumentCountMismatch { name, expected, found, span } => (
            "wrong number of arguments".to_string(),
            *span,
            "called here".to_string(),
            Some(format!("Function '{}' expects {} arguments, but {} were provided", name, expected, found)),
            "minilang::semantic::argument_count",
        ),
        SemanticError::MissingReturn { name, return_type, span } => (
            "missing return".to_string(),
            *span,
            "function declared here".to_string(),
            Some(format!("Function '{}' must return a value of type {} on all code paths", name, return_type)),
            "minilang::semantic::missing_return",
        ),
        SemanticError::BreakOutsideLoop { statement, span } => (
            "break/continue outside loop".to_string(),
            *span,
            "not inside a loop".to_string(),
            Some(format!("'{}' can only be used inside a loop (while, do-while, or for)", statement)),
            "minilang::semantic::break_outside_loop",
        ),
    };

    DiagnosticWithSource {
        title,
        source_code: NamedSource::new("main.mini", source.to_string()),
        span,
        label_text: label,
        help_text: help,
        code_str: Some(code.to_string()),
    }
}

/// Render any DiagnosticWithSource using miette's GraphicalReportHandler
fn render_diagnostic(diag: &DiagnosticWithSource) -> String {
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode());
    let mut output = String::new();
    match handler.render_report(&mut output, diag) {
        Ok(_) => output,
        Err(_) => format!("{}", diag),
    }
}

fn render_lexer_error(error: LexerError, source: &str) -> String {
    let diag = lexer_to_diagnostic(&error, source);
    render_diagnostic(&diag)
}

fn render_parser_error(error: ParserError, source: &str) -> String {
    let diag = parser_to_diagnostic(&error, source);
    render_diagnostic(&diag)
}

fn render_semantic_errors(errors: Vec<SemanticError>, source: &str) -> String {
    let mut output = String::new();
    for error in &errors {
        let diag = semantic_to_diagnostic(error, source);
        output.push_str(&render_diagnostic(&diag));
        output.push('\n');
    }
    output
}

#[wasm_bindgen]
pub fn compile(source: &str, opt_level: u8) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            let error_ansi = render_lexer_error(e.clone(), source);
            return serde_json::to_string(&CompilationResult {
                success: false,
                output: None,
                error: Some(format!("{:?}", e)),
                error_ansi: Some(error_ansi),
                c_code: None,
                stats: OptimizationStats::default(),
                tokens: None,
                ast: None,
            }).unwrap();
        }
    };

    let token_info: Vec<TokenInfo> = tokens.iter().map(|tok| {
        let line = source[..tok.span.start].matches('\n').count() + 1;
        let last_newline = source[..tok.span.start].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let column = tok.span.start - last_newline + 1;
        TokenInfo {
            token_type: format!("{:?}", tok.token).split('(').next().unwrap_or("Unknown").to_string(),
            value: source[tok.span.start..tok.span.end].to_string(),
            line,
            column,
        }
    }).collect();

    let mut parser = Parser::new(tokens, source.to_string());
    let mut program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            let error_ansi = render_parser_error(e.clone(), source);
            return serde_json::to_string(&CompilationResult {
                success: false,
                output: None,
                error: Some(format!("{:?}", e)),
                error_ansi: Some(error_ansi),
                c_code: None,
                stats: OptimizationStats::default(),
                tokens: Some(token_info),
                ast: None,
            }).unwrap();
        }
    };

    let ast_display = serde_json::to_string_pretty(&program)
        .unwrap_or_else(|_| format!("{:#?}", program));

    let mut type_checker = TypeChecker::new();
    if let Err(errors) = type_checker.check_program(&program) {
        let error_ansi = render_semantic_errors(errors.clone(), source);
        let error_plain = errors.iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join("\n");
        return serde_json::to_string(&CompilationResult {
            success: false,
            output: None,
            error: Some(error_plain),
            error_ansi: Some(error_ansi),
            c_code: None,
            stats: OptimizationStats::default(),
            tokens: Some(token_info),
            ast: Some(ast_display),
        }).unwrap();
    }

    let mut optimizer = Optimizer::new(opt_level);
    let opt_stats = optimizer.optimize(&mut program);

    let mut codegen = CodeGenerator::new();
    let c_code = match codegen.generate(&program) {
        Ok(code) => code,
        Err(e) => {
            return serde_json::to_string(&CompilationResult {
                success: false,
                output: None,
                error: Some(format!("Code Generation Error: {}", e)),
                error_ansi: Some(format!("Code Generation Error: {}", e)),
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

    serde_json::to_string(&CompilationResult {
        success: true,
        output: Some(format!(
            "Compilation successful!\nOptimization level: {}\n\nDownload the C code and compile locally:\n  gcc program.c -o program\n  ./program",
            opt_level
        )),
        error: None,
        error_ansi: None,
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

#[wasm_bindgen]
pub fn analyze(source: &str) -> String {
    // Tokenize
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            let error_ansi = render_lexer_error(e.clone(), source);
            return serde_json::to_string(&AnalyzeResult {
                success: false,
                report: None,
                error: Some(format!("{:?}", e)),
                error_ansi: Some(error_ansi),
            }).unwrap();
        }
    };

    // Parse
    let mut parser = Parser::new(tokens, source.to_string());
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            let error_ansi = render_parser_error(e.clone(), source);
            return serde_json::to_string(&AnalyzeResult {
                success: false,
                report: None,
                error: Some(format!("{:?}", e)),
                error_ansi: Some(error_ansi),
            }).unwrap();
        }
    };

    // Type check — report errors but still analyze
    let mut type_checker = crate::TypeChecker::new();
    let type_error = if let Err(errors) = type_checker.check_program(&program) {
        Some(render_semantic_errors(errors, source))
    } else {
        None
    };

    // Analyze
    let report = crate::analyzer::analyze_program(&program, source);

    serde_json::to_string(&AnalyzeResult {
        success: true,
        report: Some(report),
        error: type_error.clone(),
        error_ansi: type_error,
    }).unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct AnalyzeResult {
    pub success: bool,
    pub report: Option<crate::analyzer::AnalysisReport>,
    pub error: Option<String>,
    pub error_ansi: Option<String>,
}