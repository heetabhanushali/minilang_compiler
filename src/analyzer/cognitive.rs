// src/analyzer/cognitive.rs - Cognitive Complexity (SonarSource-style)
//
// Unlike cyclomatic complexity, cognitive complexity accounts for
// human perception of code difficulty:
//
// 1. Control flow structures get +1 PLUS the current nesting level
// 2. else gets +1 (flat, no nesting bonus)
// 3. break/continue get +1 (flat)
// 4. Logical operator sequences: +1 for first &&/||, +1 for each type switch
// 5. Nesting level increases inside control flow bodies

use crate::ast::*;

/// Calculate cognitive complexity for a function
pub fn calculate(func: &Function) -> usize {
    cognitive_block(&func.body, 0)
}

fn cognitive_block(block: &Block, nesting: usize) -> usize {
    let mut total = 0;
    for stmt in &block.statements {
        total += cognitive_statement(stmt, nesting);
    }
    total
}

fn cognitive_statement(stmt: &Statement, nesting: usize) -> usize {
    match stmt {
        Statement::If(if_stmt) => {
            let mut score = 1 + nesting; // +1 base, +nesting
            score += cognitive_expression(&if_stmt.condition);
            score += cognitive_block(&if_stmt.then_block, nesting + 1);
            if let Some(ref else_block) = if_stmt.else_block {
                score += 1; // else: +1, no nesting bonus
                score += cognitive_block(else_block, nesting + 1);
            }
            score
        }
        Statement::While(while_stmt) => {
            let mut score = 1 + nesting;
            score += cognitive_expression(&while_stmt.condition);
            score += cognitive_block(&while_stmt.body, nesting + 1);
            score
        }
        Statement::DoWhile(do_while_stmt) => {
            let mut score = 1 + nesting;
            score += cognitive_expression(&do_while_stmt.condition);
            score += cognitive_block(&do_while_stmt.body, nesting + 1);
            score
        }
        Statement::For(for_stmt) => {
            let mut score = 1 + nesting;
            if let Some(ref init) = for_stmt.init {
                score += cognitive_statement(init, nesting);
            }
            if let Some(ref cond) = for_stmt.condition {
                score += cognitive_expression(cond);
            }
            if let Some(ref update) = for_stmt.update {
                score += cognitive_expression(update);
            }
            score += cognitive_block(&for_stmt.body, nesting + 1);
            score
        }
        Statement::Break(_) => 1,
        Statement::Continue(_) => 1,
        Statement::Block(block) => {
            // Bare block: no increment, no nesting increase
            cognitive_block(block, nesting)
        }
        // Non-control-flow: only logical operators in expressions count
        Statement::Let(let_stmt) => {
            if let Some(ref value) = let_stmt.value {
                cognitive_expression(value)
            } else {
                0
            }
        }
        Statement::Const(const_stmt) => {
            cognitive_expression(&const_stmt.value)
        }
        Statement::Display(display_stmt) => {
            display_stmt.expressions.iter()
                .map(|e| cognitive_expression(e))
                .sum()
        }
        Statement::Return(ret_stmt) => {
            if let Some(ref value) = ret_stmt.value {
                cognitive_expression(value)
            } else {
                0
            }
        }
        Statement::Expression(expr_stmt) => {
            cognitive_expression(&expr_stmt.expression)
        }
    }
}

/// Score logical operator sequences in an expression
fn cognitive_expression(expr: &Expression) -> usize {
    count_logical_complexity(expr, None)
}

/// Walk the expression tree counting logical operator sequences.
///
/// - First && or || encountered (no prior context): +1
/// - Same operator as parent logical op: +0 (continuation)
/// - Different operator from parent logical op: +1 (switch)
/// - Non-logical operator resets context
fn count_logical_complexity(expr: &Expression, last_logical_op: Option<&BinaryOp>) -> usize {
    match expr {
        Expression::Binary(bin) => {
            match bin.op {
                BinaryOp::And | BinaryOp::Or => {
                    let increment = match last_logical_op {
                        None => 1,                                // first in sequence
                        Some(last) if *last != bin.op => 1,       // operator switch
                        Some(_) => 0,                             // continuation
                    };
                    increment
                        + count_logical_complexity(&bin.left, Some(&bin.op))
                        + count_logical_complexity(&bin.right, Some(&bin.op))
                }
                _ => {
                    // Non-logical binary op resets context
                    count_logical_complexity(&bin.left, None)
                        + count_logical_complexity(&bin.right, None)
                }
            }
        }
        Expression::Unary(un) => {
            count_logical_complexity(&un.operand, None)
        }
        Expression::Call(call) => {
            call.args.iter()
                .map(|a| count_logical_complexity(a, None))
                .sum()
        }
        Expression::Index(idx) => {
            count_logical_complexity(&idx.array, None)
                + count_logical_complexity(&idx.index, None)
        }
        Expression::Assign(assign) => {
            count_logical_complexity(&assign.value, None)
        }
        Expression::Literal(_) | Expression::Identifier(_) => 0,
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

    fn make_bool_literal(val: bool) -> Expression {
        Expression::Literal(LiteralExpr {
            value: Literal::Boolean(val),
            span: Span::default(),
        })
    }

    fn make_identifier(name: &str) -> Expression {
        Expression::Identifier(IdentifierExpr {
            name: name.to_string(),
            span: Span::default(),
        })
    }

    fn make_binary(left: Expression, op: BinaryOp, right: Expression) -> Expression {
        Expression::Binary(BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span: Span::default(),
            optimization_hint: None,
        })
    }

    fn make_if(condition: Expression, then_stmts: Vec<Statement>, else_stmts: Option<Vec<Statement>>) -> Statement {
        Statement::If(IfStmt {
            condition,
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

    fn make_while(condition: Expression, body_stmts: Vec<Statement>) -> Statement {
        Statement::While(WhileStmt {
            condition,
            body: Block {
                statements: body_stmts,
                span: Span::default(),
            },
            span: Span::default(),
        })
    }

    fn make_do_while(condition: Expression, body_stmts: Vec<Statement>) -> Statement {
        Statement::DoWhile(DoWhileStmt {
            condition,
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

    fn make_break() -> Statement {
        Statement::Break(BreakStmt { span: Span::default() })
    }

    fn make_continue() -> Statement {
        Statement::Continue(ContinueStmt { span: Span::default() })
    }

    // ---- Basic tests ----

    #[test]
    fn test_empty_function() {
        let func = make_function(vec![]);
        assert_eq!(calculate(&func), 0);
    }

    #[test]
    fn test_linear_code() {
        let func = make_function(vec![
            make_let("x"),
            make_let("y"),
            make_let("z"),
        ]);
        assert_eq!(calculate(&func), 0);
    }

    // ---- Control flow (no nesting) ----

    #[test]
    fn test_single_if() {
        // if: +1 + 0(nesting) = 1
        let func = make_function(vec![
            make_if(make_bool_literal(true), vec![make_let("x")], None),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_if_else() {
        // if: +1, else: +1 = 2
        let func = make_function(vec![
            make_if(
                make_bool_literal(true),
                vec![make_let("x")],
                Some(vec![make_let("y")]),
            ),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_single_while() {
        // while: +1 + 0 = 1
        let func = make_function(vec![
            make_while(make_bool_literal(true), vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_single_do_while() {
        // do-while: +1 + 0 = 1
        let func = make_function(vec![
            make_do_while(make_bool_literal(true), vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_single_for() {
        // for: +1 + 0 = 1
        let func = make_function(vec![
            make_for(vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    // ---- Break / Continue ----

    #[test]
    fn test_break() {
        // break: +1
        let func = make_function(vec![make_break()]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_continue() {
        // continue: +1
        let func = make_function(vec![make_continue()]);
        assert_eq!(calculate(&func), 1);
    }

    // ---- Nesting ----

    #[test]
    fn test_nested_if_in_if() {
        // outer if: +1 + 0 = 1
        // inner if: +1 + 1 = 2
        // total: 3
        let func = make_function(vec![
            make_if(make_bool_literal(true), vec![
                make_if(make_bool_literal(true), vec![make_let("x")], None),
            ], None),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_nested_while_in_if() {
        // if: +1 + 0 = 1
        // while: +1 + 1 = 2
        // total: 3
        let func = make_function(vec![
            make_if(make_bool_literal(true), vec![
                make_while(make_bool_literal(true), vec![make_let("x")]),
            ], None),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_if_else_with_nested_if_in_else() {
        // if: +1 + 0 = 1
        // else: +1
        // inner if (inside else, nesting=1): +1 + 1 = 2
        // total: 4
        let func = make_function(vec![
            make_if(
                make_bool_literal(true),
                vec![make_let("x")],
                Some(vec![
                    make_if(make_bool_literal(true), vec![make_let("y")], None),
                ]),
            ),
        ]);
        assert_eq!(calculate(&func), 4);
    }

    #[test]
    fn test_deeply_nested_4_levels() {
        // if:       +1 + 0 = 1
        //   while:  +1 + 1 = 2
        //     if:   +1 + 2 = 3
        //       for: +1 + 3 = 4
        // total: 10
        let func = make_function(vec![
            make_if(make_bool_literal(true), vec![
                make_while(make_bool_literal(true), vec![
                    make_if(make_bool_literal(true), vec![
                        make_for(vec![make_let("x")]),
                    ], None),
                ]),
            ], None),
        ]);
        assert_eq!(calculate(&func), 10);
    }

    // ---- Logical operators ----

    #[test]
    fn test_single_and() {
        // a && b → +1
        let func = make_function(vec![
            make_if(
                make_binary(make_identifier("a"), BinaryOp::And, make_identifier("b")),
                vec![make_let("x")],
                None,
            ),
        ]);
        // if: +1, &&: +1 = 2
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_single_or() {
        // a || b → +1
        let func = make_function(vec![
            make_if(
                make_binary(make_identifier("a"), BinaryOp::Or, make_identifier("b")),
                vec![make_let("x")],
                None,
            ),
        ]);
        // if: +1, ||: +1 = 2
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_same_operator_sequence() {
        // a && b && c → only +1 (same operator continues)
        // Parsed as (a && b) && c
        let condition = make_binary(
            make_binary(make_identifier("a"), BinaryOp::And, make_identifier("b")),
            BinaryOp::And,
            make_identifier("c"),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        // if: +1, && sequence: +1 = 2
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_operator_switch_and_then_or() {
        // a && b || c → +1(&&) +1(||) = 2
        // Parsed as (a && b) || c
        let condition = make_binary(
            make_binary(make_identifier("a"), BinaryOp::And, make_identifier("b")),
            BinaryOp::Or,
            make_identifier("c"),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        // if: +1, logical: +2 = 3
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_operator_switch_three_groups() {
        // a && b || c && d → +1(&&) +1(||) +1(&&) = 3
        // Parsed as (a && b) || (c && d)
        let condition = make_binary(
            make_binary(make_identifier("a"), BinaryOp::And, make_identifier("b")),
            BinaryOp::Or,
            make_binary(make_identifier("c"), BinaryOp::And, make_identifier("d")),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        // if: +1, logical: +3 = 4
        assert_eq!(calculate(&func), 4);
    }

    #[test]
    fn test_arithmetic_not_counted() {
        // if (a + b > c) → no logical operators
        let condition = make_binary(
            make_binary(make_identifier("a"), BinaryOp::Add, make_identifier("b")),
            BinaryOp::Greater,
            make_identifier("c"),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        // if: +1, no logical ops = 1
        assert_eq!(calculate(&func), 1);
    }

    // ---- Complex combined ----

    #[test]
    fn test_complex_function() {
        // if (a && b) {           +1(if) +0(nesting) +1(&&) = 2
        //   while (c || d) {      +1(while) +1(nesting) +1(||) = 3
        //     break;              +1
        //   }
        // } else {                +1
        //   for ... {             +1(for) +1(nesting) = 2
        //     let x;
        //   }
        // }
        // do-while (e && f || g)  +1(do-while) +0(nesting) +2(&&,||) = 3
        // {
        //   continue;            +1
        // }
        // total: 2 + 3 + 1 + 1 + 2 + 3 + 1 = 13
        let func = make_function(vec![
            make_if(
                make_binary(make_identifier("a"), BinaryOp::And, make_identifier("b")),
                vec![
                    make_while(
                        make_binary(make_identifier("c"), BinaryOp::Or, make_identifier("d")),
                        vec![make_break()],
                    ),
                ],
                Some(vec![
                    make_for(vec![make_let("x")]),
                ]),
            ),
            make_do_while(
                // e && f || g → parsed as (e && f) || g
                make_binary(
                    make_binary(make_identifier("e"), BinaryOp::And, make_identifier("f")),
                    BinaryOp::Or,
                    make_identifier("g"),
                ),
                vec![make_continue()],
            ),
        ]);
        assert_eq!(calculate(&func), 13);
    }

    #[test]
    fn test_break_inside_nested_loop() {
        // while (true) {        +1 + 0 = 1
        //   if (x) {            +1 + 1 = 2
        //     break;            +1
        //   }
        // }
        // total: 4
        let func = make_function(vec![
            make_while(make_bool_literal(true), vec![
                make_if(make_identifier("x"), vec![
                    make_break(),
                ], None),
            ]),
        ]);
        assert_eq!(calculate(&func), 4);
    }

    #[test]
    fn test_sibling_ifs_no_nesting_penalty() {
        // if (a) { }  → +1 + 0 = 1
        // if (b) { }  → +1 + 0 = 1
        // if (c) { }  → +1 + 0 = 1
        // total: 3
        let func = make_function(vec![
            make_if(make_identifier("a"), vec![make_let("x")], None),
            make_if(make_identifier("b"), vec![make_let("y")], None),
            make_if(make_identifier("c"), vec![make_let("z")], None),
        ]);
        assert_eq!(calculate(&func), 3);
    }
}