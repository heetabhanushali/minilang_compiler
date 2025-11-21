// lib.rs - Module declarations and exports

pub mod lexer;
pub mod errors;
pub mod test_utils;
pub mod ast;
pub mod parser;
pub mod symbol_table;
pub mod type_checker;
pub mod codegen;
pub mod cli;
pub mod optimizer;
pub mod wasm;

// Re-export main types for easier use
pub use errors::{CompilerError, LexerError, ParserError, SemanticError};
pub use lexer::{Token, Lexer, TokenWithSpan};
pub use ast::{Program, Function, Statement, Expression, Literal, Type, BinaryOp, UnaryOp, StringPart, OptimizationHint};
pub use parser::Parser;
pub use symbol_table::{SymbolTable, Symbol, SymbolType};
pub use type_checker::TypeChecker;
pub use codegen::CodeGenerator;
pub use optimizer::{Optimizer, OptimizationStats};