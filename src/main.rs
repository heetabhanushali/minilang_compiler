// src/main.rs - With subcommand support

use minilang_compiler::{
    Lexer, LexerError, 
    Parser, ParserError, 
    TypeChecker, SemanticError,
    CodeGenerator,
    Optimizer,
    cli::{Cli, Commands}
};
use clap::Parser as ClapParser;
use miette::{NamedSource, Report};
use std::{fs, time::{Instant}};
use std::process::{self, Command};
use std::path::{Path, PathBuf};

fn main() {
    let args = Cli::parse();
    
    // Determine which command to run
    match &args.command {  // Borrow instead of move
        Some(Commands::Compile { file, to_c }) => {
            // Explicit compile command
            handle_compile(file, &args, *to_c, false);  // *to_c to dereference
        }
        Some(Commands::Run { file }) => {
            // Run command
            handle_compile(file, &args, false, true);
        }
        Some(Commands::Check { file }) => {
            handle_check(file);
        }
        Some(Commands::Ast { file }) => {
            handle_ast(file);
        }
        Some(Commands::Tokens { file }) => {
            handle_tokens(file);
        }
        Some(Commands::Stats { file, show_time }) => {
            handle_stats(file, *show_time);
        }
        Some(Commands::Clean { directory, dry_run }) => {
            handle_clean(directory, *dry_run);
        }
        None => {
            // No subcommand - check if file was provided as positional arg
            if let Some(ref file) = args.file {  // Use ref here
                // Default behavior: compile
                handle_compile(file, &args, false, false);
            } else {
                // No file provided
                eprintln!("Error: No input file provided");
                eprintln!("Usage: minilang <FILE> or minilang <COMMAND> <FILE>");
                eprintln!("Try 'minilang --help' for more information");
                process::exit(1);
            }
        }
    }
}

fn handle_compile(file: &PathBuf, args: &Cli, to_c_only: bool, should_run: bool) {
    // Check if file exists
    if !file.exists() {
        eprintln!("❌ Error: File '{}' not found", file.display());
        process::exit(1);
    }
    
    // Read source code
    let source = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
            process::exit(1);
        }
    };
    
    let filename = file.to_str().unwrap_or("unknown.mini");
    
    // Compile
    compile_source(&source, filename, file, args, to_c_only, should_run);
}

fn determine_output_path(file: &PathBuf, custom_name: &Option<String>) -> PathBuf {
    let source_dir = file.parent().unwrap_or(Path::new("."));
    
    if let Some(ref name) = custom_name {
        source_dir.join(name)
    } else {
        file.with_extension("")
    }
}


fn handle_check(file: &PathBuf) {
    if !file.exists() {
        eprintln!("❌ Error: File '{}' not found", file.display());
        process::exit(1);
    }
    
    let source = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
            process::exit(1);
        }
    };
    
    let filename = file.to_str().unwrap_or("unknown.mini");
    
    println!("Checking: {}", file.display());
    println!("{}", "=".repeat(50));
    
    // Lexical analysis
    print!("Lexer........... ");
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => {
            println!("✅");
            tokens
        }
        Err(e) => {
            println!("❌");
            display_beautiful_error_lexer(e, &source, filename);
            process::exit(1);
        }
    };
    
    // Parsing
    print!("Parser.......... ");
    let mut parser = Parser::new(tokens, source.to_string());
    let program = match parser.parse_program() {
        Ok(prog) => {
            println!("✅");
            prog
        }
        Err(e) => {
            println!("❌");
            display_beautiful_error_parser(e, &source, filename);
            process::exit(1);
        }
    };
    
    // Type checking
    print!("Type Checker.... ");
    let mut type_checker = TypeChecker::new();
    match type_checker.check_program(&program) {
        Ok(()) => {
            println!("✅");
            let warnings = type_checker.get_warnings();
            if !warnings.is_empty() {
                println!("\n⚠️  {} warning(s) found:", warnings.len());
                for warning in warnings {
                    warning.display(&source, filename);
                }
            }
        }
        Err(errors) => {
            println!("❌");
            display_beautiful_error_semantic(errors, &source, filename);
            process::exit(1);
        }
    }
    
    println!("\n✅ All checks passed! No errors found.");
}

fn handle_ast(file: &PathBuf) {
    if !file.exists() {
        eprintln!("❌ Error: File '{}' not found", file.display());
        process::exit(1);
    }
    
    let source = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
            process::exit(1);
        }
    };
    
    let filename = file.to_str().unwrap_or("unknown.mini");
    
    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            display_beautiful_error_lexer(e, &source, filename);
            process::exit(1);
        }
    };
    
    // Parse
    let mut parser = Parser::new(tokens, source.to_string());
    let program = match parser.parse_program() {
        Ok(prog) => prog,
        Err(e) => {
            display_beautiful_error_parser(e, &source, filename);
            process::exit(1);
        }
    };
    
    // Display the AST
    program.display_tree();
}

fn handle_tokens(file: &PathBuf) {
    if !file.exists() {
        eprintln!("❌ Error: File '{}' not found", file.display());
        process::exit(1);
    }
    
    let source = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
            process::exit(1);
        }
    };
    
    let filename = file.to_str().unwrap_or("unknown.mini");
    
    println!("Tokens for: {}", file.display());
    println!("{}", "=".repeat(60));
    
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            display_beautiful_error_lexer(e, &source, filename);
            process::exit(1);
        }
    };
    
    println!("Total tokens: {}\n", tokens.len());
    
    // Display tokens with their position
    for (i, token_with_span) in tokens.iter().enumerate() {
        let line_num = source[..token_with_span.span.start]
            .chars()
            .filter(|&c| c == '\n')
            .count() + 1;
        
        println!("{:4} | Line {:3} | {:?}", 
            i + 1, 
            line_num, 
            token_with_span.token
        );
    }
}

fn handle_stats(file: &PathBuf, show_time: bool) {
    if !file.exists() {
        eprintln!("❌ Error: File '{}' not found", file.display());
        process::exit(1);
    }
    
    let source = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
            process::exit(1);
        }
    };
    
    let filename = file.to_str().unwrap_or("unknown.mini");
    
    println!("Statistics for: {}", file.display());
    println!("{}", "=".repeat(60));
    
    // Source stats
    println!("\nSource File:");
    println!("   Lines of code: {}", source.lines().count());
    println!("   Characters: {}", source.len());
    println!("   Non-empty lines: {}", source.lines().filter(|l| !l.trim().is_empty()).count());
    
    // Lexer stats
    let start = Instant::now();
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            display_beautiful_error_lexer(e, &source, filename);
            process::exit(1);
        }
    };
    let lex_time = start.elapsed();
    
    println!("\nTokens:");
    println!("   Total: {}", tokens.len());
    
    // Count token types
    let keywords = tokens.iter().filter(|t| matches!(
        t.token,
        minilang_compiler::Token::Let | minilang_compiler::Token::Func |
        minilang_compiler::Token::If | minilang_compiler::Token::While |
        minilang_compiler::Token::For | minilang_compiler::Token::Display |
        minilang_compiler::Token::Send | minilang_compiler::Token::Do
    )).count();
    
    let identifiers = tokens.iter().filter(|t| matches!(
        t.token,
        minilang_compiler::Token::Identifier(_)
    )).count();
    
    let literals = tokens.iter().filter(|t| matches!(
        t.token,
        minilang_compiler::Token::Integer(_) | 
        minilang_compiler::Token::Float(_) |
        minilang_compiler::Token::String(_) |
        minilang_compiler::Token::True |
        minilang_compiler::Token::False
    )).count();
    
    println!("   Keywords: {}", keywords);
    println!("   Identifiers: {}", identifiers);
    println!("   Literals: {}", literals);
    
    // Parser stats
    let start = Instant::now();
    let mut parser = Parser::new(tokens, source.to_string());
    let program = match parser.parse_program() {
        Ok(prog) => prog,
        Err(e) => {
            display_beautiful_error_parser(e, &source, filename);
            process::exit(1);
        }
    };
    let parse_time = start.elapsed();
    
    println!("\nAbstract Syntax Tree:");
    println!("   Functions: {}", program.functions.len());
    
    let total_stmts: usize = program.functions.iter()
        .map(|f| f.body.statements.len())
        .sum();
    println!("   Total statements: {}", total_stmts);
    
    for func in &program.functions {
        println!("   • {} ({} params, {} statements)", 
            func.name, 
            func.params.len(),
            func.body.statements.len()
        );
    }
    
    // Type checking stats
    let start = Instant::now();
    let mut type_checker = TypeChecker::new();
    let type_check_result = type_checker.check_program(&program);
    let type_time = start.elapsed();
    
    println!("\n✅ Type Checking:");
    match type_check_result {
        Ok(()) => {
            println!("   Status: Passed");
            let warnings = type_checker.get_warnings();
            println!("   Warnings: {}", warnings.len());
        }
        Err(errors) => {
            println!("   Status: Failed");
            println!("   Errors: {}", errors.len());
        }
    }
    
    // Code generation stats
    let start = Instant::now();
    let mut codegen = CodeGenerator::new();
    if let Ok(c_code) = codegen.generate(&program) {
        let gen_time = start.elapsed();
        
        println!("\nCode Generation:");
        println!("   C code lines: {}", c_code.lines().count());
        println!("   C code size: {} bytes", c_code.len());
        
        if show_time {
            println!("\nCompilation Times:");
            println!("   Lexer:        {:?}", lex_time);
            println!("   Parser:       {:?}", parse_time);
            println!("   Type Check:   {:?}", type_time);
            println!("   Code Gen:     {:?}", gen_time);
            println!("   Total:        {:?}", lex_time + parse_time + type_time + gen_time);
        }
    }
}

fn handle_clean(directory: &PathBuf, dry_run: bool) {
    println!("Cleaning generated files in: {}", directory.display());
    println!("{}", "=".repeat(60));
    
    if !directory.exists() {
        eprintln!("❌ Error: Directory '{}' not found", directory.display());
        process::exit(1);
    }
    
    let mut files_to_delete = Vec::new();
    let mut total_size = 0u64;
    
    // Find all .c files and executables
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let should_delete = 
                    // .c files
                    path.extension() == Some(std::ffi::OsStr::new("c")) ||
                    // Files without extension (likely executables)
                    (path.extension().is_none() && 
                     path.file_stem().is_some() &&
                     // Check if there's a corresponding .mini file
                     directory.join(format!("{}.mini", 
                         path.file_stem().unwrap().to_str().unwrap())).exists());
                
                if should_delete {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                    files_to_delete.push(path);
                }
            }
        }
    }
    
    if files_to_delete.is_empty() {
        println!("✅ No generated files found. Directory is clean!");
        return;
    }
    
    println!("Found {} file(s) to clean ({} KB total):\n", 
        files_to_delete.len(), 
        total_size / 1024);
    
    for file in &files_to_delete {
        println!("   • {}", file.display());
    }
    
    if dry_run {
        println!("\nDry run - no files were deleted");
        println!("   Run without --dry-run to actually delete");
    } else {
        println!("\nDeleting files...");
        let mut deleted = 0;
        
        for file in files_to_delete {
            match fs::remove_file(&file) {
                Ok(()) => {
                    deleted += 1;
                    println!("   ✅ Deleted: {}", file.display());
                }
                Err(e) => {
                    println!("   ❌ Failed to delete {}: {}", file.display(), e);
                }
            }
        }
        
        println!("\n✅ Cleaned {} file(s)", deleted);
    }
}

fn compile_source(
    source: &str, 
    filename: &str, 
    file: &PathBuf,
    args: &Cli,
    to_c_only: bool,
    should_run: bool,
) {
    // Show compilation steps only if detail flag is set
    let show_details = args.detail;

    if show_details {
        println!("\nCompiling");
        println!("{}", "=".repeat(60));
    }
    
    // STEP 1: Lexical Analysis
    if show_details {
        println!("\n_______________________________________");
        println!("Lexer: Tokenizing source code...");
    }
    
    let mut lexer = Lexer::new(source);
    
    let tokens = match lexer.tokenize() {
        Ok(tokens) => {
            if show_details {
                println!("   ✅ Successfully tokenized!");
                println!("   Found {} tokens", tokens.len());
                
                // Show token breakdown
                let keywords = tokens.iter().filter(|t| matches!(
                    t.token,
                    minilang_compiler::Token::Let | minilang_compiler::Token::Func |
                    minilang_compiler::Token::If | minilang_compiler::Token::While |
                    minilang_compiler::Token::For | minilang_compiler::Token::Display
                )).count();
                
                let identifiers = tokens.iter().filter(|t| matches!(
                    t.token,
                    minilang_compiler::Token::Identifier(_)
                )).count();
                
                println!("   {} keywords, {} identifiers", keywords, identifiers);
            }
            tokens
        }
        Err(e) => {
            display_beautiful_error_lexer(e, source, filename);
            process::exit(1);
        }
    };
    
    // STEP 2: Parsing
    if show_details {
        println!("\n_______________________________________");
        println!("Parser: Building Abstract Syntax Tree...");
    }
    
    let mut parser = Parser::new(tokens, source.to_string());
    
    let mut program = match parser.parse_program() {
        Ok(prog) => {
            if show_details {
                println!("   ✅ Successfully parsed!");
                println!("   Found {} function(s)", prog.functions.len());
                
                // Show function details
                for func in &prog.functions {
                    let return_type = if let Some(ref rt) = func.return_type {
                        format!(" -> {:?}", rt)
                    } else {
                        String::from(" (void)")
                    };
                    
                    println!("      • {}({} params){}", 
                        func.name, 
                        func.params.len(),
                        return_type
                    );
                }
                
                // Count total statements
                let total_stmts: usize = prog.functions.iter()
                    .map(|f| f.body.statements.len())
                    .sum();
                println!("   Total statements: {}", total_stmts);
            }
            prog
        }
        Err(e) => {
            display_beautiful_error_parser(e, source, filename);
            process::exit(1);
        }
    };
    
    // STEP 3: Semantic Analysis
    if show_details {
        println!("\n_______________________________________");
        println!("Semantic Analyzer: Type checking...");
    }
    
    let mut type_checker = TypeChecker::new();
    
    match type_checker.check_program(&program) {
        Ok(()) => {
            if show_details {
                println!("   ✅ Type checking passed!");
                
                let warnings = type_checker.get_warnings();
                if warnings.is_empty() {
                    println!("   No type errors or warnings");
                } else {
                    println!("   ⚠️ {} warning(s) found", warnings.len());
                    for warning in warnings {
                        warning.display(source, filename);
                    }
                }
            }
        }
        Err(errors) => {
            display_beautiful_error_semantic(errors, source, filename);
            process::exit(1);
        }
    }

    // STEP 3.5: OPTIMIZATION
    if args.optimization > 0 {
        if show_details {
            println!("\n_______________________________________");
            println!("Optimizer: Running optimization passes (level {})...", args.optimization);
        }
        
        let mut optimizer = Optimizer::new(args.optimization);
        let opt_stats = optimizer.optimize(&mut program);
        
        if show_details {
            println!("  ✅ Optimization complete!");
            if opt_stats.constants_folded > 0 {
                println!("   Constants folded: {}", opt_stats.constants_folded);
            }
            if opt_stats.dead_code_removed > 0 {
                println!("   Dead code removed: {}", opt_stats.dead_code_removed);
            }
            if opt_stats.constants_propagated > 0 {
                println!("   Constants propagated: {}", opt_stats.constants_propagated);
            }
            if opt_stats.strength_reductions > 0 {
                println!("   Strength reductions: {}", opt_stats.strength_reductions);
            }
            
            if opt_stats.constants_folded == 0 && 
               opt_stats.dead_code_removed == 0 && 
               opt_stats.constants_propagated == 0 &&
               opt_stats.strength_reductions == 0 {
                println!("  No optimizations applied");
            }
        }
    } else if show_details {
        println!("\n_______________________________________");
        println!("Optimizer: Skipped (optimization level 0)");
    }
    
    // STEP 4: Code Generation
    if show_details {
        println!("\n_______________________________________");
        println!("Code Generator: Generating C code...");
    }
    
    let mut codegen = CodeGenerator::new();
    
    // FIXED: Declare c_code outside the match
    let c_code = match codegen.generate(&program) {
        Ok(code) => code,  // Just return the code
        Err(e) => {
            eprintln!("❌ Code generation failed: {}", e);
            process::exit(1);
        }
    };
    
    // Now show details and save (c_code is in scope here)
    if show_details {
        println!("   ✅ C code generated successfully!");
        println!("   {} lines of C code", c_code.lines().count());
        
        // Show included headers count
        let headers: Vec<&str> = c_code.lines()
            .filter(|line| line.starts_with("#include"))
            .collect();
        println!("   {} system headers included", headers.len());
    }
    
    // Save C file
    let c_output_path = file.with_extension("c");
    
    if let Err(e) = fs::write(&c_output_path, &c_code) {
        eprintln!("❌ Failed to save C code: {}", e);
        process::exit(1);
    }
    
    if show_details && args.keep_c {
        println!("   Saved to: {}", c_output_path.display());
    }
    
    // If --to-c, stop here
    if to_c_only {
        println!("\n✅ Conversion to C successful!");
        println!("   Output: {}", c_output_path.display());
        return;
    }
    
    // STEP 5: Native Compilation
    if show_details {
        println!("\n_______________________________________");
        println!("GCC: Compiling to native executable...");
    }
    
    let exe_output_path = determine_output_path(file, &args.output);
    
    let gcc_result = Command::new("gcc")
        .arg(&c_output_path)
        .arg("-o")
        .arg(&exe_output_path)
        .arg("-std=c99")
        .arg("-Wall")
        .arg("-O2")
        .output();
    
    match gcc_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("❌ GCC compilation failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                process::exit(1);
            }
            
            if show_details {
                println!("   ✅ Native compilation successful!");
                
                // Show file size if possible
                if let Ok(metadata) = fs::metadata(&exe_output_path) {
                    let size_kb = metadata.len() / 1024;
                    println!("   Executable size: {} KB", size_kb);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Could not run GCC: {}", e);
            eprintln!("   Make sure GCC is installed: gcc --version");
            process::exit(1);
        }
    }
    
    // Clean up C file unless --keep-c
    if !args.keep_c {
        if let Err(_) = fs::remove_file(&c_output_path) {
            if show_details {
                println!("   ⚠️  Could not delete temporary C file");
            }
        }
    } 
    
    // If this is a run command, execute the program
    if should_run {
        if show_details {
            println!("\n Running");
            println!("{}", "=".repeat(60));
        }
        
        // Ensure we can find the executable
        let exec_path = if exe_output_path.parent().is_none() || exe_output_path.parent() == Some(Path::new("")) {
            // If no directory specified, use ./
            Path::new("./").join(&exe_output_path)
        } else {
            exe_output_path.clone()
        };
        
        let run_result = Command::new(&exec_path).output();
        
        match run_result {
            Ok(output) => {
                // Print program output
                print!("{}", String::from_utf8_lossy(&output.stdout));
                
                if !output.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&output.stderr));
                }
                
                // Exit with program's exit code
                if !output.status.success() {
                    process::exit(output.status.code().unwrap_or(1));
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to run program: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Just compiling - show final success message
        println!("\n✅ Compilation successful!");
        if !to_c_only{
            println!("   Executable: {}", exe_output_path.display());
        }
        if args.keep_c || to_c_only{
            println!("   C File: {}", c_output_path.display());
        }

    }
}

fn display_beautiful_error_lexer(error: LexerError, source: &str, filename: &str) {
    let named_source = NamedSource::new(filename, source.to_string());
    let report = Report::from(error).with_source_code(named_source);
    eprintln!("{:?}", report);
}

fn display_beautiful_error_parser(error: ParserError, source: &str, filename: &str) {
    let named_source = NamedSource::new(filename, source.to_string());
    let report = Report::from(error).with_source_code(named_source);
    eprintln!("{:?}", report);
}

fn display_beautiful_error_semantic(errors: Vec<SemanticError>, source: &str, filename: &str) {
    let named_source = NamedSource::new(filename, source.to_string());
    for error in errors {
        let report = Report::from(error).with_source_code(named_source.clone());
        eprintln!("{:?}\n", report);
    }
}