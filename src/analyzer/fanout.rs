// src/analyzer/fanout.rs - Fan-out (number of distinct functions called)
//
// Measures coupling: how many other functions does this function depend on.
// Higher fan-out = more dependencies = harder to test/maintain.

use std::collections::HashSet;
use crate::ast::*;

/// Calculate fan-out: number of distinct functions called
pub fn calculate(func: &Function) -> usize {
    let mut called: HashSet<String> = HashSet::new();
    collect_block(&func.body, &mut called);
    called.len()
}

fn collect_block(block: &Block, called: &mut HashSet<String>) {
    for stmt in &block.statements {
        collect_statement(stmt, called);
    }
}

fn collect_statement(stmt: &Statement, called: &mut HashSet<String>) {
    match stmt {
        Statement::Let(let_stmt) => {
            if let Some(ref value) = let_stmt.value {
                collect_expression(value, called);
            }
        }
        Statement::Const(const_stmt) => {
            collect_expression(&const_stmt.value, called);
        }
        Statement::Display(display_stmt) => {
            for expr in &display_stmt.expressions {
                collect_expression(expr, called);
            }
        }
        Statement::If(if_stmt) => {
            collect_expression(&if_stmt.condition, called);
            collect_block(&if_stmt.then_block, called);
            if let Some(ref else_block) = if_stmt.else_block {
                collect_block(else_block, called);
            }
        }
        Statement::While(while_stmt) => {
            collect_expression(&while_stmt.condition, called);
            collect_block(&while_stmt.body, called);
        }
        Statement::DoWhile(do_while_stmt) => {
            collect_expression(&do_while_stmt.condition, called);
            collect_block(&do_while_stmt.body, called);
        }
        Statement::For(for_stmt) => {
            if let Some(ref init) = for_stmt.init {
                collect_statement(init, called);
            }
            if let Some(ref cond) = for_stmt.condition {
                collect_expression(cond, called);
            }
            if let Some(ref update) = for_stmt.update {
                collect_expression(update, called);
            }
            collect_block(&for_stmt.body, called);
        }
        Statement::Return(ret_stmt) => {
            if let Some(ref value) = ret_stmt.value {
                collect_expression(value, called);
            }
        }
        Statement::Expression(expr_stmt) => {
            collect_expression(&expr_stmt.expression, called);
        }
        Statement::Block(block) => {
            collect_block(block, called);
        }
        Statement::Break(_) | Statement::Continue(_) => {}
    }
}

fn collect_expression(expr: &Expression, called: &mut HashSet<String>) {
    match expr {
        Expression::Call(call) => {
            called.insert(call.function.clone());
            for arg in &call.args {
                collect_expression(arg, called);
            }
        }
        Expression::Binary(bin) => {
            collect_expression(&bin.left, called);
            collect_expression(&bin.right, called);
        }
        Expression::Unary(un) => {
            collect_expression(&un.operand, called);
        }
        Expression::Index(idx) => {
            collect_expression(&idx.array, called);
            collect_expression(&idx.index, called);
        }
        Expression::Assign(assign) => {
            collect_expression(&assign.value, called);
        }
        Expression::Literal(lit_expr) => {
            match &lit_expr.value {
                Literal::Array(elements) => {
                    for elem in elements {
                        collect_expression(elem, called);
                    }
                }
                Literal::InterpolatedString(parts) => {
                    for part in parts {
                        if let StringPart::Expression(expr) = part {
                            collect_expression(expr, called);
                        }
                    }
                }
                _ => {}
            }
        }
        Expression::Identifier(_) => {}
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

    fn make_let(name: &str, value: Expression) -> Statement {
        Statement::Let(LetStmt {
            name: name.to_string(),
            typ: Type::Int,
            value: Some(value),
            span: Span::default(),
        })
    }

    fn make_let_no_value(name: &str) -> Statement {
        Statement::Let(LetStmt {
            name: name.to_string(),
            typ: Type::Int,
            value: None,
            span: Span::default(),
        })
    }

    fn make_int_literal(n: i32) -> Expression {
        Expression::Literal(LiteralExpr {
            value: Literal::Integer(n),
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

    fn make_call(name: &str, args: Vec<Expression>) -> Expression {
        Expression::Call(CallExpr {
            function: name.to_string(),
            args,
            span: Span::default(),
        })
    }

    fn make_expr_stmt(expr: Expression) -> Statement {
        Statement::Expression(ExprStmt {
            expression: expr,
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

    fn make_return(value: Expression) -> Statement {
        Statement::Return(ReturnStmt {
            value: Some(value),
            span: Span::default(),
        })
    }

    fn make_display(exprs: Vec<Expression>) -> Statement {
        Statement::Display(DisplayStmt {
            expressions: exprs,
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
    fn test_no_calls() {
        let func = make_function(vec![
            make_let("x", make_int_literal(5)),
            make_let_no_value("y"),
        ]);
        assert_eq!(calculate(&func), 0);
    }

    #[test]
    fn test_single_call() {
        // foo()
        let func = make_function(vec![
            make_expr_stmt(make_call("foo", vec![])),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_same_function_called_twice() {
        // foo(); foo(); → still fan-out = 1 (distinct)
        let func = make_function(vec![
            make_expr_stmt(make_call("foo", vec![])),
            make_expr_stmt(make_call("foo", vec![])),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_two_distinct_calls() {
        // foo(); bar();
        let func = make_function(vec![
            make_expr_stmt(make_call("foo", vec![])),
            make_expr_stmt(make_call("bar", vec![])),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_three_distinct_calls() {
        let func = make_function(vec![
            make_expr_stmt(make_call("foo", vec![])),
            make_expr_stmt(make_call("bar", vec![])),
            make_expr_stmt(make_call("baz", vec![])),
        ]);
        assert_eq!(calculate(&func), 3);
    }

    #[test]
    fn test_call_in_let_value() {
        // let x: int = foo();
        let func = make_function(vec![
            make_let("x", make_call("foo", vec![])),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_return() {
        // send foo();
        let func = make_function(vec![
            make_return(make_call("foo", vec![])),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_display() {
        // display foo();
        let func = make_function(vec![
            make_display(vec![make_call("foo", vec![])]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_if_condition() {
        // if (check()) { ... }
        let func = make_function(vec![
            Statement::If(IfStmt {
                condition: make_call("check", vec![]),
                then_block: Block {
                    statements: vec![],
                    span: Span::default(),
                },
                else_block: None,
                span: Span::default(),
            }),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_if_body() {
        // if (true) { foo(); }
        let func = make_function(vec![
            make_if(vec![
                make_expr_stmt(make_call("foo", vec![])),
            ], None),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_else_body() {
        // if (true) { } else { bar(); }
        let func = make_function(vec![
            make_if(
                vec![],
                Some(vec![make_expr_stmt(make_call("bar", vec![]))]),
            ),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_while_body() {
        // while (true) { process(); }
        let func = make_function(vec![
            make_while(vec![
                make_expr_stmt(make_call("process", vec![])),
            ]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_for_body() {
        // for ... { step(); }
        let func = make_function(vec![
            make_for(vec![
                make_expr_stmt(make_call("step", vec![])),
            ]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_call_in_do_while_body() {
        // do { work(); } while (true);
        let func = make_function(vec![
            make_do_while(vec![
                make_expr_stmt(make_call("work", vec![])),
            ]),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_nested_call_in_argument() {
        // foo(bar(x))
        let func = make_function(vec![
            make_expr_stmt(make_call("foo", vec![
                make_call("bar", vec![make_identifier("x")]),
            ])),
        ]);
        assert_eq!(calculate(&func), 2); // foo, bar
    }

    #[test]
    fn test_call_in_binary_expression() {
        // let x = foo() + bar();
        let func = make_function(vec![
            make_let("x", Expression::Binary(BinaryExpr {
                left: Box::new(make_call("foo", vec![])),
                op: BinaryOp::Add,
                right: Box::new(make_call("bar", vec![])),
                span: Span::default(),
                optimization_hint: None,
            })),
        ]);
        assert_eq!(calculate(&func), 2);
    }

    #[test]
    fn test_call_in_assign() {
        // x = compute();
        let func = make_function(vec![
            make_expr_stmt(Expression::Assign(AssignExpr {
                target: "x".to_string(),
                value: Box::new(make_call("compute", vec![])),
                span: Span::default(),
            })),
        ]);
        assert_eq!(calculate(&func), 1);
    }

    #[test]
    fn test_many_calls_across_branches() {
        // if (check()) {
        //   foo();
        //   bar();
        // } else {
        //   baz();
        //   foo();  // duplicate
        // }
        // while (true) { qux(); }
        let func = make_function(vec![
            Statement::If(IfStmt {
                condition: make_call("check", vec![]),
                then_block: Block {
                    statements: vec![
                        make_expr_stmt(make_call("foo", vec![])),
                        make_expr_stmt(make_call("bar", vec![])),
                    ],
                    span: Span::default(),
                },
                else_block: Some(Block {
                    statements: vec![
                        make_expr_stmt(make_call("baz", vec![])),
                        make_expr_stmt(make_call("foo", vec![])), // dup
                    ],
                    span: Span::default(),
                }),
                span: Span::default(),
            }),
            make_while(vec![
                make_expr_stmt(make_call("qux", vec![])),
            ]),
        ]);
        // distinct: check, foo, bar, baz, qux = 5
        assert_eq!(calculate(&func), 5);
    }

    #[test]
    fn test_call_in_unary() {
        // !is_valid()
        let func = make_function(vec![
            make_expr_stmt(Expression::Unary(UnaryExpr {
                op: UnaryOp::Not,
                operand: Box::new(make_call("is_valid", vec![])),
                span: Span::default(),
            })),
        ]);
        assert_eq!(calculate(&func), 1);
    }
}