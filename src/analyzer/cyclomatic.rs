// src/analyzer/cyclomatic.rs - Cyclomatic Complexity
//
// Cyclomatic complexity = 1 + number of decision points
//
// Decision points:
//   if, while, do-while, for     → +1 each
//   && (And), || (Or)            → +1 each

use crate::ast::*;

/// Calculate cyclomatic complexity for a function
pub fn calculate(func: &Function) -> usize {
    // Start at 1 (the base path through the function)
    1 + count_decisions_in_block(&func.body)
}

fn count_decisions_in_block(block: &Block) -> usize {
    let mut count = 0;
    for stmt in &block.statements {
        count += count_decisions_in_statement(stmt);
    }
    count
}

fn count_decisions_in_statement(stmt: &Statement) -> usize {
    match stmt {
        Statement::If(if_stmt) => {
            let mut count = 1; // the if itself is a decision
            count += count_decisions_in_expression(&if_stmt.condition);
            count += count_decisions_in_block(&if_stmt.then_block);
            if let Some(ref else_block) = if_stmt.else_block {
                count += count_decisions_in_block(else_block);
            }
            count
        }
        Statement::While(while_stmt) => {
            let mut count = 1;
            count += count_decisions_in_expression(&while_stmt.condition);
            count += count_decisions_in_block(&while_stmt.body);
            count
        }
        Statement::DoWhile(do_while_stmt) => {
            let mut count = 1;
            count += count_decisions_in_expression(&do_while_stmt.condition);
            count += count_decisions_in_block(&do_while_stmt.body);
            count
        }
        Statement::For(for_stmt) => {
            let mut count = 1;
            if let Some(ref condition) = for_stmt.condition {
                count += count_decisions_in_expression(condition);
            }
            if let Some(ref init) = for_stmt.init {
                count += count_decisions_in_statement(init);
            }
            if let Some(ref update) = for_stmt.update {
                count += count_decisions_in_expression(update);
            }
            count += count_decisions_in_block(&for_stmt.body);
            count
        }
        Statement::Block(block) => {
            count_decisions_in_block(block)
        }
        // Statements that may contain expressions with && / ||
        Statement::Let(let_stmt) => {
            if let Some(ref value) = let_stmt.value {
                count_decisions_in_expression(value)
            } else {
                0
            }
        }
        Statement::Const(const_stmt) => {
            count_decisions_in_expression(&const_stmt.value)
        }
        Statement::Display(display_stmt) => {
            display_stmt.expressions.iter()
                .map(|e| count_decisions_in_expression(e))
                .sum()
        }
        Statement::Return(ret_stmt) => {
            if let Some(ref value) = ret_stmt.value {
                count_decisions_in_expression(value)
            } else {
                0
            }
        }
        Statement::Expression(expr_stmt) => {
            count_decisions_in_expression(&expr_stmt.expression)
        }
        Statement::Break(_) | Statement::Continue(_) => 0,
    }
}

fn count_decisions_in_expression(expr: &Expression) -> usize {
    match expr {
        Expression::Binary(bin) => {
            let op_count = match bin.op {
                BinaryOp::And | BinaryOp::Or => 1,
                _ => 0,
            };
            op_count
                + count_decisions_in_expression(&bin.left)
                + count_decisions_in_expression(&bin.right)
        }
        Expression::Unary(un) => {
            count_decisions_in_expression(&un.operand)
        }
        Expression::Call(call) => {
            call.args.iter()
                .map(|a| count_decisions_in_expression(a))
                .sum()
        }
        Expression::Index(idx) => {
            count_decisions_in_expression(&idx.array)
                + count_decisions_in_expression(&idx.index)
        }
        Expression::Assign(assign) => {
            count_decisions_in_expression(&assign.value)
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

    // ---- Tests ----

    #[test]
    fn test_empty_function() {
        // No decisions → complexity = 1
        let func = make_function(vec![]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_linear_code() {
        // No branches → complexity = 1
        let func = make_function(vec![
            make_let("x"),
            make_let("y"),
            make_let("z"),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_single_if() {
        // 1 + 1(if) = 2
        let func = make_function(vec![
            make_if(make_bool_literal(true), vec![make_let("x")], None),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_if_else() {
        // if-else is still just one decision point
        // 1 + 1(if) = 2
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
    fn test_two_ifs() {
        // 1 + 1(if) + 1(if) = 3
        let func = make_function(vec![
            make_if(make_bool_literal(true), vec![make_let("x")], None),
            make_if(make_bool_literal(false), vec![make_let("y")], None),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_single_while() {
        // 1 + 1(while) = 2
        let func = make_function(vec![
            make_while(make_bool_literal(true), vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_single_do_while() {
        // 1 + 1(do-while) = 2
        let func = make_function(vec![
            make_do_while(make_bool_literal(true), vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_single_for() {
        // 1 + 1(for) = 2
        let func = make_function(vec![
            make_for(vec![make_let("x")]),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_condition_with_and() {
        // if (a && b) → 1(if) + 1(&&) = 2 decisions → complexity = 3
        let condition = make_binary(
            make_identifier("a"),
            BinaryOp::And,
            make_identifier("b"),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_condition_with_or() {
        // if (a || b) → 1(if) + 1(||) = 2 decisions → complexity = 3
        let condition = make_binary(
            make_identifier("a"),
            BinaryOp::Or,
            make_identifier("b"),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_compound_condition() {
        // if (a && b || c) → 1(if) + 1(&&) + 1(||) = 3 decisions → complexity = 4
        let condition = make_binary(
            make_binary(
                make_identifier("a"),
                BinaryOp::And,
                make_identifier("b"),
            ),
            BinaryOp::Or,
            make_identifier("c"),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        assert_eq!(calculate(&func), 4);
    }

    #[test]
    fn test_nested_if_in_while() {
        // while { if { } } → 1 + 1(while) + 1(if) = 3
        let func = make_function(vec![
            make_while(
                make_bool_literal(true),
                vec![
                    make_if(make_bool_literal(true), vec![make_let("x")], None),
                ],
            ),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_complex_function() {
        // while (a || b) {
        //   if (c && d) { ... }
        //   if (e) { ... }
        // }
        // Decisions: 1(while) + 1(||) + 1(if) + 1(&&) + 1(if) = 5
        // Complexity = 1 + 5 = 6
        let func = make_function(vec![
            make_while(
                make_binary(make_identifier("a"), BinaryOp::Or, make_identifier("b")),
                vec![
                    make_if(
                        make_binary(make_identifier("c"), BinaryOp::And, make_identifier("d")),
                        vec![make_let("x")],
                        None,
                    ),
                    make_if(
                        make_identifier("e"),
                        vec![make_let("y")],
                        None,
                    ),
                ],
            ),
        ]);
        assert_eq!(calculate(&func), 6);
    }

    #[test]
    fn test_arithmetic_not_counted() {
        // if (a + b > c * d) → only 1(if), arithmetic ops don't count
        let condition = make_binary(
            make_binary(make_identifier("a"), BinaryOp::Add, make_identifier("b")),
            BinaryOp::Greater,
            make_binary(make_identifier("c"), BinaryOp::Multiply, make_identifier("d")),
        );
        let func = make_function(vec![
            make_if(condition, vec![make_let("x")], None),
        ]);
        assert_eq!(calculate(&func), 2);
    }
}