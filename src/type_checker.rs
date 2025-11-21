// src/type_checker.rs - Type checking and semantic analysis with beautiful errors

use crate::ast::*;
use crate::symbol_table::{SymbolTable, Symbol, SymbolType as SymType, FunctionSignature};
use crate::errors::SemanticError;
use std::collections::HashMap;
use crate::errors::CompilerWarning;

/// Type checker with semantic analysis
pub struct TypeChecker {
    symbol_table: SymbolTable,
    errors: Vec<SemanticError>,
    warnings: Vec<CompilerWarning>,
    variable_usage: HashMap<String, bool>,
    current_function: Option<String>,
    current_return_type: Option<Type>,
    loop_depth: usize,
    has_return: bool,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            variable_usage: HashMap::new(),
            current_function: None,
            current_return_type: None,
            loop_depth: 0,
            has_return: false,
        }
    }
    
    /// Check entire program
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        // First pass: Register all functions
        for function in &program.functions {
            let _ = self.register_function(function);
        }
        
        // Second pass: Check function bodies
        for function in &program.functions {
            let _ = self.check_function(function);
        }
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    /// Register a function in the symbol table
    fn register_function(&mut self, function: &Function) -> Result<(), ()> {
        let params = function.params.iter()
            .map(|p| p.typ.clone())
            .collect();
        
        let sig = FunctionSignature {
            name: function.name.clone(),
            params,
            return_type: function.return_type.clone(),
        };
        
        if let Err(_) = self.symbol_table.register_function(sig) {
            // Function already defined
            self.errors.push(SemanticError::DuplicateDefinition {
                name: function.name.clone(),
                span: (function.span.start..function.span.end).into(),
                original: (function.span.start..function.span.end).into(), // TODO: Track actual original
            });
            return Err(());
        }
        
        Ok(())
    }
    
    /// Check a function
    fn check_function(&mut self, function: &Function) -> Result<(), ()> {
        // Set current function context
        self.current_function = Some(function.name.clone());
        self.current_return_type = function.return_type.clone();
        
        // Enter function scope
        self.symbol_table.enter_scope();
        
        // Add parameters to scope
        for param in &function.params {
            let symbol = Symbol {
                name: param.name.clone(),
                symbol_type: SymType::Parameter,
                data_type: param.typ.clone(),
                scope_level: self.symbol_table.current_scope_level(),
                defined_at: param.span.start,
            };
            
            if let Err(_) = self.symbol_table.insert(symbol) {
                self.errors.push(SemanticError::DuplicateDefinition {
                    name: param.name.clone(),
                    span: (param.span.start..param.span.end).into(),
                    original: (param.span.start..param.span.end).into(),
                });
            }
        }
        
        // Check function body
        let _ = self.check_block(&function.body);
        
        // Check if non-void function has return on all paths
        if function.return_type.is_some() {
            if !self.block_returns(&function.body) {
                self.errors.push(SemanticError::MissingReturn {
                    name: function.name.clone(),
                    return_type: format!("{:?}", function.return_type.as_ref().unwrap()),
                    span: (function.span.start..function.span.end).into(),
                });
            }
        }
        
        self.check_unused_variables();
        // Exit function scope
        self.symbol_table.exit_scope();
        
        // Clear function context
        self.current_function = None;
        self.current_return_type = None;
        
        Ok(())
    }
    
    /// Check a block
    fn check_block(&mut self, block: &Block) -> Result<(), ()> {
        let mut seen_return = false;
        
        for (_, statement) in block.statements.iter().enumerate() {
            if seen_return {
                // Code after return is unreachable
                self.warnings.push(CompilerWarning::UnreachableCode {
                    span: self.get_statement_span(statement).into(),
                    reason: "Code after 'send' statement will never execute".to_string(),
                });
            }
            
            let _ = self.check_statement(statement);
            
            if matches!(statement, Statement::Return(_)) {
                seen_return = true;
                self.has_return = true;
            }
        }
        Ok(())
    }
    
    /// Check a statement
    fn check_statement(&mut self, statement: &Statement) -> Result<(), ()> {
        match statement {
            Statement::Const(const_stmt) => self.check_const_statement(const_stmt),
            Statement::Let(let_stmt) => self.check_let_statement(let_stmt),
            Statement::Display(display_stmt) => self.check_display_statement(display_stmt),
            Statement::If(if_stmt) => self.check_if_statement(if_stmt),
            Statement::While(while_stmt) => self.check_while_statement(while_stmt),
            Statement::DoWhile(do_while) => self.check_do_while_statement(do_while),
            Statement::For(for_stmt) => self.check_for_statement(for_stmt),
            Statement::Return(return_stmt) => self.check_return_statement(return_stmt),
            Statement::Expression(expr_stmt) => {
                if let Expression::Call(call_expr) = &expr_stmt.expression{
                    if let Some(func_sig) = self.symbol_table.lookup_function(&call_expr.function).cloned() {
                        if call_expr.args.len() != func_sig.params.len(){
                            self.errors.push(SemanticError::ArgumentCountMismatch {
                                name: call_expr.function.clone(),
                                expected: func_sig.params.len(),
                                found: call_expr.args.len(),
                                span: (call_expr.span.start..call_expr.span.end).into(),
                            });
                            return Err(());
                        }
                        for (arg, expected) in call_expr.args.iter().zip(&func_sig.params) {
                            if let Ok(arg_type) = self.infer_expression_type(arg) {
                                if !self.types_compatible(expected, &arg_type) {
                                    self.errors.push(SemanticError::TypeMismatch {
                                        expected: format!("{:?}", expected),
                                        found: format!("{:?}", arg_type),
                                        span: (call_expr.span.start..call_expr.span.end).into(),
                                    });
                                }
                            }
                        }
                        
                        // Void function calls are OK in statement context
                        return Ok(());
                    }
                }
                let _ = self.check_expression(&expr_stmt.expression);
                Ok(())
            }
            Statement::Block(block) => {
                self.symbol_table.enter_scope();
                let _ = self.check_block(block);
                self.check_unused_variables();
                self.symbol_table.exit_scope();
                Ok(())
            }
            Statement::Break(break_stmt) => {
                if self.loop_depth == 0 {
                    self.errors.push(SemanticError::BreakOutsideLoop {
                        statement: "break".to_string(),
                        span: (break_stmt.span.start..break_stmt.span.end).into(),
                    });
                    return Err(());
                }
                Ok(())
            }

            Statement::Continue(continue_stmt) => {
                if self.loop_depth == 0 {
                    self.errors.push(SemanticError::BreakOutsideLoop {
                        statement: "continue".to_string(),
                        span: (continue_stmt.span.start..continue_stmt.span.end).into(),
                    });
                    return Err(());
                }
                Ok(())
            }
        }
    }

    /// Check const statement
    fn check_const_statement(&mut self, stmt: &ConstStmt) -> Result<(), ()> {
        // Check if const already exists in current scope
        if self.symbol_table.exists_in_current_scope(&stmt.name) {
            let original_span = self.symbol_table.lookup(&stmt.name)
                .map(|s| s.defined_at..s.defined_at + stmt.name.len())
                .unwrap_or(0..1);
            
            self.errors.push(SemanticError::DuplicateDefinition {
                name: stmt.name.clone(),
                span: (stmt.span.start..stmt.span.end).into(),
                original: original_span.into(),
            });
            return Err(());
        }
        
        // Validate the value expression type
        if let Ok(value_type) = self.infer_expression_type(&stmt.value) {
            if !self.types_compatible(&stmt.typ, &value_type) {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: format!("{:?}", stmt.typ),
                    found: format!("{:?}", value_type),
                    span: (stmt.span.start..stmt.span.end).into(),
                });
                return Err(());
            }
        }
        
        // Add constant to symbol table (mark as constant in SymbolType)
        let symbol = Symbol {
            name: stmt.name.clone(),
            symbol_type: SymType::Constant,  // We'll need to add Constant variant
            data_type: stmt.typ.clone(),
            scope_level: self.symbol_table.current_scope_level(),
            defined_at: stmt.span.start,
        };
        
        // Don't track constants as potentially unused
        // They're meant to be compile-time values
        
        if let Err(_) = self.symbol_table.insert(symbol) {
            self.errors.push(SemanticError::DuplicateDefinition {
                name: stmt.name.clone(),
                span: (stmt.span.start..stmt.span.end).into(),
                original: (stmt.span.start..stmt.span.end).into(),
            });
            return Err(());
        }
        
        Ok(())
    }
    
    /// Check let statement
    fn check_let_statement(&mut self, stmt: &LetStmt) -> Result<(), ()> {
        // Check if variable already exists in current scope
        if self.symbol_table.exists_in_current_scope(&stmt.name) {
            let original_span = self.symbol_table.lookup(&stmt.name)
                .map(|s| s.defined_at..s.defined_at + stmt.name.len())
                .unwrap_or(0..1);
            
            self.errors.push(SemanticError::DuplicateDefinition {
                name: stmt.name.clone(),
                span: (stmt.span.start..stmt.span.end).into(),
                original: original_span.into(),
            });
            return Err(());
        }
        
        // If there's an initializer, check its type
        if let Some(ref value) = stmt.value {
            if let Ok(value_type) = self.infer_expression_type(value) {
                // Check type compatibility
                if !self.types_compatible(&stmt.typ, &value_type) {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: format!("{:?}", stmt.typ),
                        found: format!("{:?}", value_type),
                        span: (stmt.span.start..stmt.span.end).into(),
                    });
                    return Err(());
                }
            }
        }
        
        // Add variable to symbol table
        let symbol = Symbol {
            name: stmt.name.clone(),
            symbol_type: SymType::Variable,
            data_type: stmt.typ.clone(),
            scope_level: self.symbol_table.current_scope_level(),
            defined_at: stmt.span.start,
        };
        self.variable_usage.insert(stmt.name.clone(), false);
        
        if let Err(_) = self.symbol_table.insert(symbol) {
            self.errors.push(SemanticError::DuplicateDefinition {
                name: stmt.name.clone(),
                span: (stmt.span.start..stmt.span.end).into(),
                original: (stmt.span.start..stmt.span.end).into(),
            });
            return Err(());
        }
        
        Ok(())
    }
    
    /// Check display statement
    fn check_display_statement(&mut self, stmt: &DisplayStmt) -> Result<(), ()> {
        for expr in &stmt.expressions {
            let _ = self.check_expression(expr);
        }
        Ok(())
    }
    
    /// Check if statement
    fn check_if_statement(&mut self, stmt: &IfStmt) -> Result<(), ()> {
        // Check condition is boolean
        if let Ok(cond_type) = self.infer_expression_type(&stmt.condition) {
            if cond_type != Type::Bool {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "Bool".to_string(),
                    found: format!("{:?}", cond_type),
                    span: (stmt.span.start..stmt.span.end).into(),
                });
            }
        }
        
        // Check then block
        self.symbol_table.enter_scope();
        let _ = self.check_block(&stmt.then_block);
        self.check_unused_variables();
        self.symbol_table.exit_scope();
        
        // Check else block if present
        if let Some(else_block) = &stmt.else_block {
            self.symbol_table.enter_scope();
            let _ = self.check_block(else_block);
            self.check_unused_variables();
            self.symbol_table.exit_scope();
        }
        
        Ok(())
    }
    
    /// Check while statement
    fn check_while_statement(&mut self, stmt: &WhileStmt) -> Result<(), ()> {
        // Check condition is boolean
        if let Ok(cond_type) = self.infer_expression_type(&stmt.condition) {
            if cond_type != Type::Bool {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "Bool".to_string(),
                    found: format!("{:?}", cond_type),
                    span: (stmt.span.start..stmt.span.end).into(),
                });
            }
        }
        
        // Check body
        self.symbol_table.enter_scope();
        self.loop_depth += 1;
        let _ = self.check_block(&stmt.body);
        self.loop_depth -= 1;
        self.check_unused_variables();
        self.symbol_table.exit_scope();
        
        Ok(())
    }
    
    /// Check do-while statement
    fn check_do_while_statement(&mut self, stmt: &DoWhileStmt) -> Result<(), ()> {
        // Check body
        self.symbol_table.enter_scope();
        self.loop_depth += 1;
        let _ = self.check_block(&stmt.body);
        self.loop_depth -= 1;
        self.check_unused_variables();
        self.symbol_table.exit_scope();
        
        // Check condition is boolean
        if let Ok(cond_type) = self.infer_expression_type(&stmt.condition) {
            if cond_type != Type::Bool {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "Bool".to_string(),
                    found: format!("{:?}", cond_type),
                    span: (stmt.span.start..stmt.span.end).into(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Check for statement
    fn check_for_statement(&mut self, stmt: &ForStmt) -> Result<(), ()> {
        self.symbol_table.enter_scope();
        
        // Check init
        if let Some(init) = &stmt.init {
            let _ = self.check_statement(init);
        }
        
        // Check condition
        if let Some(condition) = &stmt.condition {
            if let Ok(cond_type) = self.infer_expression_type(condition) {
                if cond_type != Type::Bool {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "Bool".to_string(),
                        found: format!("{:?}", cond_type),
                        span: (stmt.span.start..stmt.span.end).into(),
                    });
                }
            }
        }
        
        // Check update
        if let Some(update) = &stmt.update {
            let _ = self.check_expression(update);
        }
        
        // Check body
        self.loop_depth += 1;
        let _ = self.check_block(&stmt.body);
        self.loop_depth -= 1;
        self.check_unused_variables();
        self.symbol_table.exit_scope();
        Ok(())
    }
    
    /// Check return statement
    fn check_return_statement(&mut self, stmt: &ReturnStmt) -> Result<(), ()> {
        let current_return_type = self.current_return_type.clone();
        match (&stmt.value, &current_return_type) {
            (Some(value), Some(expected_type)) => {
                // Function expects a return value
                if let Ok(value_type) = self.infer_expression_type(value) {
                    if !self.types_compatible(expected_type, &value_type) {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{:?}", expected_type),
                            found: format!("{:?}", value_type),
                            span: (stmt.span.start..stmt.span.end).into(),
                        });
                    }
                }
            }
            (None, Some(expected_type)) => {
                // Function expects a value but got none
                self.errors.push(SemanticError::TypeMismatch {
                    expected: format!("{:?}", expected_type),
                    found: "void".to_string(),
                    span: (stmt.span.start..stmt.span.end).into(),
                });
            }
            (Some(_), None) => {
                // Void function trying to return a value
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "void".to_string(),
                    found: "some value".to_string(),
                    span: (stmt.span.start..stmt.span.end).into(),
                });
            }
            (None, None) => {
                // Void return in void function - OK
            }
        }
        
        Ok(())
    }
    
    /// Check expression (just validate, don't need type)
    fn check_expression(&mut self, expr: &Expression) -> Result<(), ()> {
        let _ = self.infer_expression_type(expr);
        Ok(())
    }
    
    /// Infer the type of an expression
    fn infer_expression_type(&mut self, expr: &Expression) -> Result<Type, ()> {
        match expr {
            Expression::Literal(lit_expr) => self.literal_type(&lit_expr.value, &lit_expr.span),
            
            Expression::Identifier(id_expr) => {
                if let Some(symbol) = self.symbol_table.lookup(&id_expr.name) {
                    let data_type = symbol.data_type.clone(); 
                    self.mark_variable_used(&id_expr.name);
                    Ok(data_type)
                } else {
                    // Find similar variable names
                    let similar = self.symbol_table.find_similar_names(&id_expr.name, 3);
                    let suggestion = if !similar.is_empty() {
                        format!("Did you mean: {}?", similar.join(", "))
                    } else {
                        "Did you forget to declare this variable with 'let'?".to_string()
                    };
                    
                    self.errors.push(SemanticError::UndefinedVariable {
                        name: id_expr.name.clone(),
                        span: (id_expr.span.start..id_expr.span.end).into(),
                        suggestion,
                        context: self.get_context(),
                    });
                    Err(())
                }
            }
            
            Expression::Binary(binary) => self.infer_binary_type(binary),
            
            Expression::Unary(unary) => self.infer_unary_type(unary),
            
            Expression::Call(call) => self.infer_call_type(call),
            
            Expression::Index(index) => self.infer_index_type(index),
            
            Expression::Assign(assign) => self.infer_assign_type(assign),
        }
    }
    
    /// Get type of literal
    fn literal_type(&mut self, lit: &Literal , span: &Span) -> Result<Type , ()> {
        match lit {
            Literal::Integer(_) => Ok(Type::Int),
            Literal::Float(_) => Ok(Type::Float),
            Literal::String(_) => Ok(Type::String),
            Literal::Boolean(_) => Ok(Type::Bool),
            Literal::InterpolatedString(parts) => {
                for part in parts {
                    if let StringPart::Expression(expr) = part {
                        // Verify expression has a valid type
                        let _ = self.infer_expression_type(expr)?;
                    }
                }
                Ok(Type::String)
            }
            Literal::Array(elements) => {
                if elements.is_empty() {
                    Ok(Type::Array(Box::new(Type::Int), 0))
                } else {
                    // Infer type from first element
                    let first_type = self.infer_expression_type(&elements[0])?;
                    
                    // Verify all elements have same type
                    for element in &elements[1..] {
                        let elem_type = self.infer_expression_type(element)?;
                        if elem_type != first_type {
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: format!("{:?}", first_type),
                                found: format!("{:?}", elem_type),
                                span: (span.start..span.end).into(), // You'd use proper span
                            });
                            return Err(());
                        }
                    }
                    
                    Ok(Type::Array(Box::new(first_type), elements.len()))
                }
            }
        }
    }
    
    /// Infer type of binary expression
    fn infer_binary_type(&mut self, binary: &BinaryExpr) -> Result<Type, ()> {
        let left_type = self.infer_expression_type(&binary.left)?;
        let right_type = self.infer_expression_type(&binary.right)?;
        
        match binary.op {
            // Arithmetic operators
            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                if left_type == right_type && (left_type == Type::Int || left_type == Type::Float) {
                    Ok(left_type)
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: format!("{:?}", left_type),
                        found: format!("{:?}", right_type),
                        span: (binary.span.start..binary.span.end).into(),
                    });
                    Err(())
                }
            }
            
            // Comparison operators
            BinaryOp::Equal | BinaryOp::NotEqual => {
                if self.types_compatible(&left_type, &right_type) {
                    Ok(Type::Bool)
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: format!("{:?}", left_type),
                        found: format!("{:?}", right_type),
                        span: (binary.span.start..binary.span.end).into(),
                    });
                    Err(())
                }
            }
            
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual => {
                if left_type == right_type && (left_type == Type::Int || left_type == Type::Float) {
                    Ok(Type::Bool)
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: format!("{:?}", left_type),
                        found: format!("{:?}", right_type),
                        span: (binary.span.start..binary.span.end).into(),
                    });
                    Err(())
                }
            }
            
            // Logical operators
            BinaryOp::And | BinaryOp::Or => {
                if left_type == Type::Bool && right_type == Type::Bool {
                    Ok(Type::Bool)
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "Bool".to_string(),
                        found: format!("{:?} and {:?}", left_type, right_type),
                        span: (binary.span.start..binary.span.end).into(),
                    });
                    Err(())
                }
            }
        }
    }
    
    /// Infer type of unary expression
    fn infer_unary_type(&mut self, unary: &UnaryExpr) -> Result<Type, ()> {
        let operand_type = self.infer_expression_type(&unary.operand)?;
        
        match unary.op {
            UnaryOp::Not => {
                if operand_type == Type::Bool {
                    Ok(Type::Bool)
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "Bool".to_string(),
                        found: format!("{:?}", operand_type),
                        span: (unary.span.start..unary.span.end).into(),
                    });
                    Err(())
                }
            }
            UnaryOp::Negate => {
                if operand_type == Type::Int || operand_type == Type::Float {
                    Ok(operand_type)
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "Int or Float".to_string(),
                        found: format!("{:?}", operand_type),
                        span: (unary.span.start..unary.span.end).into(),
                    });
                    Err(())
                }
            }
        }
    }
    
    /// Infer type of function call
    fn infer_call_type(&mut self, call: &CallExpr) -> Result<Type, ()> {
        if let Some(func_sig) = self.symbol_table.lookup_function(&call.function).cloned() {
            // Check argument count
            if call.args.len() != func_sig.params.len() {
                self.errors.push(SemanticError::ArgumentCountMismatch {
                    name: call.function.clone(),
                    expected: func_sig.params.len(),
                    found: call.args.len(),
                    span: (call.span.start..call.span.end).into(),
                });
                return Err(());
            }
            
            // Check argument types
            for (_, (arg, expected)) in call.args.iter().zip(&func_sig.params).enumerate() {
                if let Ok(arg_type) = self.infer_expression_type(arg) {
                    if !self.types_compatible(expected, &arg_type) {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{:?}", expected),
                            found: format!("{:?}", arg_type),
                            span: (call.span.start..call.span.end).into(),
                        });
                    }
                }
            }
            
            // Return function's return type
            if let Some(ret_type) = func_sig.return_type {
                Ok(ret_type)
            } else {
                // Void function used in expression context
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "some return type".to_string(),
                    found: "void".to_string(),
                    span: (call.span.start..call.span.end).into(),
                });
                Err(())
            }
        } else {
            // Find similar function names
            let similar = self.symbol_table.find_similar_functions(&call.function, 3);
            let suggestion = if !similar.is_empty() {
                format!("Did you mean: {}?", similar.join(", "))
            } else {
                format!("Function '{}' is not defined. Check spelling or define the function.", call.function)
            };
            
            self.errors.push(SemanticError::UndefinedFunction {
                name: call.function.clone(),
                span: (call.span.start..call.span.end).into(),
                suggestion,
                context: self.get_context(),
            });
            Err(())
        }
    }
    
    /// Infer type of array indexing
    fn infer_index_type(&mut self, index: &IndexExpr) -> Result<Type, ()> {
        let array_type = self.infer_expression_type(&index.array)?;
        let index_type = self.infer_expression_type(&index.index)?;
        
        // Check index is integer
        if index_type != Type::Int {
            self.errors.push(SemanticError::TypeMismatch {
                expected: "Int".to_string(),
                found: format!("{:?}", index_type),
                span: (index.span.start..index.span.end).into(),
            });
        }
        
        // Extract element type from array type
        match array_type {
            Type::Array(elem_type, _) => Ok(*elem_type),
            _ => {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "Array".to_string(),
                    found: format!("{:?}", array_type),
                    span: (index.span.start..index.span.end).into(),
                });
                Err(())
            }
        }
    }
    
    /// Infer type of assignment
    fn infer_assign_type(&mut self, assign: &AssignExpr) -> Result<Type, ()> {
        if assign.target.starts_with("__ARRAY_INDEX__:") {
        // Just validate the value expression
            return self.infer_expression_type(&assign.value);
        }
        
        if let Some(symbol) = self.symbol_table.lookup(&assign.target) {
            if symbol.symbol_type == SymType::Constant {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: "mutable variable".to_string(),
                    found: "constant (cannot be reassigned)".to_string(),
                    span: (assign.span.start..assign.span.end).into(),
                });
                return Err(());
            }
            let target_type = symbol.data_type.clone();
            
            if let Ok(value_type) = self.infer_expression_type(&assign.value) {
                if !self.types_compatible(&target_type, &value_type) {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: format!("{:?}", target_type),
                        found: format!("{:?}", value_type),
                        span: (assign.span.start..assign.span.end).into(),
                    });
                    return Err(());
                }
            }
            
            Ok(target_type)
        } else {
            let similar = self.symbol_table.find_similar_names(&assign.target, 3);
            let suggestion = if !similar.is_empty() {
                format!("Did you mean: {}?", similar.join(", "))
            } else {
                "Did you forget to declare this variable with 'let'?".to_string()
            };
            
            self.errors.push(SemanticError::UndefinedVariable {
                name: assign.target.clone(),
                span: (assign.span.start..assign.span.end).into(),
                suggestion,
                context: self.get_context(),
            });
            Err(())
        }
    }
    
    /// Check if two types are compatible
    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        expected == actual
        // TODO: Add implicit conversions if needed (e.g., int to float)
    }
    
    /// Check if block has return statement on all paths
    fn block_returns(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            if self.statement_returns(stmt) {
                return true;
            }
        }
        false
    }
    
    /// Check if statement returns
    fn statement_returns(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::Return(_) => true,
            Statement::If(if_stmt) => {
                // Both branches must return
                if let Some(else_block) = &if_stmt.else_block {
                    self.block_returns(&if_stmt.then_block) && self.block_returns(else_block)
                } else {
                    false
                }
            }
            Statement::Block(block) => self.block_returns(block),
            _ => false,
        }
    }
    
    /// Get collected errors
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Mark a variable as used
    fn mark_variable_used(&mut self, name: &str) {
        self.variable_usage.insert(name.to_string(), true);
    }

    /// Check for unused variables at scope exit
    fn check_unused_variables(&mut self) {
        for (name, symbol) in self.symbol_table.current_scope_symbols() {
            if !self.variable_usage.get(&name).unwrap_or(&false) {
                // Don't warn about parameters or special variables
                if !matches!(symbol.symbol_type, SymType::Parameter) && !name.starts_with("_") {
                    self.warnings.push(CompilerWarning::UnusedVariable {
                        name: name.clone(),
                        span: (symbol.defined_at..symbol.defined_at + name.len()).into(),
                        defined_at: (symbol.defined_at..symbol.defined_at + name.len()).into(),
                    });
                }
            }
        }
    }

    /// Get warnings
    pub fn get_warnings(&self) -> &[CompilerWarning] {
        &self.warnings
    }

    /// Get current context string
    fn get_context(&self) -> Option<String> {
        self.current_function.as_ref().map(|f| format!("function '{}'", f))
    }

    fn get_statement_span(&self, stmt: &Statement) -> std::ops::Range<usize> {
        match stmt {
            Statement::Const(s) => s.span.start..s.span.end,
            Statement::Let(s) => s.span.start..s.span.end,
            Statement::Display(s) => s.span.start..s.span.end,
            Statement::If(s) => s.span.start..s.span.end,
            Statement::While(s) => s.span.start..s.span.end,
            Statement::DoWhile(s) => s.span.start..s.span.end,
            Statement::For(s) => s.span.start..s.span.end,
            Statement::Return(s) => s.span.start..s.span.end,
            Statement::Expression(s) => s.span.start..s.span.end,
            Statement::Block(s) => s.span.start..s.span.end,
            Statement::Break(s) => s.span.start..s.span.end,
            Statement::Continue(s) => s.span.start..s.span.end,
        }
    }

}