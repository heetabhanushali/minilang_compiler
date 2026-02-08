// src/analyzer/nesting.rs - Maximum Nesting Depth
//
// Tracks how deeply nested control flow gets within a function.
// Each if/while/do-while/for/block increases depth by 1.

use crate::ast::*;

/// Calculate the maximum nesting depth of a function body
pub fn calculate(func: &Function) -> usize {
    max_depth_in_block(&func.body, 0)
}

fn max_depth_in_block(block: &Block, current_depth: usize) -> usize {
    let mut max = current_depth;

    for stmt in &block.statements {
        let depth = max_depth_in_statement(stmt, current_depth);
        if depth > max {
            max = depth;
        }
    }

    max
}

fn max_depth_in_statement(stmt: &Statement, current_depth: usize) -> usize {
    match stmt {
        Statement::If(if_stmt) => {
            let nested_depth = current_depth + 1;
            let mut max = nested_depth;

            let then_max = max_depth_in_block(&if_stmt.then_block, nested_depth);
            if then_max > max {
                max = then_max;
            }

            if let Some(ref else_block) = if_stmt.else_block {
                let else_max = max_depth_in_block(else_block, nested_depth);
                if else_max > max {
                    max = else_max;
                }
            }

            max
        }
        Statement::While(while_stmt) => {
            let nested_depth = current_depth + 1;
            let body_max = max_depth_in_block(&while_stmt.body, nested_depth);
            nested_depth.max(body_max)
        }
        Statement::DoWhile(do_while_stmt) => {
            let nested_depth = current_depth + 1;
            let body_max = max_depth_in_block(&do_while_stmt.body, nested_depth);
            nested_depth.max(body_max)
        }
        Statement::For(for_stmt) => {
            let nested_depth = current_depth + 1;
            let body_max = max_depth_in_block(&for_stmt.body, nested_depth);
            nested_depth.max(body_max)
        }
        Statement::Block(block) => {
            let nested_depth = current_depth + 1;
            let block_max = max_depth_in_block(block, nested_depth);
            nested_depth.max(block_max)
        }
        // Non-nesting statements
        Statement::Let(_)
        | Statement::Const(_)
        | Statement::Display(_)
        | Statement::Return(_)
        | Statement::Expression(_)
        | Statement::Break(_)
        | Statement::Continue(_) => current_depth,
    }
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Helpers ----

    fn make_function(stmts: Vec<Statement>) -> Function {
        Function {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: Block {
                statements: stmts,
                span: Span::default(),
            },
            span: Span::default(),
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

    fn make_display() -> Statement {
        Statement::Display(DisplayStmt {
            expressions: vec![Expression::Literal(LiteralExpr {
                value: Literal::String("hello".to_string()),
                span: Span::default(),
            })],
            span: Span::default(),
        })
    }

    fn make_bool_literal(val: bool) -> Expression {
        Expression::Literal(LiteralExpr {
            value: Literal::Boolean(val),
            span: Span::default(),
        })
    }

    fn make_if(then_stmts: Vec<Statement>, else_stmts: Option<Vec<Statement>>) -> Statement {
        Statement::If(IfStmt {
            condition: make_bool_literal(true),
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

    fn make_do_while(body_stmts: Vec<Statement>) -> Statement {
        Statement::DoWhile(DoWhileStmt {
            condition: make_bool_literal(true),
            body: Block {
                statements: body_stmts,
                span: Span::default(),
            },
            span: Span::default(),
        })
    }

    fn make_for(body_stmts: Vec<Statement>) -> Statement {
        Statement::For(ForStmt {
            init: None,
            condition: None,
            update: None,
            body: Block {
                statements: body_stmts,
                span: Span::default(),
            },
            span: Span::default(),
        })
    }

    fn make_block(stmts: Vec<Statement>) -> Statement {
        Statement::Block(Block {
            statements: stmts,
            span: Span::default(),
        })
    }

    // ---- Tests ----

    #[test]
    fn test_empty_function() {
        let func = make_function(vec![]);
        assert_eq!(calculate(&func), 0);
    }

    #[test]
    fn test_flat_code() {
        // No nesting at all
        let func = make_function(vec![
            make_let("x"),
            make_let("y"),
            make_display(),
        ]);
        assert_eq!(calculate(&func), 0);
    }

    #[test]
    fn test_single_if_depth_1() {
        let func = make_function(vec![
            make_if(vec![make_let("x")], None),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_if_else_depth_1() {
        // if-else is still depth 1 (both branches at same level)
        let func = make_function(vec![
            make_if(
                vec![make_let("x")],
                Some(vec![make_let("y")]),
            ),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_single_while_depth_1() {
        let func = make_function(vec![
            make_while(vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_single_do_while_depth_1() {
        let func = make_function(vec![
            make_do_while(vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_single_for_depth_1() {
        let func = make_function(vec![
            make_for(vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_nested_if_in_if_depth_2() {
        // if { if { ... } }
        let func = make_function(vec![
            make_if(vec![
                make_if(vec![make_let("x")], None),
            ], None),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_nested_if_in_while_depth_2() {
        // while { if { ... } }
        let func = make_function(vec![
            make_while(vec![
                make_if(vec![make_let("x")], None),
            ]),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_triple_nesting_depth_3() {
        // for { while { if { ... } } }
        let func = make_function(vec![
            make_for(vec![
                make_while(vec![
                    make_if(vec![make_let("x")], None),
                ]),
            ]),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_deep_nesting_depth_5() {
        // if { while { if { for { do-while { ... } } } } }
        let func = make_function(vec![
            make_if(vec![
                make_while(vec![
                    make_if(vec![
                        make_for(vec![
                            make_do_while(vec![make_let("x")]),
                        ]),
                    ], None),
                ]),
            ], None),
        ]);
        assert_eq!(calculate(&func), 5);
    }

    #[test]
    fn test_sibling_branches_max() {
        // Two siblings at depth 1, one goes deeper
        // if { let } ← depth 1
        // while { if { let } } ← depth 2
        // max = 2
        let func = make_function(vec![
            make_if(vec![make_let("x")], None),
            make_while(vec![
                make_if(vec![make_let("y")], None),
            ]),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_else_branch_deeper() {
        // if { let } else { while { let } }
        // then branch: depth 1
        // else branch: depth 2
        // max = 2
        let func = make_function(vec![
            make_if(
                vec![make_let("x")],
                Some(vec![
                    make_while(vec![make_let("y")]),
                ]),
            ),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_block_statement_adds_depth() {
        // { if { let } } → block is depth 1, if inside is depth 2
        let func = make_function(vec![
            make_block(vec![
                make_if(vec![make_let("x")], None),
            ]),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_mixed_flat_and_nested() {
        // let x;
        // if { let y; }
        // let z;
        // while { for { let w; } }
        // max depth = 2 (while → for)
        let func = make_function(vec![
            make_let("x"),
            make_if(vec![make_let("y")], None),
            make_let("z"),
            make_while(vec![
                make_for(vec![make_let("w")]),
            ]),
        ]);
        assert_eq!(calculate(&func), 2);
    }
}