// src/analyzer/mod.rs - Static Analysis & Complexity Calculator

pub mod basic;
pub mod cyclomatic;
pub mod nesting;
pub mod cognitive;
pub mod halstead;
pub mod fanout;

use serde::{Serialize, Deserialize};
use crate::ast::Program;

/// Complete analysis report for a program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub functions: Vec<FunctionMetrics>,
    pub program_totals: ProgramMetrics,
}

/// Metrics for a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetrics {
    pub name: String,
    pub loc: usize,
    pub statement_count: usize,
    pub parameter_count: usize,
    pub cyclomatic_complexity: usize,
    pub max_nesting_depth: usize,
    pub cognitive_complexity: usize,
    pub halstead: HalsteadMetrics,
    pub fan_out: usize,
    pub rating: Rating,
}

/// Program-wide aggregate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetrics {
    pub total_functions: usize,
    pub total_loc: usize,
    pub total_statements: usize,
    pub avg_cyclomatic: f64,
    pub max_cyclomatic: usize,
    pub avg_cognitive: f64,
    pub max_cognitive: usize,
    pub overall_rating: Rating,
}

/// Halstead complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HalsteadMetrics {
    pub unique_operators: usize,
    pub unique_operands: usize,
    pub total_operators: usize,
    pub total_operands: usize,
    pub vocabulary: usize,
    pub length: usize,
    pub volume: f64,
    pub difficulty: f64,
    pub effort: f64,
}

/// Complexity rating
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Rating {
    A,
    B,
    C,
    D,
    F,
}

impl Default for Rating {
    fn default() -> Self {
        Rating::A
    }
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rating::A => write!(f, "A"),
            Rating::B => write!(f, "B"),
            Rating::C => write!(f, "C"),
            Rating::D => write!(f, "D"),
            Rating::F => write!(f, "F"),
        }
    }
}

impl Rating {
    /// Get a descriptive label for the rating
    pub fn label(&self) -> &str {
        match self {
            Rating::A => "Excellent",
            Rating::B => "Good",
            Rating::C => "Moderate",
            Rating::D => "Complex",
            Rating::F => "Very Complex",
        }
    }

    /// Get an emoji for the rating
    pub fn emoji(&self) -> &str {
        match self {
            Rating::A => "🟢",
            Rating::B => "🟡",
            Rating::C => "🟠",
            Rating::D => "🔴",
            Rating::F => "💀",
        }
    }

    /// Return the worse (higher) of two ratings
    pub fn worst(a: &Rating, b: &Rating) -> Rating {
        if a >= b { a.clone() } else { b.clone() }
    }
}

/// Compute a rating from cyclomatic complexity
fn rate_cyclomatic(cc: usize) -> Rating {
    match cc {
        0..=5 => Rating::A,
        6..=10 => Rating::B,
        11..=20 => Rating::C,
        21..=50 => Rating::D,
        _ => Rating::F,
    }
}

/// Compute a rating from cognitive complexity
fn rate_cognitive(cog: usize) -> Rating {
    match cog {
        0..=5 => Rating::A,
        6..=10 => Rating::B,
        11..=15 => Rating::C,
        16..=30 => Rating::D,
        _ => Rating::F,
    }
}

/// Compute overall rating for a function from its metrics
fn rate_function(func: &FunctionMetrics) -> Rating {
    let cc_rating = rate_cyclomatic(func.cyclomatic_complexity);
    let cog_rating = rate_cognitive(func.cognitive_complexity);
    Rating::worst(&cc_rating, &cog_rating)
}

/// Main entry point: analyze an entire program
pub fn analyze_program(program: &Program, source: &str) -> AnalysisReport {
    let mut functions = Vec::new();

    for func in &program.functions {
        let loc = basic::count_loc(func, source);
        let statement_count = basic::count_statements(&func.body);
        let parameter_count = func.params.len();
        let cyclomatic_complexity = cyclomatic::calculate(func);
        let max_nesting_depth = nesting::calculate(func);
        let cognitive_complexity = cognitive::calculate(func);
        let halstead = halstead::calculate(func);
        let fan_out = fanout::calculate(func);

        let mut metrics = FunctionMetrics {
            name: func.name.clone(),
            loc,
            statement_count,
            parameter_count,
            cyclomatic_complexity,
            max_nesting_depth,
            cognitive_complexity,
            halstead,
            fan_out,
            rating: Rating::default(),
        };

        metrics.rating = rate_function(&metrics);
        functions.push(metrics);
    }

    let program_totals = compute_program_totals(&functions);

    AnalysisReport {
        functions,
        program_totals,
    }
}

/// Compute aggregate metrics across all functions
fn compute_program_totals(functions: &[FunctionMetrics]) -> ProgramMetrics {
    let total_functions = functions.len();
    let total_loc: usize = functions.iter().map(|f| f.loc).sum();
    let total_statements: usize = functions.iter().map(|f| f.statement_count).sum();

    let avg_cyclomatic = if total_functions > 0 {
        functions.iter().map(|f| f.cyclomatic_complexity).sum::<usize>() as f64
            / total_functions as f64
    } else {
        0.0
    };
    let max_cyclomatic = functions.iter().map(|f| f.cyclomatic_complexity).max().unwrap_or(0);

    let avg_cognitive = if total_functions > 0 {
        functions.iter().map(|f| f.cognitive_complexity).sum::<usize>() as f64
            / total_functions as f64
    } else {
        0.0
    };
    let max_cognitive = functions.iter().map(|f| f.cognitive_complexity).max().unwrap_or(0);

    let overall_rating = functions.iter()
        .map(|f| &f.rating)
        .fold(Rating::A, |worst, r| Rating::worst(&worst, r));

    ProgramMetrics {
        total_functions,
        total_loc,
        total_statements,
        avg_cyclomatic,
        max_cyclomatic,
        avg_cognitive,
        max_cognitive,
        overall_rating,
    }
}

/// Pretty-print the analysis report to the terminal
pub fn display_report(report: &AnalysisReport) {
    println!("\n{}", "═".repeat(62));
    println!("  📊 Static Analysis Report");
    println!("{}", "═".repeat(62));

    for func in &report.functions {
        println!();
        println!("  {} {} Function: {}",
            func.rating.emoji(),
            func.rating,
            func.name,
        );
        println!("  {}", "─".repeat(40));

        // Basic counts
        println!("    Lines of code:     {}", func.loc);
        println!("    Statements:        {}", func.statement_count);
        println!("    Parameters:        {}", func.parameter_count);

        // Complexity
        println!("    Cyclomatic:        {}  {}", 
            func.cyclomatic_complexity,
            complexity_bar(func.cyclomatic_complexity, 20),
        );
        println!("    Cognitive:         {}  {}",
            func.cognitive_complexity,
            complexity_bar(func.cognitive_complexity, 20),
        );
        println!("    Max nesting:       {}", func.max_nesting_depth);

        // Halstead
        println!("    Halstead volume:   {:.1}", func.halstead.volume);
        println!("    Halstead effort:   {:.1}", func.halstead.effort);

        // Fan-out
        println!("    Fan-out:           {}", func.fan_out);

        // Rating
        println!("    Rating:            {} ({})", func.rating, func.rating.label());

        // Warnings for this function
        display_function_warnings(func);
    }

    // Program totals
    println!();
    println!("  {}", "─".repeat(40));
    println!("  📋 Program Summary");
    println!("  {}", "─".repeat(40));
    println!("    Functions:         {}", report.program_totals.total_functions);
    println!("    Total LOC:         {}", report.program_totals.total_loc);
    println!("    Total statements:  {}", report.program_totals.total_statements);
    println!("    Avg cyclomatic:    {:.1}", report.program_totals.avg_cyclomatic);
    println!("    Max cyclomatic:    {}", report.program_totals.max_cyclomatic);
    println!("    Avg cognitive:     {:.1}", report.program_totals.avg_cognitive);
    println!("    Max cognitive:     {}", report.program_totals.max_cognitive);
    println!("    Overall rating:    {} {} ({})",
        report.program_totals.overall_rating.emoji(),
        report.program_totals.overall_rating,
        report.program_totals.overall_rating.label(),
    );

    println!("\n{}", "═".repeat(62));
}

/// Generate a simple bar visualization for a complexity value
fn complexity_bar(value: usize, max_width: usize) -> String {
    // Scale: each block = ~2.5 units, cap at max_width
    let filled = (value).min(max_width);
    let bar_char = match value {
        0..=5 => '▪',
        6..=10 => '▪',
        11..=20 => '▪',
        _ => '▪',
    };
    let color = match value {
        0..=5 => "\x1b[32m",   // green
        6..=10 => "\x1b[33m",  // yellow
        11..=20 => "\x1b[33m", // orange-ish (yellow)
        _ => "\x1b[31m",       // red
    };
    let reset = "\x1b[0m";
    format!("{}{}{}{}", color, std::iter::repeat(bar_char).take(filled).collect::<String>(), reset,
        if value > max_width { format!(" ({})", value) } else { String::new() })
}

/// Display warnings/suggestions for a function
fn display_function_warnings(func: &FunctionMetrics) {
    let mut warnings: Vec<String> = Vec::new();

    if func.cyclomatic_complexity > 10 {
        warnings.push(format!(
            "High cyclomatic complexity ({}). Consider breaking this function into smaller ones.",
            func.cyclomatic_complexity,
        ));
    }

    if func.cognitive_complexity > 15 {
        warnings.push(format!(
            "High cognitive complexity ({}). This function may be hard to understand.",
            func.cognitive_complexity,
        ));
    }

    if func.max_nesting_depth > 3 {
        warnings.push(format!(
            "Deep nesting (depth {}). Consider using early returns or extracting helper functions.",
            func.max_nesting_depth,
        ));
    }

    if func.parameter_count > 5 {
        warnings.push(format!(
            "Too many parameters ({}). Consider grouping related parameters.",
            func.parameter_count,
        ));
    }

    if func.fan_out > 8 {
        warnings.push(format!(
            "High fan-out ({}). This function depends on many others.",
            func.fan_out,
        ));
    }

    if func.loc > 50 {
        warnings.push(format!(
            "Long function ({} LOC). Consider splitting into smaller functions.",
            func.loc,
        ));
    }

    if !warnings.is_empty() {
        for warning in &warnings {
            println!("    ⚠️  {}", warning);
        }
    }
}

// ==================== TESTS ====================

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn make_program(functions: Vec<Function>) -> Program {
        Program { functions }
    }

    fn make_function_with_name(name: &str, stmts: Vec<Statement>, span: Span) -> Function {
        Function {
            name: name.to_string(),
            params: vec![],
            return_type: None,
            body: Block {
                statements: stmts,
                span: span.clone(),
            },
            span,
        }
    }

    fn make_let(name: &str) -> Statement {
        Statement::Let(LetStmt {
            name: name.to_string(),
            typ: Type::Int,
            value: Some(Expression::Literal(LiteralExpr {
                value: Literal::Integer(0),
                span: Span::default(),
            })),
            span: Span::default(),
        })
    }

    fn make_bool_literal(val: bool) -> Expression {
        Expression::Literal(LiteralExpr {
            value: Literal::Boolean(val),
            span: Span::default(),
        })
    }

    fn make_if(then_stmts: Vec<Statement>) -> Statement {
        Statement::If(IfStmt {
            condition: make_bool_literal(true),
            then_block: Block {
                statements: then_stmts,
                span: Span::default(),
            },
            else_block: None,
            span: Span::default(),
        })
    }

    fn make_while(body_stmts: Vec<Statement>) -> Statement {
        Statement::While(WhileStmt {
            condition: make_bool_literal(true),
            body: Block {
                statements: body_stmts,
                span: Span::default(),
            },
            span: Span::default(),
        })
    }

    // ---- Rating tests ----

    #[test]
    fn test_rate_cyclomatic_a() {
        assert_eq!(rate_cyclomatic(1), Rating::A);
        assert_eq!(rate_cyclomatic(5), Rating::A);
    }

    #[test]
    fn test_rate_cyclomatic_b() {
        assert_eq!(rate_cyclomatic(6), Rating::B);
        assert_eq!(rate_cyclomatic(10), Rating::B);
    }

    #[test]
    fn test_rate_cyclomatic_c() {
        assert_eq!(rate_cyclomatic(11), Rating::C);
        assert_eq!(rate_cyclomatic(20), Rating::C);
    }

    #[test]
    fn test_rate_cyclomatic_d() {
        assert_eq!(rate_cyclomatic(21), Rating::D);
        assert_eq!(rate_cyclomatic(50), Rating::D);
    }

    #[test]
    fn test_rate_cyclomatic_f() {
        assert_eq!(rate_cyclomatic(51), Rating::F);
        assert_eq!(rate_cyclomatic(100), Rating::F);
    }

    #[test]
    fn test_rate_cognitive_a() {
        assert_eq!(rate_cognitive(0), Rating::A);
        assert_eq!(rate_cognitive(5), Rating::A);
    }

    #[test]
    fn test_rate_cognitive_b() {
        assert_eq!(rate_cognitive(6), Rating::B);
        assert_eq!(rate_cognitive(10), Rating::B);
    }

    #[test]
    fn test_rate_cognitive_c() {
        assert_eq!(rate_cognitive(11), Rating::C);
        assert_eq!(rate_cognitive(15), Rating::C);
    }

    #[test]
    fn test_rate_cognitive_d() {
        assert_eq!(rate_cognitive(16), Rating::D);
        assert_eq!(rate_cognitive(30), Rating::D);
    }

    #[test]
    fn test_rate_cognitive_f() {
        assert_eq!(rate_cognitive(31), Rating::F);
    }

    #[test]
    fn test_worst_rating() {
        assert_eq!(Rating::worst(&Rating::A, &Rating::A), Rating::A);
        assert_eq!(Rating::worst(&Rating::A, &Rating::F), Rating::F);
        assert_eq!(Rating::worst(&Rating::B, &Rating::C), Rating::C);
        assert_eq!(Rating::worst(&Rating::D, &Rating::B), Rating::D);
    }

    // ---- Integration tests ----

    #[test]
    fn test_analyze_empty_program() {
        let source = "";
        let program = make_program(vec![]);
        let report = analyze_program(&program, source);

        assert_eq!(report.functions.len(), 0);
        assert_eq!(report.program_totals.total_functions, 0);
        assert_eq!(report.program_totals.total_loc, 0);
        assert_eq!(report.program_totals.overall_rating, Rating::A);
    }

    #[test]
    fn test_analyze_simple_function() {
        let source = "func main() {\n    let x: int = 0;\n}\n";
        let func = make_function_with_name("main",
            vec![make_let("x")],
            Span::new(0, source.len()),
        );
        let program = make_program(vec![func]);
        let report = analyze_program(&program, source);

        assert_eq!(report.functions.len(), 1);
        let f = &report.functions[0];
        assert_eq!(f.name, "main");
        assert_eq!(f.statement_count, 1);
        assert_eq!(f.cyclomatic_complexity, 1);
        assert_eq!(f.cognitive_complexity, 0);
        assert_eq!(f.max_nesting_depth, 0);
        assert_eq!(f.fan_out, 0);
        assert_eq!(f.rating, Rating::A);
    }

    #[test]
    fn test_analyze_multiple_functions() {
        let source = "func a() {\n}\nfunc b() {\n}\n";
        let func_a = make_function_with_name("a", vec![], Span::new(0, 14));
        let func_b = make_function_with_name("b", vec![], Span::new(15, source.len()));
        let program = make_program(vec![func_a, func_b]);
        let report = analyze_program(&program, source);

        assert_eq!(report.functions.len(), 2);
        assert_eq!(report.program_totals.total_functions, 2);
    }

    #[test]
    fn test_overall_rating_worst_of_all() {
        // One simple function (A) + one with nested ifs (worse rating)
        let source = "func a() {\n}\nfunc b() {\n}\n";

        // func b has many ifs → higher cyclomatic
        let many_ifs: Vec<Statement> = (0..12).map(|i| {
            make_if(vec![make_let(&format!("x{}", i))])
        }).collect();

        let func_a = make_function_with_name("a", vec![make_let("x")], Span::new(0, 14));
        let func_b = make_function_with_name("b", many_ifs, Span::new(15, source.len()));
        let program = make_program(vec![func_a, func_b]);
        let report = analyze_program(&program, source);

        // func_b has cyclomatic = 13 → Rating C
        assert_eq!(report.functions[1].rating, Rating::C);
        // overall should be C (worst)
        assert_eq!(report.program_totals.overall_rating, Rating::C);
    }

    #[test]
    fn test_averages() {
        let source = "func a() {\n}\nfunc b() {\n}\n";
        // func_a: 1 if → cyclomatic 2, cognitive 1
        // func_b: 2 ifs → cyclomatic 3, cognitive 2
        let func_a = make_function_with_name("a",
            vec![make_if(vec![make_let("x")])],
            Span::new(0, 14),
        );
        let func_b = make_function_with_name("b",
            vec![
                make_if(vec![make_let("y")]),
                make_if(vec![make_let("z")]),
            ],
            Span::new(15, source.len()),
        );
        let program = make_program(vec![func_a, func_b]);
        let report = analyze_program(&program, source);

        // avg cyclomatic = (2 + 3) / 2 = 2.5
        assert!((report.program_totals.avg_cyclomatic - 2.5).abs() < 0.001);
        assert_eq!(report.program_totals.max_cyclomatic, 3);

        // avg cognitive = (1 + 2) / 2 = 1.5
        assert!((report.program_totals.avg_cognitive - 1.5).abs() < 0.001);
        assert_eq!(report.program_totals.max_cognitive, 2);
    }

    #[test]
    fn test_rating_display() {
        assert_eq!(format!("{}", Rating::A), "A");
        assert_eq!(Rating::A.label(), "Excellent");
        assert_eq!(Rating::F.label(), "Very Complex");
        assert_eq!(Rating::A.emoji(), "🟢");
        assert_eq!(Rating::D.emoji(), "🔴");
        assert_eq!(Rating::F.emoji(), "💀");
    }
}