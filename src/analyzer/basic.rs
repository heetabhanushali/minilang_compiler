// src/analyzer/basic.rs - Basic counts: LOC, statement count, parameter count

use crate::ast::{Function, Block, Statement};

/// Count lines of code for a function using its span in the source
pub fn count_loc(func: &Function, source: &str) -> usize {
    let start = func.span.start.min(source.len());
    let end = func.span.end.min(source.len());

    if start >= end {
        return 0;
    }

    let slice = &source[start..end];

    slice.lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Count non-empty, non-comment-only lines
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .count()
}

/// Recursively count all statements in a block (including nested blocks)
pub fn count_statements(block: &Block) -> usize {
    let mut count = 0;

    for stmt in &block.statements {
        count += 1; // count this statement itself

        // also count statements inside nested blocks
        count += count_nested_statements(stmt);
    }

    count
}

/// Count statements nested inside a statement (if/while/for bodies, etc.)
fn count_nested_statements(stmt: &Statement) -> usize {
    match stmt {
        Statement::If(if_stmt) => {
            let mut count = count_statements(&if_stmt.then_block);
            if let Some(ref else_block) = if_stmt.else_block {
                count += count_statements(else_block);
            }
            count
        }
        Statement::While(while_stmt) => {
            count_statements(&while_stmt.body)
        }
        Statement::DoWhile(do_while_stmt) => {
            count_statements(&do_while_stmt.body)
        }
        Statement::For(for_stmt) => {
            count_statements(&for_stmt.body)
        }
        Statement::Block(block) => {
            count_statements(block)
        }
        // These statements have no nested blocks
        Statement::Let(_)
        | Statement::Display(_)
        | Statement::Return(_)
        | Statement::Expression(_)
        | Statement::Break(_)
        | Statement::Continue(_)
        | Statement::Const(_) => 0,
    }
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    /// Helper: create a minimal function with given body statements and a span
    fn make_function(name: &str, params: Vec<Parameter>, stmts: Vec<Statement>, span: Span) -> Function {
        Function {
            name: name.to_string(),
            params,
            return_type: None,
            body: Block {
                statements: stmts,
                span: span.clone(),
            },
            span,
        }
    }

    /// Helper: create a simple let statement
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

    /// Helper: create a simple display statement
    fn make_display() -> Statement {
        Statement::Display(DisplayStmt {
            expressions: vec![Expression::Literal(LiteralExpr {
                value: Literal::String("hello".to_string()),
                span: Span::default(),
            })],
            span: Span::default(),
        })
    }

    /// Helper: create an if statement with given then/else bodies
    fn make_if(then_stmts: Vec<Statement>, else_stmts: Option<Vec<Statement>>) -> Statement {
        Statement::If(IfStmt {
            condition: Expression::Literal(LiteralExpr {
                value: Literal::Boolean(true),
                span: Span::default(),
            }),
            then_block: Block {
                statements: then_stmts,
                span: Span::default(),
            },
            else_block: else_stmts.map(|stmts| Block {
                statements: stmts,
                span: Span::default(),
            }),
            span: Span::default(),
        })
    }

    /// Helper: create a while statement
    fn make_while(body_stmts: Vec<Statement>) -> Statement {
        Statement::While(WhileStmt {
            condition: Expression::Literal(LiteralExpr {
                value: Literal::Boolean(true),
                span: Span::default(),
            }),
            body: Block {
                statements: body_stmts,
                span: Span::default(),
            },
            span: Span::default(),
        })
    }

    // ---- LOC tests ----

    #[test]
    fn test_loc_simple_function() {
        let source = "func main() {\n    let x: int = 5;\n    display x;\n}\n";
        // span covers entire source
        let func = make_function("main", vec![], vec![], Span::new(0, source.len()));
        let loc = count_loc(&func, source);
        // 4 non-empty lines: func, let, display, }
        assert_eq!(loc, 4);
    }

    #[test]
    fn test_loc_with_blank_lines() {
        let source = "func main() {\n\n    let x: int = 5;\n\n    display x;\n\n}\n";
        let func = make_function("main", vec![], vec![], Span::new(0, source.len()));
        let loc = count_loc(&func, source);
        // blank lines are excluded
        assert_eq!(loc, 4);
    }

    #[test]
    fn test_loc_with_comments() {
        let source = "func main() {\n    // this is a comment\n    let x: int = 5;\n}\n";
        let func = make_function("main", vec![], vec![], Span::new(0, source.len()));
        let loc = count_loc(&func, source);
        // comment line excluded: func, let, }
        assert_eq!(loc, 3);
    }

    #[test]
    fn test_loc_empty_function() {
        let source = "func main() {\n}\n";
        let func = make_function("main", vec![], vec![], Span::new(0, source.len()));
        let loc = count_loc(&func, source);
        // func main() { and }
        assert_eq!(loc, 2);
    }

    // ---- Statement count tests ----

    #[test]
    fn test_count_empty_block() {
        let block = Block {
            statements: vec![],
            span: Span::default(),
        };
        assert_eq!(count_statements(&block), 0);
    }

    #[test]
    fn test_count_flat_statements() {
        let block = Block {
            statements: vec![
                make_let("x"),
                make_let("y"),
                make_display(),
            ],
            span: Span::default(),
        };
        assert_eq!(count_statements(&block), 3);
    }

    #[test]
    fn test_count_nested_if() {
        // if (true) { let x; let y; } else { display; }
        let block = Block {
            statements: vec![
                make_if(
                    vec![make_let("x"), make_let("y")],
                    Some(vec![make_display()]),
                ),
            ],
            span: Span::default(),
        };
        // 1 (the if) + 2 (then) + 1 (else) = 4
        assert_eq!(count_statements(&block), 4);
    }

    #[test]
    fn test_count_nested_while() {
        // while (true) { let x; display; }
        let block = Block {
            statements: vec![
                make_while(vec![make_let("x"), make_display()]),
            ],
            span: Span::default(),
        };
        // 1 (the while) + 2 (body) = 3
        assert_eq!(count_statements(&block), 3);
    }

    #[test]
    fn test_count_deeply_nested() {
        // while { if { let x; } }
        let block = Block {
            statements: vec![
                make_while(vec![
                    make_if(vec![make_let("x")], None),
                ]),
            ],
            span: Span::default(),
        };
        // 1 (while) + 1 (if) + 1 (let) = 3
        assert_eq!(count_statements(&block), 3);
    }

    // ---- Parameter count tests ----

    #[test]
    fn test_param_count_zero() {
        let func = make_function("main", vec![], vec![], Span::default());
        assert_eq!(func.params.len(), 0);
    }

    #[test]
    fn test_param_count_multiple() {
        let params = vec![
            Parameter { name: "a".to_string(), typ: Type::Int, span: Span::default() },
            Parameter { name: "b".to_string(), typ: Type::Float, span: Span::default() },
            Parameter { name: "c".to_string(), typ: Type::Bool, span: Span::default() },
        ];
        let func = make_function("add", params, vec![], Span::default());
        assert_eq!(func.params.len(), 3);
    }
}