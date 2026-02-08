// src/optimizer.rs - Code optimization passes

use crate::ast::*;
use std::collections::{HashMap,HashSet};

/// Statistics about optimizations performed
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub constants_folded: usize,
    pub dead_code_removed: usize,
    pub constants_propagated: usize,
    pub strength_reductions: usize,
}

/// The optimizer - performs multiple optimization passes on the AST
pub struct Optimizer {
    optimization_level: u8,  // 0 = none, 1 = basic, 2 = aggressive
    stats: OptimizationStats,
    // Track constant variables for propagation
    constant_values: HashMap<String, Literal>,
}

impl Optimizer {
    pub fn new(level: u8) -> Self {
        Self {
            optimization_level: level,
            stats: OptimizationStats::default(),
            constant_values: HashMap::new(),
        }
    }

    fn is_power_of_two(&self, n: i32) -> bool {
        n > 0 && (n & (n - 1)) == 0
    }
    
    /// Run all optimization passes on the program
    pub fn optimize(&mut self, program: &mut Program) -> OptimizationStats {
        // If optimization is disabled, return immediately
        if self.optimization_level == 0 {
            return self.stats.clone();
        }
        
        // Run optimization passes on each function
        for function in &mut program.functions {
            self.optimize_function(function);
        }
        
        self.stats.clone()
    }
    
    /// Optimize a single function
    fn optimize_function(&mut self, function: &mut Function) {
        self.constant_values.clear();

        if self.optimization_level >= 1 {
            self.apply_strength_reduction_to_block(&mut function.body);
        }
        
        self.fold_constants_in_block(&mut function.body);
        
        if self.optimization_level >= 2 {
            self.track_constants_in_block(&function.body);
            
            self.propagate_constants_in_block(&mut function.body);
            
            self.fold_constants_in_block(&mut function.body);

            self.apply_strength_reduction_to_block(&mut function.body);

            self.fold_constants_in_block(&mut function.body);
        }
        self.eliminate_dead_code_in_block(&mut function.body);
    }
    
    /// Fold constants in a block of statements
    fn fold_constants_in_block(&mut self, block: &mut Block) {
        for statement in &mut block.statements {
            self.fold_constants_in_statement(statement);
        }
    }
    
    /// Fold constants in a single statement
    fn fold_constants_in_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::Let(let_stmt) => {
                // Fold constants in the initializer
                if let Some(ref mut value) = let_stmt.value {
                    self.fold_constants_in_expression(value);
                }
            }
            Statement::Const(const_stmt) => {
                // Fold constants in const value
                self.fold_constants_in_expression(&mut const_stmt.value);
            }
            Statement::Display(display_stmt) => {
                // Fold constants in each expression
                for expr in &mut display_stmt.expressions {
                    self.fold_constants_in_expression(expr);
                }
            }
            Statement::If(if_stmt) => {
                // Fold in condition and both branches
                self.fold_constants_in_expression(&mut if_stmt.condition);
                self.fold_constants_in_block(&mut if_stmt.then_block);
                if let Some(ref mut else_block) = if_stmt.else_block {
                    self.fold_constants_in_block(else_block);
                }
            }
            Statement::While(while_stmt) => {
                self.fold_constants_in_expression(&mut while_stmt.condition);
                self.fold_constants_in_block(&mut while_stmt.body);
            }
            Statement::DoWhile(do_while) => {
                self.fold_constants_in_block(&mut do_while.body);
                self.fold_constants_in_expression(&mut do_while.condition);
            }
            Statement::For(for_stmt) => {
                if let Some(ref mut init) = for_stmt.init {
                    self.fold_constants_in_statement(init);
                }
                if let Some(ref mut condition) = for_stmt.condition {
                    self.fold_constants_in_expression(condition);
                }
                if let Some(ref mut update) = for_stmt.update {
                    self.fold_constants_in_expression(update);
                }
                self.fold_constants_in_block(&mut for_stmt.body);
            }
            Statement::Return(return_stmt) => {
                if let Some(ref mut value) = return_stmt.value {
                    self.fold_constants_in_expression(value);
                }
            }
            Statement::Expression(expr_stmt) => {
                self.fold_constants_in_expression(&mut expr_stmt.expression);
            }
            Statement::Block(block) => {
                self.fold_constants_in_block(block);
            }
            Statement::Break(_) | Statement::Continue(_) => {
                // Nothing to optimize
            }
        }
    }
    
    /// Fold constants in an expression (the core optimization)
    fn fold_constants_in_expression(&mut self, expr: &mut Expression) {
        // First, recursively fold any sub-expressions
        match expr {
            Expression::Binary(binary) => {
                // Fold left and right first
                self.fold_constants_in_expression(&mut binary.left);
                self.fold_constants_in_expression(&mut binary.right);
                
                // Now try to fold this binary operation
                if let Some(folded) = self.try_fold_binary(binary) {
                    // Replace the entire binary expression with the constant
                    *expr = Expression::Literal(LiteralExpr {
                        value: folded,
                        span: binary.span.clone(),
                    });
                    self.stats.constants_folded += 1;
                }
            }
            Expression::Unary(unary) => {
                // Fold the operand first
                self.fold_constants_in_expression(&mut unary.operand);
                
                // Try to fold this unary operation
                if let Some(folded) = self.try_fold_unary(unary) {
                    *expr = Expression::Literal(LiteralExpr {
                        value: folded,
                        span: unary.span.clone(),
                    });
                    self.stats.constants_folded += 1;
                }
            }
            Expression::Call(call) => {
                // Fold arguments
                for arg in &mut call.args {
                    self.fold_constants_in_expression(arg);
                }
            }
            Expression::Index(index) => {
                self.fold_constants_in_expression(&mut index.array);
                self.fold_constants_in_expression(&mut index.index);
            }
            Expression::Assign(assign) => {
                self.fold_constants_in_expression(&mut assign.value);
            }
            Expression::Literal(_) | Expression::Identifier(_) => {
                // Already constants or variables, nothing to fold
            }
        }
    }
    
    /// Try to fold a binary operation if both operands are constants
    fn try_fold_binary(&self, binary: &BinaryExpr) -> Option<Literal> {
        // Extract literal values from both sides
        let left_lit = if let Expression::Literal(lit) = &*binary.left {
            Some(&lit.value)
        } else {
            None
        };
        
        let right_lit = if let Expression::Literal(lit) = &*binary.right {
            Some(&lit.value)
        } else {
            None
        };
        
        // If both are literals, try to fold
        match (left_lit, &binary.op, right_lit) {
            // Integer arithmetic with overflow checking
            (Some(Literal::Integer(l)), BinaryOp::Add, Some(Literal::Integer(r))) => {
                l.checked_add(*r).map(Literal::Integer)
            }
            (Some(Literal::Integer(l)), BinaryOp::Subtract, Some(Literal::Integer(r))) => {
                l.checked_sub(*r).map(Literal::Integer)
            }
            (Some(Literal::Integer(l)), BinaryOp::Multiply, Some(Literal::Integer(r))) => {
                l.checked_mul(*r).map(Literal::Integer)
            }
            (Some(Literal::Integer(l)), BinaryOp::Divide, Some(Literal::Integer(r))) if *r != 0 => {
                l.checked_div(*r).map(Literal::Integer)
            }
            (Some(Literal::Integer(l)), BinaryOp::Modulo, Some(Literal::Integer(r))) if *r != 0 => {
                l.checked_rem(*r).map(Literal::Integer)
            }
            
            // Rest of the cases remain the same...
            // Float arithmetic
            (Some(Literal::Float(l)), BinaryOp::Add, Some(Literal::Float(r))) => {
                Some(Literal::Float(l + r))
            }
            (Some(Literal::Float(l)), BinaryOp::Subtract, Some(Literal::Float(r))) => {
                Some(Literal::Float(l - r))
            }
            (Some(Literal::Float(l)), BinaryOp::Multiply, Some(Literal::Float(r))) => {
                Some(Literal::Float(l * r))
            }
            (Some(Literal::Float(l)), BinaryOp::Divide, Some(Literal::Float(r))) if *r != 0.0 => {
                Some(Literal::Float(l / r))
            }
            
            // Integer comparisons
            (Some(Literal::Integer(l)), BinaryOp::Equal, Some(Literal::Integer(r))) => {
                Some(Literal::Boolean(l == r))
            }
            (Some(Literal::Integer(l)), BinaryOp::NotEqual, Some(Literal::Integer(r))) => {
                Some(Literal::Boolean(l != r))
            }
            (Some(Literal::Integer(l)), BinaryOp::Less, Some(Literal::Integer(r))) => {
                Some(Literal::Boolean(l < r))
            }
            (Some(Literal::Integer(l)), BinaryOp::Greater, Some(Literal::Integer(r))) => {
                Some(Literal::Boolean(l > r))
            }
            (Some(Literal::Integer(l)), BinaryOp::LessEqual, Some(Literal::Integer(r))) => {
                Some(Literal::Boolean(l <= r))
            }
            (Some(Literal::Integer(l)), BinaryOp::GreaterEqual, Some(Literal::Integer(r))) => {
                Some(Literal::Boolean(l >= r))
            }
            
            // Boolean operations
            (Some(Literal::Boolean(l)), BinaryOp::And, Some(Literal::Boolean(r))) => {
                Some(Literal::Boolean(*l && *r))
            }
            (Some(Literal::Boolean(l)), BinaryOp::Or, Some(Literal::Boolean(r))) => {
                Some(Literal::Boolean(*l || *r))
            }
            
            // Can't fold
            _ => None
        }
    }
    
    /// Try to fold a unary operation
    fn try_fold_unary(&self, unary: &UnaryExpr) -> Option<Literal> {
        let operand_lit = if let Expression::Literal(lit) = &*unary.operand {
            Some(&lit.value)
        } else {
            None
        };
        
        match (&unary.op, operand_lit) {
            (UnaryOp::Negate, Some(Literal::Integer(n))) => Some(Literal::Integer(-n)),
            (UnaryOp::Negate, Some(Literal::Float(f))) => Some(Literal::Float(-f)),
            (UnaryOp::Not, Some(Literal::Boolean(b))) => Some(Literal::Boolean(!b)),
            _ => None
        }
    }
    
    /// Track constant assignments for propagation
    fn track_constants_in_block(&mut self, block: &Block) {
        let reassigned = self.collect_assigned_variables_in_block(block);
        
        // Second: only track variables that are never reassigned
        for statement in &block.statements {
            match statement {
                Statement::Const(const_stmt) => {
                    match &const_stmt.value {
                        Expression::Literal(lit) => {
                            self.constant_values.insert(
                                const_stmt.name.clone(),
                                lit.value.clone(),
                            );
                        }
                        Expression::Binary(binary) => {
                            if let Some(folded) = self.try_fold_binary(binary) {
                                self.constant_values.insert(
                                    const_stmt.name.clone(),
                                    folded,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                Statement::Let(let_stmt) => {
                    // Only track if this variable is NEVER reassigned
                    if !reassigned.contains(&let_stmt.name) {
                        if let Some(Expression::Literal(lit)) = &let_stmt.value {
                            self.constant_values.insert(
                                let_stmt.name.clone(),
                                lit.value.clone(),
                            );
                        }
                    }
                }
                _ => {
                    // Other statements don't introduce new constant bindings
                }
            }
        }
    }

    /// Collect all variable names that are reassigned anywhere in a block
    fn collect_assigned_variables_in_block(&self, block: &Block) -> HashSet<String> {
        let mut assigned = HashSet::new();
        for statement in &block.statements {
            self.collect_assigned_variables_in_statement(statement, &mut assigned);
        }
        assigned
    }

    /// Recursively find assignments in a statement
    fn collect_assigned_variables_in_statement(
        &self,
        statement: &Statement,
        assigned: &mut HashSet<String>,
    ) {
        match statement {
            Statement::Expression(expr_stmt) => {
                self.collect_assigned_variables_in_expression(&expr_stmt.expression, assigned);
            }
            Statement::If(if_stmt) => {
                for stmt in &if_stmt.then_block.statements {
                    self.collect_assigned_variables_in_statement(stmt, assigned);
                }
                if let Some(ref else_block) = if_stmt.else_block {
                    for stmt in &else_block.statements {
                        self.collect_assigned_variables_in_statement(stmt, assigned);
                    }
                }
            }
            Statement::While(while_stmt) => {
                for stmt in &while_stmt.body.statements {
                    self.collect_assigned_variables_in_statement(stmt, assigned);
                }
            }
            Statement::DoWhile(do_while) => {
                for stmt in &do_while.body.statements {
                    self.collect_assigned_variables_in_statement(stmt, assigned);
                }
            }
            Statement::For(for_stmt) => {
                if let Some(ref init) = for_stmt.init {
                    self.collect_assigned_variables_in_statement(init, assigned);
                }
                if let Some(ref update) = for_stmt.update {
                    self.collect_assigned_variables_in_expression(update, assigned);
                }
                for stmt in &for_stmt.body.statements {
                    self.collect_assigned_variables_in_statement(stmt, assigned);
                }
            }
            Statement::Block(block) => {
                for stmt in &block.statements {
                    self.collect_assigned_variables_in_statement(stmt, assigned);
                }
            }
            // Let, Const, Display, Return, Break, Continue
            // don't reassign existing variables
            _ => {}
        }
    }

    /// Recursively find assignments in an expression
    fn collect_assigned_variables_in_expression(
        &self,
        expr: &Expression,
        assigned: &mut HashSet<String>,
    ) {
        match expr {
            Expression::Assign(assign) => {
                if !assign.target.starts_with("__ARRAY_INDEX__:") {
                    assigned.insert(assign.target.clone());
                }
                self.collect_assigned_variables_in_expression(&assign.value, assigned);
            }
            Expression::Binary(binary) => {
                self.collect_assigned_variables_in_expression(&binary.left, assigned);
                self.collect_assigned_variables_in_expression(&binary.right, assigned);
            }
            Expression::Unary(unary) => {
                self.collect_assigned_variables_in_expression(&unary.operand, assigned);
            }
            Expression::Call(call) => {
                for arg in &call.args {
                    self.collect_assigned_variables_in_expression(arg, assigned);
                }
            }
            Expression::Index(index) => {
                self.collect_assigned_variables_in_expression(&index.array, assigned);
                self.collect_assigned_variables_in_expression(&index.index, assigned);
            }
            Expression::Literal(_) | Expression::Identifier(_) => {}
        }
    }
    
    fn propagate_constants_in_block(&mut self, block: &mut Block) {
        for statement in &mut block.statements {
            self.propagate_constants_in_statement(statement);
        }
    }

    fn propagate_constants_in_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::Const(const_stmt) => {
                self.propagate_in_expression(&mut const_stmt.value);
            }
            Statement::Let(let_stmt) => {
                if let Some(ref mut value) = let_stmt.value {
                    self.propagate_in_expression(value);
                }
            }
            Statement::Display(display_stmt) => {
                for expr in &mut display_stmt.expressions {
                    self.propagate_in_expression(expr);
                }
            }
            Statement::If(if_stmt) => {
                self.propagate_in_expression(&mut if_stmt.condition);
                self.propagate_constants_in_block(&mut if_stmt.then_block);
                if let Some(ref mut else_block) = if_stmt.else_block {
                    self.propagate_constants_in_block(else_block);
                }
            }
            Statement::While(while_stmt) => {
                self.propagate_in_expression(&mut while_stmt.condition);
                self.propagate_constants_in_block(&mut while_stmt.body);
            }
            Statement::DoWhile(do_while) => {
                self.propagate_constants_in_block(&mut do_while.body);
                self.propagate_in_expression(&mut do_while.condition);
            }
            Statement::For(for_stmt) => {
                if let Some(ref mut init) = for_stmt.init {
                    self.propagate_constants_in_statement(init);
                }
                if let Some(ref mut condition) = for_stmt.condition {
                    self.propagate_in_expression(condition);
                }
                if let Some(ref mut update) = for_stmt.update {
                    self.propagate_in_expression(update);
                }
                self.propagate_constants_in_block(&mut for_stmt.body);
            }
            Statement::Return(return_stmt) => {
                if let Some(ref mut value) = return_stmt.value {
                    self.propagate_in_expression(value);
                }
            }
            Statement::Expression(expr_stmt) => {
                self.propagate_in_expression(&mut expr_stmt.expression);
            }
            Statement::Block(block) => {
                self.propagate_constants_in_block(block);
            }
            Statement::Break(_) | Statement::Continue(_) => {
                // Nothing to propagate
            }
        }
    }

    fn propagate_in_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Identifier(id) => {
                // Replace identifier with constant value if available
                if let Some(constant_value) = self.constant_values.get(&id.name) {
                    *expr = Expression::Literal(LiteralExpr {
                        value: constant_value.clone(),
                        span: id.span.clone(),
                    });
                    self.stats.constants_propagated += 1;
                }
            }
            Expression::Binary(binary) => {
                self.propagate_in_expression(&mut binary.left);
                self.propagate_in_expression(&mut binary.right);
            }
            Expression::Unary(unary) => {
                self.propagate_in_expression(&mut unary.operand);
            }
            Expression::Call(call) => {
                for arg in &mut call.args {
                    self.propagate_in_expression(arg);
                }
            }
            Expression::Index(index) => {
                self.propagate_in_expression(&mut index.array);
                self.propagate_in_expression(&mut index.index);
            }
            Expression::Assign(assign) => {
                self.propagate_in_expression(&mut assign.value);
            }
            Expression::Literal(_) => {
                // Already a literal, nothing to propagate
            }
        }
    }
    
    /// Eliminate dead code in a block of statements
    fn eliminate_dead_code_in_block(&mut self, block: &mut Block) {
        let mut new_statements = Vec::new();
        let mut is_dead = false;
        
        for statement in block.statements.drain(..) {
            if is_dead {
                // Everything after a return/break/continue is dead
                self.stats.dead_code_removed += 1;
                continue;
            }
            
            // Check if this statement makes subsequent code dead
            match &statement {
                Statement::Return(_) | Statement::Break(_) | Statement::Continue(_) => {
                    is_dead = true;
                    new_statements.push(statement);
                }
                Statement::If(if_stmt) => {
                    // Check if condition is a constant
                    if let Expression::Literal(lit) = &if_stmt.condition {
                        if let Literal::Boolean(val) = &lit.value {
                            if *val {
                                // Condition is always true - eliminate else block
                                if if_stmt.else_block.is_some() {
                                    self.stats.dead_code_removed += 1;
                                    
                                    // Convert to just the then block
                                    let mut modified_if = if_stmt.clone();
                                    modified_if.else_block = None;
                                    new_statements.push(Statement::If(modified_if));
                                } else {
                                    new_statements.push(statement);
                                }
                            } else {
                                // Condition is always false - eliminate then block
                                self.stats.dead_code_removed += 1;
                                
                                if let Some(else_block) = &if_stmt.else_block {
                                    // Keep only the else block contents
                                    for stmt in &else_block.statements {
                                        new_statements.push(stmt.clone());
                                    }
                                }
                                // If no else block, the entire if statement is eliminated
                            }
                        } else {
                            new_statements.push(statement);
                        }
                    } else {
                        // Not a constant condition, recurse into blocks
                        let mut modified_if = if_stmt.clone();
                        self.eliminate_dead_code_in_block(&mut modified_if.then_block);
                        if let Some(ref mut else_block) = modified_if.else_block {
                            self.eliminate_dead_code_in_block(else_block);
                        }
                        new_statements.push(Statement::If(modified_if));
                    }
                }
                Statement::While(while_stmt) => {
                    // Check if condition is constant false
                    if let Expression::Literal(lit) = &while_stmt.condition {
                        if let Literal::Boolean(false) = &lit.value {
                            // While false - entire loop is dead
                            self.stats.dead_code_removed += 1;
                            continue; // Skip this statement entirely
                        }
                    }
                    
                    // Otherwise recurse into the body
                    let mut modified_while = while_stmt.clone();
                    self.eliminate_dead_code_in_block(&mut modified_while.body);
                    new_statements.push(Statement::While(modified_while));
                }
                Statement::DoWhile(do_while) => {
                    // Do-while always executes at least once
                    let mut modified = do_while.clone();
                    self.eliminate_dead_code_in_block(&mut modified.body);
                    new_statements.push(Statement::DoWhile(modified));
                }
                Statement::For(for_stmt) => {
                    // Check if we can determine the loop never executes
                    // This is complex, so for now just recurse
                    let mut modified = for_stmt.clone();
                    self.eliminate_dead_code_in_block(&mut modified.body);
                    new_statements.push(Statement::For(modified));
                }
                Statement::Block(inner_block) => {
                    let mut modified = inner_block.clone();
                    self.eliminate_dead_code_in_block(&mut modified);
                    new_statements.push(Statement::Block(modified));
                }
                _ => {
                    // Keep the statement as is
                    new_statements.push(statement);
                }
            }
        }
        
        block.statements = new_statements;
    }

    fn apply_strength_reduction_to_block(&mut self, block: &mut Block) {
        for statement in &mut block.statements {
            self.apply_strength_reduction_to_statement(statement);
        }
    }

    fn apply_strength_reduction_to_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::Const(const_stmt) => {
                self.apply_strength_reduction_to_expression(&mut const_stmt.value);
            }
            Statement::Let(let_stmt) => {
                if let Some(ref mut value) = let_stmt.value {
                    self.apply_strength_reduction_to_expression(value);
                }
            }
            Statement::Display(display_stmt) => {
                for expr in &mut display_stmt.expressions {
                    self.apply_strength_reduction_to_expression(expr);
                }
            }
            Statement::If(if_stmt) => {
                self.apply_strength_reduction_to_expression(&mut if_stmt.condition);
                self.apply_strength_reduction_to_block(&mut if_stmt.then_block);
                if let Some(ref mut else_block) = if_stmt.else_block {
                    self.apply_strength_reduction_to_block(else_block);
                }
            }
            Statement::While(while_stmt) => {
                self.apply_strength_reduction_to_expression(&mut while_stmt.condition);
                self.apply_strength_reduction_to_block(&mut while_stmt.body);
            }
            Statement::DoWhile(do_while) => {
                self.apply_strength_reduction_to_block(&mut do_while.body);
                self.apply_strength_reduction_to_expression(&mut do_while.condition);
            }
            Statement::For(for_stmt) => {
                if let Some(ref mut init) = for_stmt.init {
                    self.apply_strength_reduction_to_statement(init);
                }
                if let Some(ref mut condition) = for_stmt.condition {
                    self.apply_strength_reduction_to_expression(condition);
                }
                if let Some(ref mut update) = for_stmt.update {
                    self.apply_strength_reduction_to_expression(update);
                }
                self.apply_strength_reduction_to_block(&mut for_stmt.body);
            }
            Statement::Return(return_stmt) => {
                if let Some(ref mut value) = return_stmt.value {
                    self.apply_strength_reduction_to_expression(value);
                }
            }
            Statement::Expression(expr_stmt) => {
                self.apply_strength_reduction_to_expression(&mut expr_stmt.expression);
            }
            Statement::Block(block) => {
                self.apply_strength_reduction_to_block(block);
            }
            Statement::Break(_) | Statement::Continue(_) => {
                // No expressions to optimize
            }
        }
    }

    /// Apply strength reduction to an expression
    fn apply_strength_reduction_to_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Binary(binary) => {
                // First, recursively apply to sub-expressions
                self.apply_strength_reduction_to_expression(&mut binary.left);
                self.apply_strength_reduction_to_expression(&mut binary.right);
                
                // Now check if we can apply strength reduction
                // Note: we pass mutable reference to binary to update optimization hints
                if let Some(reduced) = self.try_strength_reduce_binary(binary) {
                    *expr = reduced;
                }
                // If None returned, the optimization hint may have been added to binary
            }
            Expression::Unary(unary) => {
                self.apply_strength_reduction_to_expression(&mut unary.operand);
            }
            Expression::Call(call) => {
                for arg in &mut call.args {
                    self.apply_strength_reduction_to_expression(arg);
                }
            }
            Expression::Index(index) => {
                self.apply_strength_reduction_to_expression(&mut index.array);
                self.apply_strength_reduction_to_expression(&mut index.index);
            }
            Expression::Assign(assign) => {
                self.apply_strength_reduction_to_expression(&mut assign.value);
            }
            Expression::Literal(_) | Expression::Identifier(_) => {
                // No optimization needed for literals and identifiers
            }
        }
    }

    fn try_strength_reduce_binary(&mut self, binary: &mut BinaryExpr) -> Option<Expression> {
        // Only apply at optimization level 1 or higher
        if self.optimization_level < 1 {
            return None;
        }
        
        match binary.op {
            BinaryOp::Multiply => {
                // Check for multiplication patterns
                let (literal_value, other_side, _) = 
                    if let Expression::Literal(lit) = &*binary.left {
                        if let Literal::Integer(n) = &lit.value {
                            (Some(*n), &*binary.right, true)
                        } else {
                            (None, &*binary.right, false)
                        }
                    } else if let Expression::Literal(lit) = &*binary.right {
                        if let Literal::Integer(n) = &lit.value {
                            (Some(*n), &*binary.left, false)
                        } else {
                            (None, &*binary.left, false)
                        }
                    } else {
                        (None, &*binary.left, false)
                    };
                
                if let Some(n) = literal_value {
                    if n == 0 {
                        // x * 0 = 0
                        self.stats.strength_reductions += 1;
                        return Some(Expression::Literal(LiteralExpr {
                            value: Literal::Integer(0),
                            span: binary.span.clone(),
                        }));
                    } else if n == 1 {
                        // x * 1 = x
                        self.stats.strength_reductions += 1;
                        return Some(other_side.clone());
                    } else if n == -1 {
                        // x * -1 = -x
                        self.stats.strength_reductions += 1;
                        return Some(Expression::Unary(UnaryExpr {
                            op: UnaryOp::Negate,
                            operand: Box::new(other_side.clone()),
                            span: binary.span.clone(),
                        }));
                    } else if self.is_power_of_two(n) {
                        // x * 2^k -> mark for shift optimization
                        let shift_amount = n.trailing_zeros();
                        binary.optimization_hint = Some(OptimizationHint::ShiftLeft(shift_amount));
                        self.stats.strength_reductions += 1;
                        // Return None to keep the expression but with hint
                        return None;
                    }
                }
            }
            
            BinaryOp::Divide => {
                // Check for division patterns
                if let Expression::Literal(lit) = &*binary.right {
                    if let Literal::Integer(n) = &lit.value {
                        if *n == 1 {
                            // x / 1 = x
                            self.stats.strength_reductions += 1;
                            return Some((*binary.left).clone());
                        } else if *n == -1 {
                            // x / -1 = -x
                            self.stats.strength_reductions += 1;
                            return Some(Expression::Unary(UnaryExpr {
                                op: UnaryOp::Negate,
                                operand: Box::new((*binary.left).clone()),
                                span: binary.span.clone(),
                            }));
                        } else if *n != 0 && self.is_power_of_two(*n) {
                            // x / 2^k -> mark for shift optimization
                            let shift_amount = (*n).trailing_zeros();
                            binary.optimization_hint = Some(OptimizationHint::ShiftRight(shift_amount));
                            self.stats.strength_reductions += 1;
                            return None;
                        }
                    }
                }
            }
            
            BinaryOp::Modulo => {
                // Check for modulo patterns
                if let Expression::Literal(lit) = &*binary.right {
                    if let Literal::Integer(n) = &lit.value {
                        if self.is_power_of_two(*n) {
                            // x % 2^k -> x & (2^k - 1)
                            binary.optimization_hint = Some(OptimizationHint::BitwiseAnd(*n - 1));
                            self.stats.strength_reductions += 1;
                            return None;
                        }
                    }
                }
            }
            
            BinaryOp::Add => {
                // Check for addition patterns
                let (literal_value, other_side) = 
                    if let Expression::Literal(lit) = &*binary.left {
                        if let Literal::Integer(n) = &lit.value {
                            (Some(*n), &*binary.right)
                        } else {
                            (None, &*binary.right)
                        }
                    } else if let Expression::Literal(lit) = &*binary.right {
                        if let Literal::Integer(n) = &lit.value {
                            (Some(*n), &*binary.left)
                        } else {
                            (None, &*binary.left)
                        }
                    } else {
                        (None, &*binary.left)
                    };
                
                if let Some(0) = literal_value {
                    // x + 0 = x or 0 + x = x
                    self.stats.strength_reductions += 1;
                    return Some(other_side.clone());
                }
            }
            
            BinaryOp::Subtract => {
                // Check for subtraction patterns
                if let Expression::Literal(lit) = &*binary.right {
                    if let Literal::Integer(0) = &lit.value {
                        // x - 0 = x
                        self.stats.strength_reductions += 1;
                        return Some((*binary.left).clone());
                    }
                }
                
                // Check for x - x = 0
                if self.expressions_equal(&*binary.left, &*binary.right) {
                    self.stats.strength_reductions += 1;
                    return Some(Expression::Literal(LiteralExpr {
                        value: Literal::Integer(0),
                        span: binary.span.clone(),
                    }));
                }
            }
            
            _ => {
                // No strength reduction for other operators
            }
        }
        
        None
    }

    fn expressions_equal(&self, left: &Expression, right: &Expression) -> bool {
        match (left, right) {
            (Expression::Identifier(l), Expression::Identifier(r)) => l.name == r.name,
            (Expression::Literal(l), Expression::Literal(r)) => {
                match (&l.value, &r.value) {
                    (Literal::Integer(a), Literal::Integer(b)) => a == b,
                    (Literal::Float(a), Literal::Float(b)) => (a - b).abs() < f64::EPSILON,
                    (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
                    (Literal::String(a), Literal::String(b)) => a == b,
                    _ => false,
                }
            }
            _ => false,
        }
    }
    
}