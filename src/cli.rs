// src/cli.rs - Subcommand structure

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// MiniLang Compiler - A compiler for the MiniLang programming language
#[derive(Parser, Debug)]
#[command(name = "minilang")]
#[command(version = "0.1.0")]
#[command(about = "MiniLang compiler and toolchain")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short = 'o', long = "output", global = true)]
    pub output: Option<String>,
    
    #[arg(long = "keep-c", global = true)]
    pub keep_c: bool,
    
    #[arg(short = 'd', long = "detail", global = true)]
    pub detail: bool,

    #[arg(short = 'O', long = "opt", default_value = "1", global = true)]
    pub optimization: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compile source file to executable
    Compile {
        file: PathBuf,
        
        #[arg(long = "to-c")]
        to_c: bool,
    },
    
    /// Compile and run the program
    Run {
        file: PathBuf,
    },

    /// Check for compilation errors without generating code
    Check {
        file: PathBuf,
    },

    /// Display the Abstract Syntax Tree
    Ast {
        file: PathBuf,
    },

    /// Display all tokens from lexical analysis
    Tokens {
        file: PathBuf,
    },

    /// Show compilation statistics
    Stats {
        file: PathBuf,

        #[arg(long = "time")]
        show_time: bool,
    },

    /// Clean generated files
    Clean {
        #[arg(default_value = ".")]
        directory: PathBuf,
        
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Run static analysis and complexity metrics
    Analyze {
        file: PathBuf,

        /// Output as JSON instead of formatted text
        #[arg(long = "json")]
        json: bool,
    },
}