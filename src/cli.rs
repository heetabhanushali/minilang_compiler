// src/cli.rs - Subcommand structure

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// MiniLang Compiler - A compiler for the MiniLang programming language
#[derive(Parser, Debug)]
#[command(name = "minilang")]
#[command(version = "0.1.0")]
#[command(about = "MiniLang compiler and toolchain")]
#[command(long_about = r#"
MiniLang compiler and toolchain

Usage: minilang [OPTIONS] [FILE] [COMMAND]

Commands:
  compile  Compile source file to executable
  run      Compile and run the program
  check    Check for compilation errors without generating code
  ast      Display the Abstract Syntax Tree
  tokens   Display all tokens from lexical analysis
  stats    Show compilation statistics
  clean    Clean generated files
  watch    Watch file for changes and auto-recompile
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [FILE]  Source file (when no subcommand is specified)

Options:
  -o, --output <OUTPUT>      Custom name for the output executable
      --keep-c               Keep intermediate C file after compilation
  -d, --detail               Show detailed compilation steps
  -O, --opt <OPTIMIZATION>   Optimization level (0-3) [default: 1]
  -h, --help                 Print help
  -V, --version              Print version
"#)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    #[arg(short = 'o', long = "output", global = true)]
    pub output: Option<String>,
    
    #[arg(long = "keep-c", global = true)]
    pub keep_c: bool,
    
    #[arg(short = 'd', long = "detail", global = true)]
    pub detail: bool,

    // 0 = none, 1= basic, 2=aggressive
    #[arg(short = 'O', long = "opt", default_value = "1", global = true)]
    pub optimization: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Compile {
        file: PathBuf,
        
        #[arg(long = "to-c")]
        to_c: bool,
    },
    
    Run {
        file: PathBuf,
    },
    Check{
        file: PathBuf,
    },
    Ast {
        file: PathBuf,
    },
    Tokens {
        file: PathBuf,
    },
    Stats {
        file: PathBuf,

        #[arg(long = "time")]
        show_time: bool,
    },
    Clean {
        #[arg(default_value = ".")]
        directory: PathBuf,
        
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}