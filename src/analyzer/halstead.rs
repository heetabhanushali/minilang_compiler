// src/analyzer/halstead.rs - Halstead Complexity Metrics
//
// Measures program complexity by counting operators and operands:
//
// n1 = unique operators     n2 = unique operands
// N1 = total operators      N2 = total operands
// Vocabulary:  n = n1 + n2
// Length:      N = N1 + N2
// Volume:      N * log2(n)
// Difficulty: (n1 / 2) * (N2 / n2)
// Effort:     difficulty * volume

use std::collections::HashSet;
use crate::ast::*;
use crate::analyzer::HalsteadMetrics;

/// Collector that accumulates operators and operands
struct HalsteadCollector {
    operators: Vec<String>,
    operands: Vec<String>,
}

impl HalsteadCollector {
    fn new() -> Self {
        Self {
            operators: Vec::new(),
            operands: Vec::new(),
        }
    }

    fn add_operator(&mut self, op: &str) {
        self.operators.push(op.to_string());
    }

    fn add_operand(&mut self, operand: &str) {
        self.operands.push(operand.to_string());
    }

    fn compute(self) -> HalsteadMetrics {
        let unique_operators: HashSet<&str> = self.operators.iter().map(|s| s.as_str()).collect();
        let unique_operands: HashSet<&str> = self.operands.iter().map(|s| s.as_str()).collect();

        let n1 = unique_operators.len();
        let n2 = unique_operands.len();
        let big_n1 = self.operators.len();
        let big_n2 = self.operands.len();

        let vocabulary = n1 + n2;
        let length = big_n1 + big_n2;

        let volume = if vocabulary > 0 {
            length as f64 * (vocabulary as f64).log2()
        } else {
            0.0
        };

        let difficulty = if n2 > 0 {
            (n1 as f64 / 2.0) * (big_n2 as f64 / n2 as f64)
        } else {
            0.0
        };

        let effort = difficulty * volume;

        HalsteadMetrics {
            unique_operators: n1,
            unique_operands: n2,
            total_operators: big_n1,
            total_operands: big_n2,
            vocabulary,
            length,
            volume,
            difficulty,
            effort,
        }
    }
}

/// Calculate Halstead metrics for a function
pub fn calculate(func: &Function) -> HalsteadMetrics {
    let mut collector = HalsteadCollector::new();
    collect_block(&func.body, &mut collector);
    collector.compute()
}

fn collect_block(block: &Block, c: &mut HalsteadCollector) {
    for stmt in &block.statements {
        collect_statement(stmt, c);
    }
}

fn collect_statement(stmt: &Statement, c: &mut HalsteadCollector) {
    match stmt {
        Statement::Let(let_stmt) => {
            c.add_operator("let");
            c.add_operand(&let_stmt.name);
            if let Some(ref value) = let_stmt.value {
                c.add_operator("=");
                collect_expression(value, c);
            }
        }
        Statement::Const(const_stmt) => {
            c.add_operator("const");
            c.add_operand(&const_stmt.name);
            c.add_operator("=");
            collect_expression(&const_stmt.value, c);
        }
        Statement::Display(display_stmt) => {
            c.add_operator("display");
            for expr in &display_stmt.expressions {
                collect_expression(expr, c);
            }
        }
        Statement::If(if_stmt) => {
            c.add_operator("if");
            collect_expression(&if_stmt.condition, c);
            collect_block(&if_stmt.then_block, c);
            if let Some(ref else_block) = if_stmt.else_block {
                c.add_operator("else");
                collect_block(else_block, c);
            }
        }
        Statement::While(while_stmt) => {
            c.add_operator("while");
            collect_expression(&while_stmt.condition, c);
            collect_block(&while_stmt.body, c);
        }
        Statement::DoWhile(do_while_stmt) => {
            c.add_operator("do-while");
            collect_expression(&do_while_stmt.condition, c);
            collect_block(&do_while_stmt.body, c);
        }
        Statement::For(for_stmt) => {
            c.add_operator("for");
            if let Some(ref init) = for_stmt.init {
                collect_statement(init, c);
            }
            if let Some(ref cond) = for_stmt.condition {
                collect_expression(cond, c);
            }
            if let Some(ref update) = for_stmt.update {
                collect_expression(update, c);
            }
            collect_block(&for_stmt.body, c);
        }
        Statement::Return(ret_stmt) => {
            c.add_operator("send");
            if let Some(ref value) = ret_stmt.value {
                collect_expression(value, c);
            }
        }
        Statement::Expression(expr_stmt) => {
            collect_expression(&expr_stmt.expression, c);
        }
        Statement::Block(block) => {
            collect_block(block, c);
        }
        Statement::Break(_) => {
            c.add_operator("break");
        }
        Statement::Continue(_) => {
            c.add_operator("continue");
        }
    }
}

fn collect_expression(expr: &Expression, c: &mut HalsteadCollector) {
    match expr {
        Expression::Literal(lit_expr) => {
            match &lit_expr.value {
                Literal::Integer(n) => c.add_operand(&n.to_string()),
                Literal::Float(f) => c.add_operand(&f.to_string()),
                Literal::String(s) => c.add_operand(&format!("\"{}\"", s)),
                Literal::Boolean(b) => c.add_operand(&b.to_string()),
                Literal::Array(elements) => {
                    c.add_operator("[]");
                    for elem in elements {
                        collect_expression(elem, c);
                    }
                }
                Literal::InterpolatedString(parts) => {
                    c.add_operator("interpolation");
                    for part in parts {
                        match part {
                            StringPart::Text(text) => c.add_operand(&format!("\"{}\"", text)),
                            StringPart::Expression(expr) => collect_expression(expr, c),
                        }
                    }
                }
            }
        }
        Expression::Identifier(id_expr) => {
            c.add_operand(&id_expr.name);
        }
        Expression::Binary(bin) => {
            let op_str = match bin.op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
                BinaryOp::Modulo => "%",
                BinaryOp::Equal => "==",
                BinaryOp::NotEqual => "!=",
                BinaryOp::Less => "<",
                BinaryOp::Greater => ">",
                BinaryOp::LessEqual => "<=",
                BinaryOp::GreaterEqual => ">=",
                BinaryOp::And => "&&",
                BinaryOp::Or => "||",
            };
            c.add_operator(op_str);
            collect_expression(&bin.left, c);
            collect_expression(&bin.right, c);
        }
        Expression::Unary(un) => {
            let op_str = match un.op {
                UnaryOp::Not => "!",
                UnaryOp::Negate => "negate",
            };
            c.add_operator(op_str);
            collect_expression(&un.operand, c);
        }
        Expression::Call(call) => {
            c.add_operator(&format!("call:{}", call.function));
            for arg in &call.args {
                collect_expression(arg, c);
            }
        }
        Expression::Index(idx) => {
            c.add_operator("index");
            collect_expression(&idx.array, c);
            collect_expression(&idx.index, c);
        }
        Expression::Assign(assign) => {
            c.add_operator("=");
            c.add_operand(&assign.target);
            collect_expression(&assign.value, c);
        }
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

    fn make_int_literal(n: i32) -> Expression {
        Expression::Literal(LiteralExpr {
            value: Literal::Integer(n),
            span: Span::default(),
        })
    }

    fn make_float_literal(f: f64) -> Expression {
        Expression::Literal(LiteralExpr {
            value: Literal::Float(f),
            span: Span::default(),
        })
    }

    fn make_string_literal(s: &str) -> Expression {
        Expression::Literal(LiteralExpr {
            value: Literal::String(s.to_string()),
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

    fn make_unary(op: UnaryOp, operand: Expression) -> Expression {
        Expression::Unary(UnaryExpr {
            op,
            operand: Box::new(operand),
            span: Span::default(),
        })
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

    fn make_display(exprs: Vec<Expression>) -> Statement {
        Statement::Display(DisplayStmt {
            expressions: exprs,
            span: Span::default(),
        })
    }

    fn make_if_simple(then_stmts: Vec<Statement>) -> Statement {
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

    fn make_return(value: Expression) -> Statement {
        Statement::Return(ReturnStmt {
            value: Some(value),
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

    fn make_assign(target: &str, value: Expression) -> Expression {
        Expression::Assign(AssignExpr {
            target: target.to_string(),
            value: Box::new(value),
            span: Span::default(),
        })
    }

    // ---- Tests ----

    #[test]
    fn test_empty_function() {
        let func = make_function(vec![]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 0);
        assert_eq!(h.unique_operands, 0);
        assert_eq!(h.total_operators, 0);
        assert_eq!(h.total_operands, 0);
        assert_eq!(h.vocabulary, 0);
        assert_eq!(h.length, 0);
        assert_eq!(h.volume, 0.0);
    }

    #[test]
    fn test_single_let() {
        // let x: int = 5;
        // operators: "let", "="
        // operands: "x", "5"
        let func = make_function(vec![
            make_let("x", make_int_literal(5)),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 2); // let, =
        assert_eq!(h.unique_operands, 2);  // x, 5
        assert_eq!(h.total_operators, 2);
        assert_eq!(h.total_operands, 2);
        assert_eq!(h.vocabulary, 4);
        assert_eq!(h.length, 4);
    }

    #[test]
    fn test_let_without_value() {
        // let x: int;
        // operators: "let"
        // operands: "x"
        let func = make_function(vec![
            make_let_no_value("x"),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 1); // let
        assert_eq!(h.unique_operands, 1);  // x
        assert_eq!(h.total_operators, 1);
        assert_eq!(h.total_operands, 1);
    }

    #[test]
    fn test_binary_expression() {
        // let result: int = a + b;
        // operators: "let", "=", "+"
        // operands: "result", "a", "b"
        let func = make_function(vec![
            make_let("result",
                make_binary(make_identifier("a"), BinaryOp::Add, make_identifier("b")),
            ),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 3); // let, =, +
        assert_eq!(h.unique_operands, 3);  // result, a, b
        assert_eq!(h.total_operators, 3);
        assert_eq!(h.total_operands, 3);
    }

    #[test]
    fn test_repeated_operands() {
        // let y: int = x + x;
        // operators: "let", "=", "+"
        // operands: "y", "x", "x" (x appears twice total)
        let func = make_function(vec![
            make_let("y",
                make_binary(make_identifier("x"), BinaryOp::Add, make_identifier("x")),
            ),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 3);  // let, =, +
        assert_eq!(h.unique_operands, 2);   // y, x (unique)
        assert_eq!(h.total_operators, 3);
        assert_eq!(h.total_operands, 3);     // y, x, x (total)
    }

    #[test]
    fn test_repeated_operators() {
        // let a: int = 1 + 2 + 3;
        // Parsed as (1 + 2) + 3
        // operators: "let", "=", "+", "+"
        // operands: "a", "1", "2", "3"
        let func = make_function(vec![
            make_let("a",
                make_binary(
                    make_binary(make_int_literal(1), BinaryOp::Add, make_int_literal(2)),
                    BinaryOp::Add,
                    make_int_literal(3),
                ),
            ),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 3);  // let, =, + (unique)
        assert_eq!(h.unique_operands, 4);   // a, 1, 2, 3
        assert_eq!(h.total_operators, 4);   // let, =, +, +
        assert_eq!(h.total_operands, 4);
    }

    #[test]
    fn test_display_statement() {
        // display "hello", x;
        // operators: "display"
        // operands: "hello", "x"
        let func = make_function(vec![
            make_display(vec![
                make_string_literal("hello"),
                make_identifier("x"),
            ]),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 1);  // display
        assert_eq!(h.unique_operands, 2);   // "hello", x
        assert_eq!(h.total_operators, 1);
        assert_eq!(h.total_operands, 2);
    }

    #[test]
    fn test_if_statement() {
        // if true { let x: int = 1; }
        // operators: "if", "let", "="
        // operands: "true", "x", "1"
        let func = make_function(vec![
            make_if_simple(vec![
                make_let("x", make_int_literal(1)),
            ]),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 3);  // if, let, =
        assert_eq!(h.unique_operands, 3);   // true, x, 1
        assert_eq!(h.total_operators, 3);
        assert_eq!(h.total_operands, 3);
    }

    #[test]
    fn test_return_statement() {
        // send 42;
        // operators: "send"
        // operands: "42"
        let func = make_function(vec![
            make_return(make_int_literal(42)),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 1);  // send
        assert_eq!(h.unique_operands, 1);   // 42
        assert_eq!(h.total_operators, 1);
        assert_eq!(h.total_operands, 1);
    }

    #[test]
    fn test_function_call() {
        // foo(a, 5)
        // operators: "call:foo"
        // operands: "a", "5"
        let func = make_function(vec![
            Statement::Expression(ExprStmt {
                expression: make_call("foo", vec![
                    make_identifier("a"),
                    make_int_literal(5),
                ]),
                span: Span::default(),
            }),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 1);  // call:foo
        assert_eq!(h.unique_operands, 2);   // a, 5
        assert_eq!(h.total_operators, 1);
        assert_eq!(h.total_operands, 2);
    }

    #[test]
    fn test_assignment() {
        // x = x + 1;
        // operators: "=", "+"
        // operands: "x", "x", "1" (x as target + x in expression)
        let func = make_function(vec![
            Statement::Expression(ExprStmt {
                expression: make_assign("x",
                    make_binary(make_identifier("x"), BinaryOp::Add, make_int_literal(1)),
                ),
                span: Span::default(),
            }),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 2);  // =, +
        assert_eq!(h.unique_operands, 2);   // x, 1 (unique)
        assert_eq!(h.total_operators, 2);
        assert_eq!(h.total_operands, 3);    // x(target), x(expr), 1
    }

    #[test]
    fn test_unary_expression() {
        // let b: int = !flag;
        // operators: "let", "=", "!"
        // operands: "b", "flag"
        let func = make_function(vec![
            make_let("b", make_unary(UnaryOp::Not, make_identifier("flag"))),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 3);  // let, =, !
        assert_eq!(h.unique_operands, 2);   // b, flag
        assert_eq!(h.total_operators, 3);
        assert_eq!(h.total_operands, 2);
    }

    #[test]
    fn test_volume_calculation() {
        // let x: int = 5;
        // vocabulary = 4, length = 4
        // volume = 4 * log2(4) = 4 * 2 = 8.0
        let func = make_function(vec![
            make_let("x", make_int_literal(5)),
        ]);
        let h = calculate(&func);
        assert!((h.volume - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_difficulty_calculation() {
        // let x: int = a + a;
        // n1=3 (let, =, +), N2=3 (x, a, a), n2=2 (x, a unique)
        // difficulty = (3/2) * (3/2) = 2.25
        let func = make_function(vec![
            make_let("x",
                make_binary(make_identifier("a"), BinaryOp::Add, make_identifier("a")),
            ),
        ]);
        let h = calculate(&func);
        assert!((h.difficulty - 2.25).abs() < 0.001);
    }

    #[test]
    fn test_float_and_bool_operands() {
        // let pi: float = 3.14;
        // let flag: bool = true;
        // operators: "let", "=", "let", "="
        // operands: "pi", "3.14", "flag", "true"
        let func = make_function(vec![
            Statement::Let(LetStmt {
                name: "pi".to_string(),
                typ: Type::Float,
                value: Some(make_float_literal(3.14)),
                span: Span::default(),
            }),
            Statement::Let(LetStmt {
                name: "flag".to_string(),
                typ: Type::Bool,
                value: Some(make_bool_literal(true)),
                span: Span::default(),
            }),
        ]);
        let h = calculate(&func);
        assert_eq!(h.unique_operators, 2);  // let, =
        assert_eq!(h.unique_operands, 4);   // pi, 3.14, flag, true
        assert_eq!(h.total_operators, 4);   // let, =, let, =
        assert_eq!(h.total_operands, 4);
    }

    #[test]
    fn test_complex_program() {
        // let x: int = 0;
        // let y: int = x + 1;
        // if (x > 0) { display x; }
        // send y;
        let func = make_function(vec![
            make_let("x", make_int_literal(0)),
            make_let("y",
                make_binary(make_identifier("x"), BinaryOp::Add, make_int_literal(1)),
            ),
            Statement::If(IfStmt {
                condition: make_binary(make_identifier("x"), BinaryOp::Greater, make_int_literal(0)),
                then_block: Block {
                    statements: vec![make_display(vec![make_identifier("x")])],
                    span: Span::default(),
                },
                else_block: None,
                span: Span::default(),
            }),
            make_return(make_identifier("y")),
        ]);
        let h = calculate(&func);

        // operators: let, =, let, =, +, if, >, display, send = 9 total
        // unique operators: let, =, +, if, >, display, send = 7
        assert_eq!(h.total_operators, 9);
        assert_eq!(h.unique_operators, 7);

        // operands: x, 0, y, x, 1, x, 0, x, y = 9 total
        // unique operands: x, 0, y, 1 = 4
        assert_eq!(h.total_operands, 9);
        assert_eq!(h.unique_operands, 4);

        assert_eq!(h.vocabulary, 11);
        assert_eq!(h.length, 18);
    }
}