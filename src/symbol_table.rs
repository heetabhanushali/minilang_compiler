// src/symbol_table.rs - Symbol table for tracking identifiers

use std::collections::HashMap;
use crate::ast::Type;

/// Symbol information stored in the table
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub data_type: Type,
    pub scope_level: usize,
    pub defined_at: usize,  // Source location
}

/// Type of symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Constant,
    Function,
    Parameter,
}

/// Function signature information
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Option<Type>,
}

/// Symbol table with scope management
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    functions: HashMap<String, FunctionSignature>,
    current_scope: usize,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
            functions: HashMap::new(),
            current_scope: 0,
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope += 1;
    }
    
    /// Exit current scope
    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            self.scopes.pop();
            self.current_scope -= 1;
        }
    }
    
    /// Add a symbol to current scope
    pub fn insert(&mut self, symbol: Symbol) -> Result<(), String> {
        let name = symbol.name.clone();
        
        // Check if already exists in current scope
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return Err(format!("Symbol '{}' already defined in this scope", name));
            }
            scope.insert(name, symbol);
            Ok(())
        } else {
            Err("No active scope".to_string())
        }
    }
    
    /// Look up a symbol (searches all scopes from current to global)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Search from current scope to global
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }
    
    /// Check if symbol exists in current scope only
    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.contains_key(name))
            .unwrap_or(false)
    }
    
    /// Register a function
    pub fn register_function(&mut self, sig: FunctionSignature) -> Result<(), String> {
        if self.functions.contains_key(&sig.name) {
            return Err(format!("Function '{}' already defined", sig.name));
        }
        self.functions.insert(sig.name.clone(), sig);
        Ok(())
    }
    
    /// Look up a function
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }
    
    /// Get current scope level
    pub fn current_scope_level(&self) -> usize {
        self.current_scope
    }

    /// Find similar names using Levenshtein distance
    pub fn find_similar_names(&self, target: &str, max_suggestions: usize) -> Vec<String> {
        let mut candidates = Vec::new();
        
        // Check all scopes from current to global
        for scope in self.scopes.iter().rev() {
            for name in scope.keys() {
                let distance = self.levenshtein_distance(target, name);
                if distance <= 2 && distance > 0 {  // Max edit distance of 2
                    candidates.push((name.clone(), distance));
                }
            }
        }
        
        // Sort by distance and take top suggestions
        candidates.sort_by_key(|&(_, dist)| dist);
        candidates.into_iter()
            .take(max_suggestions)
            .map(|(name, _)| name)
            .collect()
    }

    /// Find similar function names
    pub fn find_similar_functions(&self, target: &str, max_suggestions: usize) -> Vec<String> {
        let mut candidates = Vec::new();
        
        for name in self.functions.keys() {
            let distance = self.levenshtein_distance(target, name);
            if distance <= 2 && distance > 0 {
                candidates.push((name.clone(), distance));
            }
        }
        
        candidates.sort_by_key(|&(_, dist)| dist);
        candidates.into_iter()
            .take(max_suggestions)
            .map(|(name, _)| name)
            .collect()
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let len1 = s1_chars.len();
        let len2 = s2_chars.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i -1] == s2_chars[j-1] {
                    0
                } else {
                    1
                };
                
                matrix[i][j] = std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    std::cmp::min(
                        matrix[i][j - 1] + 1,   // insertion
                        matrix[i - 1][j - 1] + cost  // substitution
                    )
                );
            }
        }
        
        matrix[len1][len2]
    }

    /// Get all symbols in current scope (for unused variable checking)
    pub fn current_scope_symbols(&self) -> Vec<(String, Symbol)> {
        if let Some(scope) = self.scopes.last() {
            scope.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            Vec::new()
        }
    }

}