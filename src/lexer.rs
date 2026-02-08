// src/lexer.rs - Production-ready tokenizer with all edge cases handled

use logos::Logos;
use crate::errors::LexerError;

/// All possible tokens in MiniLang
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]  // Skip whitespace
#[logos(skip r"#[^\n]*")]      // Skip single-line comments
pub enum Token {
    // ===== KEYWORDS =====
    #[token("let")]
    Let,

    #[token("const")]
    Const,
    
    #[token("func")]
    Func,
    
    #[token("display")]
    Display,
    
    #[token("send")]
    Send,
    
    #[token("if")]
    If,
    
    #[token("else")]
    Else,
    
    #[token("while")]
    While,
    
    #[token("do")]
    Do,
    
    #[token("for")]
    For,
    
    #[token("true")]
    True,
    
    #[token("false")]
    False,
    
    #[token("AND")]
    And,
    
    #[token("OR")]
    Or,
    
    #[token("NOT")]
    Not,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,
    
    // ===== TYPE KEYWORDS =====
    #[token("int")]
    TypeInt,
    
    #[token("float")]
    TypeFloat,
    
    #[token("string")]
    TypeString,
    
    #[token("bool")]
    TypeBool,

    
    // ===== LITERALS =====
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i32>().ok())]
    Integer(i32),
    
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let content = &s[1..s.len()-1];
        Some(content.replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\"))
    })]
    String(String),
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 2)]
    Identifier(String),
    
    // ===== OPERATORS =====
    #[token("+")]
    Plus,
    
    #[token("-")]
    Minus,
    
    #[token("*")]
    Star,
    
    #[token("/")]
    Slash,
    
    #[token("%")]
    Percent,
    
    #[token("=")]
    Assign,
    
    #[token("==")]
    Equal,
    
    #[token("!=")]
    NotEqual,
    
    #[token("<")]
    LessThan,
    
    #[token(">")]
    GreaterThan,
    
    #[token("<=")]
    LessEqual,
    
    #[token(">=")]
    GreaterEqual,
    
    // ===== PUNCTUATION =====
    #[token("(")]
    LeftParen,
    
    #[token(")")]
    RightParen,
    
    #[token("{")]
    LeftBrace,
    
    #[token("}")]
    RightBrace,
    
    #[token("[")]
    LeftBracket,
    
    #[token("]")]
    RightBracket,
    
    #[token(",")]
    Comma,
    
    #[token(":")]
    Colon,
    
    #[token(";")]
    Semicolon,
    
    #[token("->")]
    Arrow,
}

/// Token with its location in source
#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}

/// The lexer structure
pub struct Lexer {
    source: String,
    processed_source: String,
}

impl Lexer {
    /// Create a new lexer
    pub fn new(source: &str) -> Self {
        let processed = Self::preprocess(source);
        Self {
            source: source.to_string(),
            processed_source: processed,
        }
    }
    
    /// Preprocess source to handle multi-line comments
    fn preprocess(source: &str) -> String {
        let mut result = String::new();
        let mut chars = source.char_indices().peekable();
        
        while let Some((_, ch)) = chars.next() {
            if ch == '#' {
                // Check for multi-line comment
                if let Some((_, next_ch)) = chars.peek() {
                    if *next_ch == '#' {
                        // Multi-line comment start
                        chars.next(); // consume second #
                        result.push(' ');
                        result.push(' ');
                        
                        // Find closing ##
                        let mut prev_was_hash = false;
                        while let Some((_, ch)) = chars.next() {
                            if prev_was_hash && ch == '#' {
                                result.push(' ');
                                result.push(' ');
                                break;
                            }
                            
                            prev_was_hash = ch == '#';
                            
                            // Preserve newlines for line counting
                            if ch == '\n' {
                                result.push('\n');
                            } else {
                                result.push(' ');
                            }
                        }
                    } else {
                        // Single-line comment - Logos will handle
                        result.push(ch);
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    /// Check if a string is properly terminated
    /// Returns true if all quotes are properly matched
    fn has_unterminated_string(source: &str) -> Option<usize> {
        let mut in_string = false;
        let mut last_quote_pos = 0;
        let mut escape_next = false;
        
        for (i, ch) in source.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    if in_string {
                        in_string = false;
                    } else {
                        in_string = true;
                        last_quote_pos = i;
                    }
                }
                _ => {}
            }
        }
        
        if in_string {
            Some(last_quote_pos)
        } else {
            None
        }
    }
    
    /// Tokenize the source code - finds errors in order they appear
    pub fn tokenize(&mut self) -> Result<Vec<TokenWithSpan>, LexerError> {
        // EDGE CASE 1: Check for unterminated strings FIRST
        // This catches cases like: let x = "hello (missing closing quote)
        if let Some(pos) = Self::has_unterminated_string(&self.processed_source) {
            return Err(LexerError::UnterminatedString {
                start: miette::SourceSpan::from(pos..pos + 1),
            });
        }
        
        let mut tokens = Vec::new();
        let mut lexer = Token::lexer(&self.processed_source);
        
        while let Some(result) = lexer.next() {
            let span = lexer.span();
            
            match result {
                Ok(token) => {

                    if matches!(token, Token::Integer(_)) {
                        let text = &self.processed_source[span.clone()];
                        if text.parse::<i32>().is_err(){
                            let original_pos = self.find_original_position(span.start);
                            return Err(LexerError::IntegerOverflow {
                                value: text.to_string(),
                                span: miette::SourceSpan::from(original_pos..original_pos + text.len()),
                            });
                        }
                    }

                    tokens.push(TokenWithSpan {
                        token,
                        span: span.clone(),
                    });
                }
                Err(_) => {
                    // Logos couldn't tokenize this
                    let text = &self.processed_source[span.clone()];
                    
                    // EDGE CASE 2: Skip whitespace errors
                    // Sometimes preprocessing creates whitespace that Logos flags
                    if text.trim().is_empty() {
                        continue;
                    }
                    
                    // EDGE CASE 3: Check if this is a string-related error
                    // Logos might partially match a string
                    if text.starts_with('"') {
                        let original_pos = self.find_original_position(span.start);
                        return Err(LexerError::UnterminatedString {
                            start: miette::SourceSpan::from(original_pos..original_pos + 1),
                        });
                    }
                    
                    // EDGE CASE 4: Find the actual invalid character
                    // Don't report whitespace as invalid
                    if let Some(ch) = text.chars().find(|c| !c.is_whitespace()) {
                        let original_pos = self.find_original_position(span.start);
                        
                        // EDGE CASE 5: Calculate exact position of the character
                        // Find where the non-whitespace char actually is
                        let char_offset = text.chars()
                            .take_while(|c| c.is_whitespace())
                            .count();
                        
                        return Err(LexerError::UnexpectedChar {
                            char: ch,
                            span: miette::SourceSpan::from(
                                original_pos + char_offset..original_pos + char_offset + 1
                            ),
                        });
                    }
                }
            }
        }
        
        Ok(tokens)
    }
    
    /// Find position in original source (before preprocessing)
    /// In a full production compiler, we'd maintain a position map
    /// For now, positions are roughly the same
    fn find_original_position(&self, processed_pos: usize) -> usize {
        // EDGE CASE 6: Handle position mapping
        // Since preprocessing only replaces multi-line comments with spaces,
        // positions are mostly preserved
        processed_pos.min(self.source.len().saturating_sub(1))
    }
    
    /// Get the original source for error reporting
    pub fn source(&self) -> &str {
        &self.source
    }
}