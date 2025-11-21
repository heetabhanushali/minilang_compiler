// src/errors.rs - Production-ready error types 

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Main compiler error type
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum CompilerError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lexer(#[from] LexerError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Semantic(#[from] SemanticError),
}

/// Lexer-specific errors with beautiful diagnostics
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum LexerError {
    #[error("unexpected character '{char}'")]
    #[diagnostic(
        code(minilang::lexer::unexpected_char),
        help("Valid characters include letters, digits, and operators (+, -, *, /, etc.)\nThe character '{char}' is not recognized by MiniLang")
    )]
    UnexpectedChar {
        char: char,
        #[label("unexpected character here")]
        span: SourceSpan,
    },
    
    #[error("unterminated string literal")]
    #[diagnostic(
        code(minilang::lexer::unterminated_string),
        help("Strings must be closed with a matching double quote (\")\nExample: \"Hello, World!\"")
    )]
    UnterminatedString {
        #[label("string starts here but never ends")]
        start: SourceSpan,
    },
    
    #[error("invalid number format")]
    #[diagnostic(
        code(minilang::lexer::invalid_number),
        help("Numbers should be integers (42, -13) or floats (3.14, -2.5)")
    )]
    InvalidNumber {
        #[label("invalid number format here")]
        span: SourceSpan,
    },

    #[error("integer literal out of range: '{value}'")]
    #[diagnostic(
        code(minilang::lexer::integer_overflow),
        help("Integer literals must be between -2147483648 and 2147483647")
    )]
    IntegerOverflow {
        value: String,
        #[label("this number is too large")]
        span: SourceSpan,
    },
}

/// Parser-specific errors
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum ParserError {
    #[error("unexpected token")]
    #[diagnostic(
        code(minilang::parser::unexpected_token),
        help("Expected {expected}, but found {found}")
    )]
    UnexpectedToken {
        expected: String,
        found: String,
        #[label("unexpected token here")]
        span: SourceSpan,
    },
    
    #[error("missing semicolon")]
    #[diagnostic(
        code(minilang::parser::missing_semicolon),
        help("Add a semicolon ';' at the end of the statement")
    )]
    MissingSemicolon {
        #[label("semicolon expected here")]
        span: SourceSpan,
    },
    
    #[error("missing closing brace")]
    #[diagnostic(
        code(minilang::parser::missing_brace),
        help("Add a closing brace '}}' to match the opening brace")
    )]
    MissingClosingBrace {
        #[label("closing brace expected")]
        span: SourceSpan,
    },
    
    #[error("invalid expression")]
    #[diagnostic(
        code(minilang::parser::invalid_expression),
        help("This doesn't look like a valid expression")
    )]
    InvalidExpression {
        #[label("invalid expression")]
        span: SourceSpan,
    },
    
    #[error("missing type annotation")]
    #[diagnostic(
        code(minilang::parser::missing_type),
        help("Variables require type annotations: let name: type = value;")
    )]
    MissingType {
        #[label("type annotation expected here")]
        span: SourceSpan,
    },
    
    #[error("unexpected end of input")]
    #[diagnostic(
        code(minilang::parser::unexpected_eof),
        help("The program ended unexpectedly. Check for missing closing braces or incomplete statements.")
    )]
    UnexpectedEof {
        expected: String,
    },
}


/// Semantic analysis errors
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum SemanticError {
    #[error("undefined variable '{name}'{}", context.as_ref().map(|c| format!(" in {}", c)).unwrap_or_default())]
    #[diagnostic(
        code(minilang::semantic::undefined_variable),
        help("{suggestion}")
    )]
    UndefinedVariable {
        name: String,
        #[label("undefined here")]
        span: SourceSpan,
        suggestion: String,
        context: Option<String>,
    },
    
    #[error("type mismatch")]
    #[diagnostic(
        code(minilang::semantic::type_mismatch),
        help("Expected type {expected}, but found {found}")
    )]
    TypeMismatch {
        expected: String,
        found: String,
        #[label("type mismatch here")]
        span: SourceSpan,
    },
    
    #[error("variable already defined")]
    #[diagnostic(
        code(minilang::semantic::duplicate_definition),
        help("Variable '{name}' was already defined in this scope")
    )]
    DuplicateDefinition {
        name: String,
        #[label("redefined here")]
        span: SourceSpan,
        #[label("originally defined here")]
        original: SourceSpan,
    },
    
    #[error("undefined function '{name}'{}", context.as_ref().map(|c| format!(" in {}", c)).unwrap_or_default())]
    #[diagnostic(
        code(minilang::semantic::undefined_function),
        help("{suggestion}")
    )]
    UndefinedFunction {
        name: String,
        #[label("undefined function")]
        span: SourceSpan,
        suggestion: String,
        context: Option<String>,
    },
    
    #[error("wrong number of arguments")]
    #[diagnostic(
        code(minilang::semantic::argument_count),
        help("Function '{name}' expects {expected} arguments, but {found} were provided")
    )]
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        found: usize,
        #[label("called here")]
        span: SourceSpan,
    },
    
    #[error("missing return")]
    #[diagnostic(
        code(minilang::semantic::missing_return),
        help("Function '{name}' must return a value of type {return_type} on all code paths")
    )]
    MissingReturn {
        name: String,
        return_type: String,
        #[label("function declared here")]
        span: SourceSpan,
    },

    #[error("break/continue outside loop")]
    #[diagnostic(
        code(minilang::semantic::break_outside_loop),
        help("'{statement}' can only be used inside a loop (while, do-while, or for)")
    )]
    BreakOutsideLoop {
        statement: String,
        #[label("not inside a loop")]
        span: SourceSpan,
    },
}


/// Compiler warnings (non-fatal issues)
#[derive(Debug, Clone)]
pub enum CompilerWarning {
    UnusedVariable {
        name: String,
        span: SourceSpan,
        defined_at: SourceSpan,
    },
    
    UnreachableCode {
        span: SourceSpan,
        reason: String,
    },
    
    ShadowedVariable {
        name: String,
        span: SourceSpan,
        original_span: SourceSpan,
    },
}

impl CompilerWarning {
    pub fn display(&self, source: &str, filename: &str) {
        use miette::{NamedSource, Report, Diagnostic};
        use thiserror::Error;
        
        #[derive(Error, Debug, Diagnostic)]
        enum Warning {
            #[error("unused variable '{name}'")]
            #[diagnostic(
                code(minilang::warning::unused_variable),
                severity(warning),
                help("Consider removing this variable or using it")
            )]
            UnusedVariable {
                name: String,
                #[label("defined here but never used")]
                span: SourceSpan,
            },
            
            #[error("unreachable code")]
            #[diagnostic(
                code(minilang::warning::unreachable_code),
                severity(warning),
                help("{reason}")
            )]
            UnreachableCode {
                #[label("this code will never execute")]
                span: SourceSpan,
                reason: String,
            },
            
            #[error("variable '{name}' shadows previous declaration")]
            #[diagnostic(
                code(minilang::warning::shadowed_variable),
                severity(warning),
                help("Consider using a different name")
            )]
            ShadowedVariable {
                name: String,
                #[label("shadows here")]
                span: SourceSpan,
                #[label("original defined here")]
                original: SourceSpan,
            },
        }
        
        let warning = match self {
            CompilerWarning::UnusedVariable { name, span, .. } => {
                Warning::UnusedVariable {
                    name: name.clone(),
                    span: *span,
                }
            },
            CompilerWarning::UnreachableCode { span, reason } => {
                Warning::UnreachableCode {
                    span: *span,
                    reason: reason.clone(),
                }
            },
            CompilerWarning::ShadowedVariable { name, span, original_span } => {
                Warning::ShadowedVariable {
                    name: name.clone(),
                    span: *span,
                    original: *original_span,
                }
            },
        };
        
        let named_source = NamedSource::new(filename, source.to_string());
        let report = Report::from(warning).with_source_code(named_source);
        eprintln!("{:?}", report);
    }
}
