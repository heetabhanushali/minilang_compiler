// src/ast.rs - Abstract Syntax Tree definitions with visualization

use std::fmt;
use serde::{Serialize, Deserialize};

/// The root of our AST - a complete MiniLang program
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}

/// A function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

/// A function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub typ: Type,
    pub span: Span,
}

/// A block of statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// All possible statement types in MiniLang
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Let(LetStmt),
    Display(DisplayStmt),
    If(IfStmt),
    While(WhileStmt),
    DoWhile(DoWhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Expression(ExprStmt),
    Block(Block),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Const(ConstStmt),
}

/// Constant declaration: const PI: float = 3.14159;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstStmt {
    pub name: String,
    pub typ: Type,
    pub value: Expression,  // Constants MUST have a value
    pub span: Span,
}

/// Break statement: break;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreakStmt {
    pub span: Span,
}

/// Continue statement: continue;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinueStmt {
    pub span: Span,
}

/// Variable declaration: let x: int = 42;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetStmt {
    pub name: String,
    pub typ: Type,
    pub value: Option<Expression>,
    pub span: Span,
}

/// Display statement: display "Hello", x;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayStmt {
    pub expressions: Vec<Expression>,
    pub span: Span,
}

/// If statement: if condition { ... } else { ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

/// While loop: while condition { ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStmt {
    pub condition: Expression,
    pub body: Block,
    pub span: Span,
}

/// Do-while loop: do { ... } while condition;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoWhileStmt {
    pub body: Block,
    pub condition: Expression,
    pub span: Span,
}

/// For loop: for i = 0; i < 10; i = i + 1 { ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStmt {
    pub init: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Block,
    pub span: Span,
}

/// Return statement: send value;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStmt {
    pub value: Option<Expression>,
    pub span: Span,
}

/// Expression statement: x + 1;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExprStmt {
    pub expression: Expression,
    pub span: Span,
}

/// All possible expression types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(LiteralExpr),
    Identifier(IdentifierExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    Assign(AssignExpr),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteralExpr {
    pub value: Literal,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifierExpr {
    pub name: String,
    pub span: Span,
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Expression>),
    InterpolatedString(Vec<StringPart>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    Text(String),           // Regular text
    Expression(Expression), // Interpolated {expression}
}

/// Binary expression: left op right
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpr {
    pub left: Box<Expression>,
    pub op: BinaryOp,
    pub right: Box<Expression>,
    pub span: Span,
    pub optimization_hint: Option<OptimizationHint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationHint {
    ShiftLeft(u32),   // Multiply by 2^n -> shift left by n
    ShiftRight(u32),  // Divide by 2^n -> shift right by n
    BitwiseAnd(i32),  // Modulo by 2^n -> AND with (2^n - 1)
}

/// Binary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Logical
    And,
    Or,
}

/// Unary expression: op operand
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<Expression>,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Negate,
}

/// Function call: func(args)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpr {
    pub function: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

/// Array indexing: arr[index]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexExpr {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}

/// Assignment: target = value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignExpr {
    pub target: String,
    pub value: Box<Expression>,
    pub span: Span,
}

/// Type annotations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Array(Box<Type>, usize),  // Array type with size
}

/// Source location tracking
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

// ==================== DISPLAY IMPLEMENTATIONS ====================

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for function in &self.functions {
            writeln!(f, "{}", function)?;
        }
        Ok(())
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func {}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {:?}", param.name, param.typ)?;
        }
        write!(f, ")")?;
        if let Some(ret) = &self.return_type {
            write!(f, " -> {:?}", ret)?;
        }
        write!(f, " {{ ... }}")
    }
}

// ==================== TREE VISUALIZATION ====================

impl Program {
    /// Display the AST as a nice tree structure
    pub fn display_tree(&self) {
        println!("\n{}", "═".repeat(60));
        println!("🌳 Abstract Syntax Tree (Visualization)");
        println!("{}", "═".repeat(60));
        
        if self.functions.is_empty() {
            println!("\n(empty program)");
        }
        
        for (i, func) in self.functions.iter().enumerate() {
            let is_last_func = i == self.functions.len() - 1;
            let prefix = if is_last_func { "└──" } else { "├──" };
            
            println!("\n{} 📦 Function: {}", prefix, func.name);
            self.display_function(func, if is_last_func { "    " } else { "│   " });
        }
        
        println!("\n{}", "═".repeat(60));
    }

    fn display_function(&self, func: &Function, indent: &str) {
        // Parameters
        if !func.params.is_empty() {
            println!("{}├── 📋 Parameters: {}", indent, func.params.len());
            for (i, param) in func.params.iter().enumerate() {
                let is_last = i == func.params.len() - 1;
                let p_prefix = if is_last { "└──" } else { "├──" };
                println!("{}│   {} {} : {:?}", indent, p_prefix, param.name, param.typ);
            }
        } else {
            println!("{}├── 📋 Parameters: none", indent);
        }
        
        // Return type
        if let Some(ret_type) = &func.return_type {
            println!("{}├── 🔄 Return Type: {:?}", indent, ret_type);
        }
        
        // Body
        let body_prefix = if func.return_type.is_some() { "└──" } else { "└──" };
        println!("{}{} 📝 Body: {} statement(s)", indent, body_prefix, func.body.statements.len());
        
        for (i, stmt) in func.body.statements.iter().enumerate() {
            let is_last = i == func.body.statements.len() - 1;
            let s_prefix = if is_last { "└──" } else { "├──" };
            let next_indent = if is_last { "    " } else { "│   " };
            self.display_statement(
                stmt, 
                &format!("{}    {}", indent, s_prefix), 
                &format!("{}    {}", indent, next_indent)
            );
        }
    }

    fn display_statement(&self, stmt: &Statement, prefix: &str, indent: &str) {
        match stmt {
            Statement::Const(const_stmt) => {
                println!("{} const {} : {:?} =", prefix, const_stmt.name, const_stmt.typ);
                self.display_expression(&const_stmt.value, &format!("{}└──", indent));
            }
            Statement::Let(let_stmt) => {
                if let Some(value) = &let_stmt.value {
                    println!("{} let {} : {:?} =", prefix, let_stmt.name, let_stmt.typ);
                    self.display_expression(value, &format!("{}└──", indent));
                } else {
                    println!("{} let {} : {:?}", prefix, let_stmt.name, let_stmt.typ);
                }
            }
            Statement::Display(display_stmt) => {
                println!("{} display ({} expression(s))", prefix, display_stmt.expressions.len());
                for (i, expr) in display_stmt.expressions.iter().enumerate() {
                    let is_last = i == display_stmt.expressions.len() - 1;
                    let e_prefix = if is_last { "└──" } else { "├──" };
                    self.display_expression(expr, &format!("{}{}", indent, e_prefix));
                }
            }
            Statement::If(if_stmt) => {
                println!("{} if", prefix);
                println!("{}├── condition:", indent);
                self.display_expression(&if_stmt.condition, &format!("{}│   └──", indent));
                
                let then_count = if_stmt.then_block.statements.len();
                if if_stmt.else_block.is_some() {
                    println!("{}├── then: {} statement(s)", indent, then_count);
                } else {
                    println!("{}└── then: {} statement(s)", indent, then_count);
                }
                
                if let Some(else_block) = &if_stmt.else_block {
                    println!("{}└── else: {} statement(s)", indent, else_block.statements.len());
                }
            }
            Statement::While(while_stmt) => {
                println!("{} while", prefix);
                println!("{}├── condition:", indent);
                self.display_expression(&while_stmt.condition, &format!("{}│   └──", indent));
                println!("{}└── body: {} statement(s)", indent, while_stmt.body.statements.len());
            }
            Statement::For(for_stmt) => {
                println!("{} for", prefix);
                if for_stmt.init.is_some() {
                    println!("{}├── init: present", indent);
                }
                if for_stmt.condition.is_some() {
                    println!("{}├── condition: present", indent);
                }
                if for_stmt.update.is_some() {
                    println!("{}├── update: present", indent);
                }
                println!("{}└── body: {} statement(s)", indent, for_stmt.body.statements.len());
            }
            Statement::DoWhile(do_while) => {
                println!("{} do-while", prefix);
                println!("{}├── body: {} statement(s)", indent, do_while.body.statements.len());
                println!("{}└── condition:", indent);
                self.display_expression(&do_while.condition, &format!("{}    └──", indent));
            }
            Statement::Return(ret_stmt) => {
                if let Some(value) = &ret_stmt.value {
                    println!("{} send", prefix);
                    self.display_expression(value, &format!("{}└──", indent));
                } else {
                    println!("{} send (void)", prefix);
                }
            }
            Statement::Expression(expr_stmt) => {
                println!("{} expression:", prefix);
                self.display_expression(&expr_stmt.expression, &format!("{}└──", indent));
            }
            Statement::Block(block) => {
                println!("{} block ({} statement(s))", prefix, block.statements.len());
            }
            Statement::Break(_) => {
                println!("{} break", prefix);
            }
            Statement::Continue(_) => {
                println!("{} continue", prefix);
            }
        }
    }

    fn display_expression(&self, expr: &Expression, prefix: &str) {
        match expr {
            Expression::Literal(lit_expr) => {
                match &lit_expr.value {
                    Literal::Integer(n) => println!("{} {}", prefix, n),
                    Literal::Float(f) => println!("{} {}", prefix, f),
                    Literal::String(s) => println!("{} \"{}\"", prefix, s),
                    Literal::Boolean(b) => println!("{} {}", prefix, b),
                    Literal::Array(elements) => {
                        println!("{} array [{}]", prefix, elements.len());
                    }
                    Literal::InterpolatedString(parts) => {
                        println!("{} interpolated string ({} parts)", prefix, parts.len());
                    }
                }
            }
            Expression::Identifier(id_expr) => {
                println!("{} identifier: {}", prefix, id_expr.name);
            }
            Expression::Binary(bin) => {
                println!("{} {:?}", prefix, bin.op);
            }
            Expression::Unary(un) => {
                println!("{} {:?}", prefix, un.op);
            }
            Expression::Call(call) => {
                println!("{} call: {}({} args)", prefix, call.function, call.args.len());
            }
            Expression::Index(_) => {
                println!("{} array indexing", prefix);
            }
            Expression::Assign(assign) => {
                println!("{} assign to: {}", prefix, assign.target);
            }
        }
    }
}