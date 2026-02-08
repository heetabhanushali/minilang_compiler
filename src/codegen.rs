// src/codegen.rs - C code generator for MiniLang

use std::collections::HashMap;
use crate::ast::*;

/// C Code Generator
pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    _temp_counter: usize,
    array_sizes: HashMap<String, usize>,
    variable_types: HashMap<String, Type>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            _temp_counter: 0,
            array_sizes: HashMap::new(),
            variable_types: HashMap::new(),
        }
    }
    
    /// Generate C code from a MiniLang program
    pub fn generate(&mut self, program: &Program) -> Result<String, String> {
        // Add C headers
        self.emit_headers();
        
        // Add runtime support functions
        self.emit_runtime_support();
        
        // Forward declare all functions
        for function in &program.functions {
            self.emit_function_declaration(function);
        }
        self.emit_line("");
        
        // Generate function definitions
        for function in &program.functions {
            self.emit_function(function)?;
            self.emit_line("");
        }
        
        Ok(self.output.clone())
    }
    
    /// Emit C headers
    fn emit_headers(&mut self) {
        self.emit_line("#include <stdio.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("#include <string.h>");
        self.emit_line("#include <stdbool.h>");
        self.emit_line("");
        self.emit_line("// Generated from MiniLang source");
        self.emit_line("");
    }
    
    /// Emit runtime support functions
    fn emit_runtime_support(&mut self) {
        // Array bounds checking
        self.emit_line("// Runtime support");
        self.emit_line("void _minilang_check_bounds(int index, int size, const char* file, int line) {");
        self.indent_level += 1;
        self.emit_line("if (index < 0 || index >= size) {");
        self.indent_level += 1;
        self.emit_line("fprintf(stderr, \"Runtime Error: Array index %d out of bounds (size %d)\\n\", index, size);");
        self.emit_line("fprintf(stderr, \"  at %s:%d\\n\", file, line);");
        self.emit_line("exit(1);");
        self.indent_level -= 1;
        self.emit_line("}");
        self.indent_level -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("#define CHECK_BOUNDS(idx, size) _minilang_check_bounds(idx, size, __FILE__, __LINE__)");
        self.emit_line("");
    }
    
    /// Emit function forward declaration
    fn emit_function_declaration(&mut self, function: &Function) {
        let return_type = if function.name == "main" {
            "int".to_string()
        } else {
            self.c_type(&function.return_type)
        };
        let params = function.params.iter()
            .map(|p| format!("{} {}", self.c_type(&Some(p.typ.clone())), self.c_identifier(&p.name)))
            .collect::<Vec<_>>()
            .join(", ");
        
        self.emit_line(&format!("{} {}({});", return_type, self.c_identifier(&function.name), params));
    }
    
    /// Emit function definition
    fn emit_function(&mut self, function: &Function) -> Result<(), String> {
        let return_type = if function.name == "main" {
            "int".to_string()
        } else {
            self.c_type(&function.return_type)
        };
        let params = if function.params.is_empty() {
            "void".to_string()
        } else {
            function.params.iter()
                .map(|p| format!("{} {}", self.c_type(&Some(p.typ.clone())), self.c_identifier(&p.name)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        self.emit_line(&format!("{} {}({}) {{", return_type, self.c_identifier(&function.name), params));
        self.indent_level += 1;
        
        // Generate body
        self.emit_block(&function.body)?;
        
        // Add implicit return for void functions
        if function.return_type.is_none() {
            if function.name == "main" {
                self.emit_line("return 0;");
            } else {
                self.emit_line("return;");
            }
        }
        
        self.indent_level -= 1;
        self.emit_line("}");
        
        Ok(())
    }
    
    /// Emit a block
    fn emit_block(&mut self, block: &Block) -> Result<(), String> {
        for statement in &block.statements {
            self.emit_statement(statement)?;
        }
        Ok(())
    }
    
    /// Emit a statement
    fn emit_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Const(const_stmt) => self.emit_const_statement(const_stmt),
            Statement::Let(let_stmt) => self.emit_let_statement(let_stmt),
            Statement::Display(display_stmt) => self.emit_display_statement(display_stmt),
            Statement::If(if_stmt) => self.emit_if_statement(if_stmt),
            Statement::While(while_stmt) => self.emit_while_statement(while_stmt),
            Statement::DoWhile(do_while) => self.emit_do_while_statement(do_while),
            Statement::For(for_stmt) => self.emit_for_statement(for_stmt),
            Statement::Return(return_stmt) => self.emit_return_statement(return_stmt),
            Statement::Expression(expr_stmt) => self.emit_expression_statement(expr_stmt),
            Statement::Block(block) => {
                self.emit_line("{");
                self.indent_level += 1;
                self.emit_block(block)?;
                self.indent_level -= 1;
                self.emit_line("}");
                Ok(())
            }
            Statement::Break(_) => {
                self.emit_line("break;");
                Ok(())
            }

            Statement::Continue(_) => {
                self.emit_line("continue;");
                Ok(())
            }
        }
    }

    /// Emit const statement (as C const)
    fn emit_const_statement(&mut self, stmt: &ConstStmt) -> Result<(), String> {
        let c_type = self.c_type(&Some(stmt.typ.clone()));
        let name = self.c_identifier(&stmt.name);
        let value = self.expression_to_string(&stmt.value)?;
        
        self.emit_line(&format!("const {} {} = {};", c_type, name, value));
        
        // Track the type
        self.variable_types.insert(stmt.name.clone(), stmt.typ.clone());
        
        Ok(())
    }
    
    /// Emit let statement
    fn emit_let_statement(&mut self, stmt: &LetStmt) -> Result<(), String> {
        let decl = self.c_type_declaration(&stmt.typ, &self.c_identifier(&stmt.name));

        // Cache array size and type
        if let Type::Array(_elem_type, size) = &stmt.typ {
            self.array_sizes.insert(stmt.name.clone(), *size);
            self.variable_types.insert(stmt.name.clone(), stmt.typ.clone());
        } else {
            self.variable_types.insert(stmt.name.clone(), stmt.typ.clone());
        }
        
        if let Some(value) = &stmt.value {
            let value_code = self.expression_to_string(value)?;
            
            // Special handling for array initialization
            if let Type::Array(_elem_type, _size) = &stmt.typ {
                if let Expression::Literal(lit_expr) = value {
                    if let Literal::Array(elements) = &lit_expr.value {
                        // Initialize array with literal values
                        self.emit_line(&format!("{} = {{", decl));
                        self.indent_level += 1;
                        
                        let elem_strs: Vec<String> = elements.iter()
                            .map(|e| self.expression_to_string(e))
                            .collect::<Result<Vec<_>, _>>()?;
                        
                        self.emit_line(&elem_strs.join(", "));
                        
                        self.indent_level -= 1;
                        self.emit_line("};");
                        return Ok(());
                    }
                }
            }
            
            self.emit_line(&format!("{} = {};", decl, value_code));
        } else {
            // Default initialization
            match &stmt.typ {
                Type::Array(_, _size) => {
                    self.emit_line(&format!("{} = {{}};", decl));
                }
                _ => {
                    self.emit_line(&format!("{} = 0;", decl));
                }
            }
        }
        
        Ok(())
    }
    
    /// Emit display statement
    fn emit_display_statement(&mut self, stmt: &DisplayStmt) -> Result<(), String> {
        for expr in &stmt.expressions {
            // Check if this is an interpolated string literal
            if let Expression::Literal(lit_expr) = expr {
                if let Literal::InterpolatedString(parts) = &lit_expr.value {
                    // Handle interpolated string specially
                    for part in parts {
                        match part {
                            StringPart::Text(text) => {
                                self.emit_line(&format!("printf(\"%s\", \"{}\");", 
                                    self.escape_string(text)));
                            }
                            StringPart::Expression(expr) => {
                                self.emit_display_expression(expr)?;
                            }
                        }
                    }
                    continue; // Skip normal processing
                }
            }
            
            self.emit_display_expression(expr)?;
        }
        Ok(())
    }

    fn emit_display_expression(&mut self, expr: &Expression) -> Result<(), String> {
        let expr_str = self.expression_to_string(expr)?;
        if self.is_bool_expression(expr){
            self.emit_line(&format!("printf(\"%s\", {} ? \"true\" : \"false\");", expr_str));
        } else{
            let format = self.get_printf_format(expr);
            self.emit_line(&format!("printf(\"{}\", {});", format, expr_str));
        }
        Ok(())
    }

    fn is_bool_expression(&self, expr: &Expression) -> bool {
        match expr{
            Expression::Literal(lit) => matches!(lit.value, Literal::Boolean(_)),
            Expression::Identifier(id) => {
                self.variable_types.get(&id.name)
                    .map(|t| matches!(t, Type::Bool))
                    .unwrap_or(false)
            }
            Expression::Binary(binary) => {
                matches!(binary.op,
                    BinaryOp::Equal | BinaryOp::NotEqual |
                    BinaryOp::Less | BinaryOp::Greater |
                    BinaryOp::LessEqual | BinaryOp::GreaterEqual |
                    BinaryOp::And | BinaryOp::Or
                )
            }
            Expression::Unary(unary) => {
                matches!(unary.op, UnaryOp::Not)
            }
            Expression::Call(call) => {
                self.variable_types.get(&call.function)
                    .map(|t| matches!(t, Type::Bool))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }
    
    /// Emit if statement
    fn emit_if_statement(&mut self, stmt: &IfStmt) -> Result<(), String> {
        let condition = self.expression_to_string(&stmt.condition)?;
        
        self.emit_line(&format!("if ({}) {{", condition));
        self.indent_level += 1;
        self.emit_block(&stmt.then_block)?;
        self.indent_level -= 1;
        
        if let Some(else_block) = &stmt.else_block {
            self.emit_line("} else {");
            self.indent_level += 1;
            self.emit_block(else_block)?;
            self.indent_level -= 1;
        }
        
        self.emit_line("}");
        Ok(())
    }
    
    /// Emit while statement
    fn emit_while_statement(&mut self, stmt: &WhileStmt) -> Result<(), String> {
        let condition = self.expression_to_string(&stmt.condition)?;
        
        self.emit_line(&format!("while ({}) {{", condition));
        self.indent_level += 1;
        self.emit_block(&stmt.body)?;
        self.indent_level -= 1;
        self.emit_line("}");
        
        Ok(())
    }
    
    /// Emit do-while statement
    fn emit_do_while_statement(&mut self, stmt: &DoWhileStmt) -> Result<(), String> {
        self.emit_line("do {");
        self.indent_level += 1;
        self.emit_block(&stmt.body)?;
        self.indent_level -= 1;
        
        let condition = self.expression_to_string(&stmt.condition)?;
        self.emit_line(&format!("}} while ({});", condition));
        
        Ok(())
    }
    
    /// Emit for statement
    fn emit_for_statement(&mut self, stmt: &ForStmt) -> Result<(), String> {
        self.emit("for (");
        
        // Init
        if let Some(init) = &stmt.init {
            // Special handling for init - it's already a statement
            match &**init {
                Statement::Let(let_stmt) => {
                    let c_type = self.c_type(&Some(let_stmt.typ.clone()));
                    let name = self.c_identifier(&let_stmt.name);
                    if let Some(value) = &let_stmt.value {
                        let value_code = self.expression_to_string(value)?;
                        self.output.push_str(&format!("{} {} = {}", c_type, name, value_code));
                    } else {
                        self.output.push_str(&format!("{} {} = 0", c_type, name));
                    }
                }
                Statement::Expression(expr_stmt) => {
                    let expr_str = self.expression_to_string(&expr_stmt.expression)?;
                    self.output.push_str(&expr_str);
                }
                _ => {}
            }
        }
        self.output.push_str("; ");
        
        // Condition
        if let Some(condition) = &stmt.condition {
            let cond_str = self.expression_to_string(condition)?;
            self.output.push_str(&cond_str);
        }
        self.output.push_str("; ");
        
        // Update
        if let Some(update) = &stmt.update {
            let update_str = self.expression_to_string(update)?;
            self.output.push_str(&update_str);
        }
        
        self.output.push_str(") {\n");
        
        self.indent_level += 1;
        self.emit_block(&stmt.body)?;
        self.indent_level -= 1;
        self.emit_line("}");
        
        Ok(())
    }
    
    /// Emit return statement
    fn emit_return_statement(&mut self, stmt: &ReturnStmt) -> Result<(), String> {
        if let Some(value) = &stmt.value {
            let value_str = self.expression_to_string(value)?;
            self.emit_line(&format!("return {};", value_str));
        } else {
            self.emit_line("return;");
        }
        Ok(())
    }
    
    /// Emit expression statement
    fn emit_expression_statement(&mut self, stmt: &ExprStmt) -> Result<(), String> {
        // Skip standalone identifiers (they're leftovers from array assignment placeholders)
        if let Expression::Identifier(_) = &stmt.expression {
            return Ok(());
        }
        
        let expr_str = self.expression_to_string(&stmt.expression)?;
        self.emit_line(&format!("{};", expr_str));
        Ok(())
    }
    
    /// Convert expression to C code string
    fn expression_to_string(&mut self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Literal(lit_expr) => self.literal_to_string(&lit_expr.value),
            
            Expression::Identifier(id_expr) => {
                Ok(self.c_identifier(&id_expr.name))
            }
            
            Expression::Binary(binary) => {
                if let Some(ref hint) = binary.optimization_hint {
                    let left = self.expression_to_string(&binary.left)?;
                    match hint {
                        OptimizationHint::ShiftLeft(n)=>{
                            Ok(format!("({} << {})", left , n))
                        }
                        OptimizationHint::ShiftRight(n)=>{
                            Ok(format!("({} >> {})", left , n))
                        }
                        OptimizationHint::BitwiseAnd(mask)=>{
                            Ok(format!("({} & {})", left , mask))
                        }
                    }
                } else{
                    let left = self.expression_to_string(&binary.left)?;
                    let right = self.expression_to_string(&binary.right)?;
                    let op = self.binary_op_to_string(&binary.op);
                    Ok(format!("({} {} {})", left, op, right))
                }
            }
            
            Expression::Unary(unary) => {
                let operand = self.expression_to_string(&unary.operand)?;
                let op = match unary.op {
                    UnaryOp::Not => "!",
                    UnaryOp::Negate => "-",
                };
                Ok(format!("({}{})", op, operand))
            }
            
            Expression::Call(call) => {
                let args: Vec<String> = call.args.iter()
                    .map(|arg| self.expression_to_string(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                
                Ok(format!("{}({})", self.c_identifier(&call.function), args.join(", ")))
            }
            
            Expression::Index(index) => {
                let array = self.expression_to_string(&index.array)?;
                let idx = self.expression_to_string(&index.index)?;

                // Get actual array size
                let array_size = if let Expression::Identifier(id) = &*index.array {
                    self.array_sizes.get(&id.name).copied().unwrap_or(10)
                } else {
                    10  // fallback
                };
                
                Ok(format!("({}, {})", 
                    format!("CHECK_BOUNDS({}, {})", idx, array_size), 
                    format!("{}[{}]", array, idx)
                ))
            }
            
            Expression::Assign(assign) => {
                // Check if this is an array index assignment (special marker)
                if assign.target.starts_with("__ARRAY_INDEX__:") {
                    // Extract the array index assignment from the binary expression
                    if let Expression::Binary(binary) = &*assign.value {
                        if let Expression::Index(index_expr) = &*binary.left {
                            // Generate: arr[index] = value
                            let array = self.expression_to_string(&index_expr.array)?;
                            let index = self.expression_to_string(&index_expr.index)?;
                            let value = self.expression_to_string(&binary.right)?;
                            
                            // Return without bounds checking wrapper in the assignment target
                            return Ok(format!("({}[{}] = {})", array, index, value));
                        }
                    }
                }
                
                // Regular assignment
                let value = self.expression_to_string(&assign.value)?;
                Ok(format!("({} = {})", self.c_identifier(&assign.target), value))
            }
        }
    }
    
    /// Convert literal to C string
    fn literal_to_string(&mut self, lit: &Literal) -> Result<String, String> {
        match lit {
            Literal::Integer(n) => Ok(n.to_string()),
            Literal::Float(f) => Ok(format!("{:.6}", f)),
            Literal::String(s) => Ok(format!("\"{}\"", self.escape_string(s))),
            Literal::Boolean(b) => Ok(if *b { "true".to_string() } else { "false".to_string() }),
            Literal::InterpolatedString(_parts) => {
                // This shouldn't be called directly; handled in emit_display_statement
                Err("Interpolated strings should be handled in display statement".to_string())
            }
            Literal::Array(elements) => {
                // This shouldn't be called for array literals in declarations
                // but we'll handle it anyway
                let elem_strs: Vec<String> = elements.iter()
                    .map(|e| self.expression_to_string(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", elem_strs.join(", ")))
            }
        }
    }
    
    /// Convert binary operator to C string
    fn binary_op_to_string(&self, op: &BinaryOp) -> &str {
        match op {
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
        }
    }
    
    /// Convert MiniLang type to C type
    fn c_type(&self, typ: &Option<Type>) -> String {
        match typ {
            Some(Type::Int) => "int".to_string(),
            Some(Type::Float) => "double".to_string(),
            Some(Type::String) => "const char*".to_string(),
            Some(Type::Bool) => "bool".to_string(),
            Some(Type::Array(elem_type, _size)) => {
                // For function parameters, arrays become pointers
                format!("{}*", self.c_type(&Some(*elem_type.clone())))
            }
            None => "void".to_string(),
        }
    }

    /// Convert MiniLang type to C type for variable declarations
    fn c_type_declaration(&self, typ: &Type, name: &str) -> String {
        match typ {
            Type::Array(elem_type, size) => {
                format!("{} {}[{}]", self.c_type(&Some(*elem_type.clone())), name, size)
            }
            _ => {
                format!("{} {}", self.c_type(&Some(typ.clone())), name)
            }
        }
    }
    
    /// Convert identifier to valid C identifier
    fn c_identifier(&self, name: &str) -> String {
        // Prefix with underscore if it's a C keyword
        let c_keywords = ["auto", "break", "case", "char", "const", "continue", 
                          "default", "do", "double", "else", "enum", "extern",
                          "float", "for", "goto", "if", "int", "long", 
                          "register", "return", "short", "signed", "sizeof", 
                          "static", "struct", "switch", "typedef", "union",
                          "unsigned", "void", "volatile", "while"];
        
        if c_keywords.contains(&name) {
            format!("_{}", name)
        } else {
            name.to_string()
        }
    }
    
    /// Escape string for C
    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('"', "\\\"")
         .replace('\n', "\\n")
         .replace('\r', "\\r")
         .replace('\t', "\\t")
    }
    
    /// Get printf format for expression type
    fn get_printf_format(&self, expr: &Expression) -> &str {
        match expr {
            Expression::Literal(lit_expr) => match &lit_expr.value {
                Literal::Integer(_) => "%d",
                Literal::Float(_) => "%.6f",
                Literal::String(_) => "%s",
                Literal::Boolean(_) => "%d",  // 1 or 0
                _ => "%d",
            },
            Expression::Identifier(id) => {
                // Look up the variable's type
                if let Some(typ) = self.variable_types.get(&id.name) {
                    match typ {
                        Type::Int => "%d",
                        Type::Float => "%.6f",
                        Type::String => "%s",
                        Type::Bool => "%d",
                        Type::Array(_, _) => "%p", // pointer for array
                    }
                } else {
                    "%d"  // fallback
                }
            },
            Expression::Binary(binary) => {
                // Check if it's a comparison/logical operator
                match binary.op {
                    BinaryOp::Equal | BinaryOp::NotEqual | 
                    BinaryOp::Less | BinaryOp::Greater |
                    BinaryOp::LessEqual | BinaryOp::GreaterEqual |
                    BinaryOp::And | BinaryOp::Or => "%d",  // boolean result
                    _ => {
                        // For arithmetic, check if any operand is float
                        if self.is_float_expr(&binary.left) || self.is_float_expr(&binary.right) {
                            "%.6f"
                        } else {
                            "%d"
                        }
                    }
                }
            },
            Expression::Unary(unary) => {
                match unary.op {
                    UnaryOp::Not => "%d",  // boolean
                    UnaryOp::Negate => self.get_printf_format(&unary.operand),
                }
            },
            _ => "%d",
        }
    }

    fn is_float_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Literal(lit) => matches!(lit.value, Literal::Float(_)),
            Expression::Identifier(id) => {
                self.variable_types.get(&id.name)
                    .map(|t| matches!(t, Type::Float))
                    .unwrap_or(false)
            },
            _ => false,
        }
    }
    
    
    /// Emit a line with proper indentation
    fn emit_line(&mut self, line: &str) {
        self.emit(line);
        self.output.push('\n');
    }
    
    /// Emit text with proper indentation
    fn emit(&mut self, text: &str) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
        self.output.push_str(text);
    }
}