//! Parse `.sfl` text format (Sailfish Language).
//!
//! The .sfl format is a human-readable text representation of Sailfish projects.
//! This module provides a tokenizer (lexer) and recursive descent parser that
//! converts .sfl source code into an AST (Abstract Syntax Tree).

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{ParseError, Result};

// ---------------------------------------------------------------------------
// Tokens
// ---------------------------------------------------------------------------

/// A lexical token produced by the .sfl tokenizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SflToken {
    /// An identifier (variable name, function name, etc.).
    Identifier(String),

    /// A numeric literal (integer or float).
    Number(f64),

    /// A string literal (double-quoted).
    String(String),

    /// An operator (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||).
    Operator(String),

    /// A keyword (let, fn, if, else, while, repeat, true, false, and, or, not).
    Keyword(String),

    /// Punctuation (parentheses, braces, brackets, comma, semicolon, colon, dot, arrow).
    Punctuation(String),

    /// End of file / input.
    Eof,
}

impl fmt::Display for SflToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SflToken::Identifier(s) => write!(f, "Identifier({s})"),
            SflToken::Number(n) => write!(f, "Number({n})"),
            SflToken::String(s) => write!(f, "String(\"{s}\")"),
            SflToken::Operator(s) => write!(f, "Operator({s})"),
            SflToken::Keyword(s) => write!(f, "Keyword({s})"),
            SflToken::Punctuation(s) => write!(f, "Punctuation({s})"),
            SflToken::Eof => write!(f, "Eof"),
        }
    }
}

/// The set of reserved keywords in the .sfl language.
const KEYWORDS: &[&str] = &[
    "let", "fn", "if", "else", "while", "repeat", "true", "false", "and", "or", "not", "return",
    "for", "in", "break", "continue", "null",
];

/// The set of recognized operators (checked longest-first for correct matching).
const OPERATORS: &[&str] = &[
    "==", "!=", "<=", ">=", "&&", "||", "->", "+", "-", "*", "/", "%", "<", ">", "!",
];

/// The set of punctuation characters.
const PUNCTUATION_CHARS: &[char] = &[
    '(', ')', '{', '}', '[', ']', ',', ';', ':', '.', '=',
];

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

/// Tokenize a .sfl source string into a sequence of tokens.
pub fn tokenize(source: &str) -> Vec<SflToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        let ch = chars[pos];

        // Skip whitespace
        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        // Skip single-line comments (//)
        if ch == '/' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
            pos += 2;
            while pos < chars.len() && chars[pos] != '\n' {
                pos += 1;
            }
            continue;
        }

        // Skip multi-line comments (/* ... */)
        if ch == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
            pos += 2;
            while pos + 1 < chars.len() {
                if chars[pos] == '*' && chars[pos + 1] == '/' {
                    pos += 2;
                    break;
                }
                pos += 1;
            }
            continue;
        }

        // Numbers (integer and float)
        if ch.is_ascii_digit() {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                pos += 1;
            }
            let num_str: String = chars[start..pos].iter().collect();
            if let Ok(n) = num_str.parse::<f64>() {
                tokens.push(SflToken::Number(n));
            } else {
                tokens.push(SflToken::Number(0.0));
            }
            continue;
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }
            let ident: String = chars[start..pos].iter().collect();
            if KEYWORDS.contains(&ident.as_str()) {
                tokens.push(SflToken::Keyword(ident));
            } else {
                tokens.push(SflToken::Identifier(ident));
            }
            continue;
        }

        // String literals (double-quoted)
        if ch == '"' {
            pos += 1; // skip opening quote
            let mut string_content = String::new();
            while pos < chars.len() && chars[pos] != '"' {
                if chars[pos] == '\\' && pos + 1 < chars.len() {
                    pos += 1;
                    match chars[pos] {
                        'n' => string_content.push('\n'),
                        't' => string_content.push('\t'),
                        'r' => string_content.push('\r'),
                        '\\' => string_content.push('\\'),
                        '"' => string_content.push('"'),
                        other => string_content.push(other),
                    }
                } else {
                    string_content.push(chars[pos]);
                }
                pos += 1;
            }
            if pos < chars.len() {
                pos += 1; // skip closing quote
            }
            tokens.push(SflToken::String(string_content));
            continue;
        }

        // Multi-character operators (check longest first)
        let remaining: String = chars[pos..].iter().collect();
        let mut matched_operator = false;
        for op in OPERATORS {
            if remaining.starts_with(op) {
                // Make sure we don't confuse = with == (check == first)
                tokens.push(SflToken::Operator(op.to_string()));
                pos += op.len();
                matched_operator = true;
                break;
            }
        }
        if matched_operator {
            continue;
        }

        // Punctuation
        if PUNCTUATION_CHARS.contains(&ch) {
            // Distinguish '=' as operator (assignment) vs punctuation
            if ch == '=' {
                // If we reach here, '==' was not matched above, so it's a single '='
                tokens.push(SflToken::Operator("=".to_string()));
            } else {
                tokens.push(SflToken::Punctuation(ch.to_string()));
            }
            pos += 1;
            continue;
        }

        // Unknown character – skip it
        pos += 1;
    }

    tokens.push(SflToken::Eof);
    tokens
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

/// The root AST node for a parsed .sfl program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SflProgram {
    /// Top-level statements in the program.
    pub statements: Vec<SflStatement>,
}

/// A statement in the .sfl language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SflStatement {
    /// Variable declaration: `let name = expr;`
    VarDecl {
        name: String,
        value: SflExpr,
    },

    /// Variable assignment: `name = expr;`
    Assignment {
        name: String,
        value: SflExpr,
    },

    /// Function declaration: `fn name(params) { body }`
    FuncDecl {
        name: String,
        params: Vec<String>,
        body: Vec<SflStatement>,
    },

    /// If statement: `if condition { body } else { else_body }`
    IfStmt {
        condition: SflExpr,
        then_body: Vec<SflStatement>,
        else_body: Option<Vec<SflStatement>>,
    },

    /// While loop: `while condition { body }`
    WhileLoop {
        condition: SflExpr,
        body: Vec<SflStatement>,
    },

    /// Repeat loop: `repeat count { body }`
    RepeatLoop {
        count: SflExpr,
        body: Vec<SflStatement>,
    },

    /// For-in loop: `for name in iterable { body }`
    ForLoop {
        name: String,
        iterable: SflExpr,
        body: Vec<SflStatement>,
    },

    /// Expression statement (function call, etc.)
    ExprStmt(SflExpr),

    /// Return statement: `return expr;`
    Return(Option<SflExpr>),

    /// Break statement.
    Break,

    /// Continue statement.
    Continue,
}

/// An expression in the .sfl language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SflExpr {
    /// A numeric literal.
    Number(f64),

    /// A string literal.
    String(String),

    /// A boolean literal.
    Bool(bool),

    /// The null literal.
    Null,

    /// A variable reference.
    Identifier(String),

    /// A binary operation: left op right.
    BinaryOp {
        left: Box<SflExpr>,
        op: String,
        right: Box<SflExpr>,
    },

    /// A unary operation: op expr.
    UnaryOp {
        op: String,
        expr: Box<SflExpr>,
    },

    /// A function call: name(args).
    FuncCall {
        name: String,
        args: Vec<SflExpr>,
    },

    /// A list literal: [expr, expr, ...].
    List(Vec<SflExpr>),

    /// A grouped / parenthesized expression: (expr).
    Grouped(Box<SflExpr>),

    /// An index access: expr[expr].
    IndexAccess {
        object: Box<SflExpr>,
        index: Box<SflExpr>,
    },

    /// A member access: expr.name.
    MemberAccess {
        object: Box<SflExpr>,
        member: String,
    },
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// A recursive descent parser for .sfl token streams.
pub struct Parser {
    tokens: Vec<SflToken>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<SflToken>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse the entire token stream into an AST.
    pub fn parse_program(&mut self) -> Result<SflProgram> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if self.peek() == &SflToken::Eof {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(SflProgram { statements })
    }

    // -- Helpers --

    fn peek(&self) -> &SflToken {
        self.tokens.get(self.pos).unwrap_or(&SflToken::Eof)
    }

    fn advance(&mut self) -> SflToken {
        let token = self.tokens.get(self.pos).cloned().unwrap_or(SflToken::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.peek() == &SflToken::Eof
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<()> {
        match self.advance() {
            SflToken::Keyword(k) if k == keyword => Ok(()),
            other => Err(ParseError::UnexpectedToken {
                expected: format!("keyword '{keyword}'"),
                found: format!("{other}"),
            }),
        }
    }

    fn expect_punctuation(&mut self, punct: &str) -> Result<()> {
        match self.advance() {
            SflToken::Punctuation(p) if p == punct => Ok(()),
            other => Err(ParseError::UnexpectedToken {
                expected: format!("'{punct}'"),
                found: format!("{other}"),
            }),
        }
    }

    fn expect_operator(&mut self, op: &str) -> Result<()> {
        match self.advance() {
            SflToken::Operator(o) if o == op => Ok(()),
            other => Err(ParseError::UnexpectedToken {
                expected: format!("'{op}'"),
                found: format!("{other}"),
            }),
        }
    }

    fn expect_identifier(&mut self) -> Result<String> {
        match self.advance() {
            SflToken::Identifier(name) => Ok(name),
            other => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{other}"),
            }),
        }
    }

    // -- Statements --

    fn parse_statement(&mut self) -> Result<SflStatement> {
        match self.peek().clone() {
            SflToken::Keyword(k) => match k.as_str() {
                "let" => self.parse_var_decl(),
                "fn" => self.parse_func_decl(),
                "if" => self.parse_if_stmt(),
                "while" => self.parse_while_loop(),
                "repeat" => self.parse_repeat_loop(),
                "for" => self.parse_for_loop(),
                "return" => self.parse_return_stmt(),
                "break" => {
                    self.advance();
                    self.consume_optional_semicolon();
                    Ok(SflStatement::Break)
                }
                "continue" => {
                    self.advance();
                    self.consume_optional_semicolon();
                    Ok(SflStatement::Continue)
                }
                _ => self.parse_expr_stmt(),
            },
            SflToken::Identifier(_) => {
                // Could be assignment or expression statement
                // Peek ahead to see if next token is '='
                if self.pos + 1 < self.tokens.len() {
                    if let SflToken::Operator(op) = &self.tokens[self.pos + 1] {
                        if op == "=" {
                            return self.parse_assignment();
                        }
                    }
                }
                self.parse_expr_stmt()
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_var_decl(&mut self) -> Result<SflStatement> {
        self.expect_keyword("let")?;
        let name = self.expect_identifier()?;
        self.expect_operator("=")?;
        let value = self.parse_expression()?;
        self.consume_optional_semicolon();
        Ok(SflStatement::VarDecl { name, value })
    }

    fn parse_assignment(&mut self) -> Result<SflStatement> {
        let name = self.expect_identifier()?;
        self.expect_operator("=")?;
        let value = self.parse_expression()?;
        self.consume_optional_semicolon();
        Ok(SflStatement::Assignment { name, value })
    }

    fn parse_func_decl(&mut self) -> Result<SflStatement> {
        self.expect_keyword("fn")?;
        let name = self.expect_identifier()?;
        self.expect_punctuation("(")?;
        let mut params = Vec::new();
        while self.peek() != &SflToken::Punctuation(")".to_string()) && !self.is_at_end() {
            params.push(self.expect_identifier()?);
            if self.peek() == &SflToken::Punctuation(",".to_string()) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_punctuation(")")?;
        let body = self.parse_block()?;
        Ok(SflStatement::FuncDecl { name, params, body })
    }

    fn parse_if_stmt(&mut self) -> Result<SflStatement> {
        self.expect_keyword("if")?;
        let condition = self.parse_expression()?;
        let then_body = self.parse_block()?;
        let else_body = if self.peek() == &SflToken::Keyword("else".to_string()) {
            self.advance();
            if self.peek() == &SflToken::Keyword("if".to_string()) {
                // else if
                let stmt = self.parse_if_stmt()?;
                Some(vec![stmt])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(SflStatement::IfStmt {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_while_loop(&mut self) -> Result<SflStatement> {
        self.expect_keyword("while")?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(SflStatement::WhileLoop { condition, body })
    }

    fn parse_repeat_loop(&mut self) -> Result<SflStatement> {
        self.expect_keyword("repeat")?;
        let count = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(SflStatement::RepeatLoop { count, body })
    }

    fn parse_for_loop(&mut self) -> Result<SflStatement> {
        self.expect_keyword("for")?;
        let name = self.expect_identifier()?;
        self.expect_keyword("in")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(SflStatement::ForLoop {
            name,
            iterable,
            body,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<SflStatement> {
        self.expect_keyword("return")?;
        // Check if there's a value to return
        let is_end = matches!(
            self.peek(),
            SflToken::Punctuation(p) if p == ";" || p == "}"
        ) || self.is_at_end();
        let value = if is_end {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume_optional_semicolon();
        Ok(SflStatement::Return(value))
    }

    fn parse_block(&mut self) -> Result<Vec<SflStatement>> {
        self.expect_punctuation("{")?;
        let mut stmts = Vec::new();
        while self.peek() != &SflToken::Punctuation("}".to_string()) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.expect_punctuation("}")?;
        Ok(stmts)
    }

    fn parse_expr_stmt(&mut self) -> Result<SflStatement> {
        let expr = self.parse_expression()?;
        self.consume_optional_semicolon();
        Ok(SflStatement::ExprStmt(expr))
    }

    fn consume_optional_semicolon(&mut self) {
        if self.peek() == &SflToken::Punctuation(";".to_string()) {
            self.advance();
        }
    }

    // -- Expressions (precedence climbing) --

    fn parse_expression(&mut self) -> Result<SflExpr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<SflExpr> {
        let mut left = self.parse_and()?;
        while let SflToken::Operator(op) = self.peek().clone() {
            if op == "||" || op == "or" {
                self.advance();
                let right = self.parse_and()?;
                left = SflExpr::BinaryOp {
                    left: Box::new(left),
                    op: "||".to_string(),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<SflExpr> {
        let mut left = self.parse_equality()?;
        while let SflToken::Operator(op) = self.peek().clone() {
            if op == "&&" || op == "and" {
                self.advance();
                let right = self.parse_equality()?;
                left = SflExpr::BinaryOp {
                    left: Box::new(left),
                    op: "&&".to_string(),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<SflExpr> {
        let mut left = self.parse_comparison()?;
        while let SflToken::Operator(op) = self.peek().clone() {
            if op == "==" || op == "!=" {
                self.advance();
                let right = self.parse_comparison()?;
                left = SflExpr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<SflExpr> {
        let mut left = self.parse_addition()?;
        while let SflToken::Operator(op) = self.peek().clone() {
            if op == "<" || op == ">" || op == "<=" || op == ">=" {
                self.advance();
                let right = self.parse_addition()?;
                left = SflExpr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<SflExpr> {
        let mut left = self.parse_multiplication()?;
        while let SflToken::Operator(op) = self.peek().clone() {
            if op == "+" || op == "-" {
                self.advance();
                let right = self.parse_multiplication()?;
                left = SflExpr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<SflExpr> {
        let mut left = self.parse_unary()?;
        while let SflToken::Operator(op) = self.peek().clone() {
            if op == "*" || op == "/" || op == "%" {
                self.advance();
                let right = self.parse_unary()?;
                left = SflExpr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<SflExpr> {
        if let SflToken::Operator(op) = self.peek().clone() {
            if op == "-" || op == "!" || op == "not" {
                self.advance();
                let expr = self.parse_unary()?;
                let actual_op = if op == "not" { "!" } else { &op };
                return Ok(SflExpr::UnaryOp {
                    op: actual_op.to_string(),
                    expr: Box::new(expr),
                });
            }
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<SflExpr> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                SflToken::Punctuation(p) if p == "." => {
                    self.advance();
                    let member = self.expect_identifier()?;
                    expr = SflExpr::MemberAccess {
                        object: Box::new(expr),
                        member,
                    };
                }
                SflToken::Punctuation(p) if p == "[" => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect_punctuation("]")?;
                    expr = SflExpr::IndexAccess {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<SflExpr> {
        match self.peek().clone() {
            SflToken::Number(n) => {
                self.advance();
                Ok(SflExpr::Number(n))
            }
            SflToken::String(s) => {
                self.advance();
                Ok(SflExpr::String(s))
            }
            SflToken::Keyword(k) => match k.as_str() {
                "true" => {
                    self.advance();
                    Ok(SflExpr::Bool(true))
                }
                "false" => {
                    self.advance();
                    Ok(SflExpr::Bool(false))
                }
                "null" => {
                    self.advance();
                    Ok(SflExpr::Null)
                }
                _ => Err(ParseError::UnexpectedToken {
                    expected: "expression".to_string(),
                    found: format!("Keyword({k})"),
                }),
            },
            SflToken::Identifier(name) => {
                self.advance();
                // Check for function call
                if self.peek() == &SflToken::Punctuation("(".to_string()) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    while self.peek() != &SflToken::Punctuation(")".to_string()) && !self.is_at_end() {
                        args.push(self.parse_expression()?);
                        if self.peek() == &SflToken::Punctuation(",".to_string()) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect_punctuation(")")?;
                    Ok(SflExpr::FuncCall { name, args })
                } else {
                    Ok(SflExpr::Identifier(name))
                }
            }
            SflToken::Punctuation(p) if p == "(" => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_punctuation(")")?;
                Ok(SflExpr::Grouped(Box::new(expr)))
            }
            SflToken::Punctuation(p) if p == "[" => {
                self.advance();
                let mut elements = Vec::new();
                while self.peek() != &SflToken::Punctuation("]".to_string()) && !self.is_at_end() {
                    elements.push(self.parse_expression()?);
                    if self.peek() == &SflToken::Punctuation(",".to_string()) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect_punctuation("]")?;
                Ok(SflExpr::List(elements))
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{other}"),
            }),
        }
    }
}

/// Convenience function: tokenize and parse a .sfl source string.
pub fn parse(source: &str) -> Result<SflProgram> {
    let tokens = tokenize(source);
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Tokenizer tests ----

    #[test]
    fn test_tokenize_simple_expression() {
        let tokens = tokenize("1 + 2");
        assert_eq!(tokens.len(), 4); // Number(1), Operator(+), Number(2), Eof
        assert_eq!(tokens[0], SflToken::Number(1.0));
        assert_eq!(tokens[1], SflToken::Operator("+".to_string()));
        assert_eq!(tokens[2], SflToken::Number(2.0));
        assert_eq!(tokens[3], SflToken::Eof);
    }

    #[test]
    fn test_tokenize_identifiers_and_keywords() {
        let tokens = tokenize("let x = 42");
        assert_eq!(tokens[0], SflToken::Keyword("let".to_string()));
        assert_eq!(tokens[1], SflToken::Identifier("x".to_string()));
        assert_eq!(tokens[2], SflToken::Operator("=".to_string()));
        assert_eq!(tokens[3], SflToken::Number(42.0));
    }

    #[test]
    fn test_tokenize_string_literal() {
        let tokens = tokenize(r#"let msg = "hello world""#);
        assert_eq!(tokens[0], SflToken::Keyword("let".to_string()));
        assert_eq!(tokens[3], SflToken::String("hello world".to_string()));
    }

    #[test]
    fn test_tokenize_string_with_escapes() {
        let tokens = tokenize(r#""line1\nline2""#);
        assert_eq!(tokens[0], SflToken::String("line1\nline2".to_string()));
    }

    #[test]
    fn test_tokenize_float() {
        let tokens = tokenize("3.14");
        assert_eq!(tokens[0], SflToken::Number(3.14));
    }

    #[test]
    fn test_tokenize_comparison_operators() {
        let tokens = tokenize("x == 1 && y != 2");
        assert_eq!(tokens[1], SflToken::Operator("==".to_string()));
        assert_eq!(tokens[3], SflToken::Operator("&&".to_string()));
        assert_eq!(tokens[5], SflToken::Operator("!=".to_string()));
    }

    #[test]
    fn test_tokenize_comments() {
        let tokens = tokenize("1 // comment\n2 /* block */ 3");
        assert_eq!(tokens[0], SflToken::Number(1.0));
        assert_eq!(tokens[1], SflToken::Number(2.0));
        assert_eq!(tokens[2], SflToken::Number(3.0));
        assert_eq!(tokens[3], SflToken::Eof);
    }

    #[test]
    fn test_tokenize_brackets_and_semicolons() {
        let tokens = tokenize("fn foo(x, y) { }");
        assert_eq!(tokens[0], SflToken::Keyword("fn".to_string()));
        assert_eq!(tokens[1], SflToken::Identifier("foo".to_string()));
        assert_eq!(tokens[2], SflToken::Punctuation("(".to_string()));
        assert_eq!(tokens[3], SflToken::Identifier("x".to_string()));
        assert_eq!(tokens[4], SflToken::Punctuation(",".to_string()));
        assert_eq!(tokens[5], SflToken::Identifier("y".to_string()));
        assert_eq!(tokens[6], SflToken::Punctuation(")".to_string()));
        assert_eq!(tokens[7], SflToken::Punctuation("{".to_string()));
        assert_eq!(tokens[8], SflToken::Punctuation("}".to_string()));
    }

    #[test]
    fn test_tokenize_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], SflToken::Eof);
    }

    #[test]
    fn test_tokenize_all_keywords() {
        let tokens = tokenize("let fn if else while repeat for in return break continue true false null and or not");
        let keyword_count = tokens
            .iter()
            .filter(|t| matches!(t, SflToken::Keyword(_)))
            .count();
        assert_eq!(keyword_count, 17);
    }

    // ---- Parser tests ----

    #[test]
    fn test_parse_variable_declaration() {
        let ast = parse("let x = 42;").unwrap();
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            SflStatement::VarDecl { name, value } => {
                assert_eq!(name, "x");
                assert_eq!(*value, SflExpr::Number(42.0));
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_variable_declaration_with_string() {
        let ast = parse(r#"let msg = "hello";"#).unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { name, value } => {
                assert_eq!(name, "msg");
                assert_eq!(*value, SflExpr::String("hello".to_string()));
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let ast = parse("x = 10;").unwrap();
        match &ast.statements[0] {
            SflStatement::Assignment { name, value } => {
                assert_eq!(name, "x");
                assert_eq!(*value, SflExpr::Number(10.0));
            }
            other => panic!("expected Assignment, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let ast = parse("fn greet(name) { say(name); }").unwrap();
        match &ast.statements[0] {
            SflStatement::FuncDecl { name, params, body } => {
                assert_eq!(name, "greet");
                assert_eq!(*params, vec!["name"]);
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected FuncDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let ast = parse("if x > 0 { say(x); } else { say(0); }").unwrap();
        match &ast.statements[0] {
            SflStatement::IfStmt {
                condition,
                then_body,
                else_body,
            } => {
                // Condition: x > 0
                assert!(matches!(
                    condition,
                    SflExpr::BinaryOp { op, .. } if op == ">"
                ));
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_some());
                assert_eq!(else_body.as_ref().unwrap().len(), 1);
            }
            other => panic!("expected IfStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_if_no_else() {
        let ast = parse("if x == 1 { say(x); }").unwrap();
        match &ast.statements[0] {
            SflStatement::IfStmt { else_body, .. } => {
                assert!(else_body.is_none());
            }
            other => panic!("expected IfStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let ast = parse("while x < 10 { x = x + 1; }").unwrap();
        match &ast.statements[0] {
            SflStatement::WhileLoop { condition, body } => {
                assert!(matches!(
                    condition,
                    SflExpr::BinaryOp { op, .. } if op == "<"
                ));
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected WhileLoop, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_repeat_loop() {
        let ast = parse("repeat 10 { move(1); }").unwrap();
        match &ast.statements[0] {
            SflStatement::RepeatLoop { count, body } => {
                assert_eq!(*count, SflExpr::Number(10.0));
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected RepeatLoop, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let ast = parse("for item in items { say(item); }").unwrap();
        match &ast.statements[0] {
            SflStatement::ForLoop {
                name,
                iterable,
                body,
            } => {
                assert_eq!(name, "item");
                assert!(matches!(iterable, SflExpr::Identifier(n) if n == "items"));
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected ForLoop, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let ast = parse("move(10);").unwrap();
        match &ast.statements[0] {
            SflStatement::ExprStmt(expr) => {
                assert!(matches!(
                    expr,
                    SflExpr::FuncCall { name, args } if name == "move" && args.len() == 1
                ));
            }
            other => panic!("expected ExprStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_binary_expression_precedence() {
        // "1 + 2 * 3" should parse as 1 + (2 * 3)
        let ast = parse("let x = 1 + 2 * 3;").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                // Should be: BinaryOp { left: 1, op: +, right: BinaryOp { left: 2, op: *, right: 3 } }
                match value {
                    SflExpr::BinaryOp { left, op, right } => {
                        assert_eq!(op, "+");
                        assert_eq!(**left, SflExpr::Number(1.0));
                        match &**right {
                            SflExpr::BinaryOp {
                                op: mul_op,
                                left: mul_left,
                                right: mul_right,
                            } => {
                                assert_eq!(mul_op, "*");
                                assert_eq!(**mul_left, SflExpr::Number(2.0));
                                assert_eq!(**mul_right, SflExpr::Number(3.0));
                            }
                            other => panic!("expected nested BinaryOp, got: {other:?}"),
                        }
                    }
                    other => panic!("expected BinaryOp, got: {other:?}"),
                }
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_grouped_expression() {
        // "(1 + 2) * 3" should parse as (1+2) * 3
        let ast = parse("let x = (1 + 2) * 3;").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                match value {
                    SflExpr::BinaryOp { left, op, right } => {
                        assert_eq!(op, "*");
                        // Left side should be a Grouped expression
                        match &**left {
                            SflExpr::Grouped(inner) => {
                                assert!(matches!(
                                    &**inner,
                                    SflExpr::BinaryOp { op, .. } if op == "+"
                                ));
                            }
                            other => panic!("expected Grouped, got: {other:?}"),
                        }
                        assert_eq!(**right, SflExpr::Number(3.0));
                    }
                    other => panic!("expected BinaryOp, got: {other:?}"),
                }
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_unary_negation() {
        let ast = parse("let x = -5;").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                assert!(matches!(
                    value,
                    SflExpr::UnaryOp { op, .. } if op == "-"
                ));
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_unary_not() {
        let ast = parse("let x = !flag;").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                assert!(matches!(
                    value,
                    SflExpr::UnaryOp { op, .. } if op == "!"
                ));
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_list_literal() {
        let ast = parse("let items = [1, 2, 3];").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                match value {
                    SflExpr::List(elements) => {
                        assert_eq!(elements.len(), 3);
                        assert_eq!(elements[0], SflExpr::Number(1.0));
                        assert_eq!(elements[1], SflExpr::Number(2.0));
                        assert_eq!(elements[2], SflExpr::Number(3.0));
                    }
                    other => panic!("expected List, got: {other:?}"),
                }
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_member_access() {
        let ast = parse("sprite.x;").unwrap();
        match &ast.statements[0] {
            SflStatement::ExprStmt(expr) => {
                match expr {
                    SflExpr::MemberAccess { object, member } => {
                        assert!(matches!(&**object, SflExpr::Identifier(n) if n == "sprite"));
                        assert_eq!(member, "x");
                    }
                    other => panic!("expected MemberAccess, got: {other:?}"),
                }
            }
            other => panic!("expected ExprStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_index_access() {
        let ast = parse("items[0];").unwrap();
        match &ast.statements[0] {
            SflStatement::ExprStmt(expr) => {
                match expr {
                    SflExpr::IndexAccess { object, index } => {
                        assert!(matches!(&**object, SflExpr::Identifier(n) if n == "items"));
                        assert_eq!(**index, SflExpr::Number(0.0));
                    }
                    other => panic!("expected IndexAccess, got: {other:?}"),
                }
            }
            other => panic!("expected ExprStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_bool_literals() {
        let ast = parse("let a = true; let b = false;").unwrap();
        assert_eq!(ast.statements.len(), 2);
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                assert_eq!(*value, SflExpr::Bool(true));
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
        match &ast.statements[1] {
            SflStatement::VarDecl { value, .. } => {
                assert_eq!(*value, SflExpr::Bool(false));
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_null_literal() {
        let ast = parse("let x = null;").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                assert_eq!(*value, SflExpr::Null);
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let ast = parse("fn foo() { return 42; }").unwrap();
        match &ast.statements[0] {
            SflStatement::FuncDecl { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    SflStatement::Return(value) => {
                        assert!(value.is_some());
                        assert_eq!(*value.as_ref().unwrap(), SflExpr::Number(42.0));
                    }
                    other => panic!("expected Return, got: {other:?}"),
                }
            }
            other => panic!("expected FuncDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_return_void() {
        let ast = parse("fn bar() { return; }").unwrap();
        match &ast.statements[0] {
            SflStatement::FuncDecl { body, .. } => {
                match &body[0] {
                    SflStatement::Return(value) => {
                        assert!(value.is_none());
                    }
                    other => panic!("expected Return, got: {other:?}"),
                }
            }
            other => panic!("expected FuncDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_break_and_continue() {
        let ast = parse("while true { break; continue; }").unwrap();
        match &ast.statements[0] {
            SflStatement::WhileLoop { body, .. } => {
                assert_eq!(body.len(), 2);
                assert_eq!(body[0], SflStatement::Break);
                assert_eq!(body[1], SflStatement::Continue);
            }
            other => panic!("expected WhileLoop, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_complex_program() {
        let source = r#"
            let x = 10;
            let y = 20;

            fn add(a, b) {
                return a + b;
            }

            if x > 5 {
                let result = add(x, y);
                say(result);
            } else {
                say(0);
            }

            repeat 5 {
                move(10);
                turn(15);
            }
        "#;
        let ast = parse(source).unwrap();
        // let x, let y, fn add, if, repeat = 5 top-level statements
        assert_eq!(ast.statements.len(), 5);
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let result = parse("if { }");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_program() {
        let ast = parse("").unwrap();
        assert!(ast.statements.is_empty());
    }

    #[test]
    fn test_parse_else_if_chain() {
        let source = r#"
            if x == 1 {
                say("one");
            } else if x == 2 {
                say("two");
            } else {
                say("other");
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            SflStatement::IfStmt {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 1);
                let else_stmts = else_body.as_ref().unwrap();
                assert_eq!(else_stmts.len(), 1);
                // The else body should contain another IfStmt (else if)
                assert!(matches!(&else_stmts[0], SflStatement::IfStmt { .. }));
            }
            other => panic!("expected IfStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_function_with_multiple_params() {
        let ast = parse("fn foo(a, b, c) { return a + b + c; }").unwrap();
        match &ast.statements[0] {
            SflStatement::FuncDecl { params, .. } => {
                assert_eq!(*params, vec!["a", "b", "c"]);
            }
            other => panic!("expected FuncDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_logical_operators() {
        let ast = parse("let x = true && false || true;").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                // Should parse as (true && false) || true
                match value {
                    SflExpr::BinaryOp { op, .. } => {
                        assert_eq!(op, "||");
                    }
                    other => panic!("expected BinaryOp with ||, got: {other:?}"),
                }
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_chained_member_access() {
        let ast = parse("sprite.pos.x;").unwrap();
        match &ast.statements[0] {
            SflStatement::ExprStmt(expr) => {
                // Should be MemberAccess { object: MemberAccess { object: Identifier("sprite"), member: "pos" }, member: "x" }
                match expr {
                    SflExpr::MemberAccess { object, member } => {
                        assert_eq!(member, "x");
                        match &**object {
                            SflExpr::MemberAccess { member: inner_member, .. } => {
                                assert_eq!(inner_member, "pos");
                            }
                            other => panic!("expected inner MemberAccess, got: {other:?}"),
                        }
                    }
                    other => panic!("expected MemberAccess, got: {other:?}"),
                }
            }
            other => panic!("expected ExprStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_empty_list() {
        let ast = parse("let empty = [];").unwrap();
        match &ast.statements[0] {
            SflStatement::VarDecl { value, .. } => {
                match value {
                    SflExpr::List(elements) => {
                        assert!(elements.is_empty());
                    }
                    other => panic!("expected List, got: {other:?}"),
                }
            }
            other => panic!("expected VarDecl, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_function_call_no_args() {
        let ast = parse("reset();").unwrap();
        match &ast.statements[0] {
            SflStatement::ExprStmt(expr) => {
                match expr {
                    SflExpr::FuncCall { name, args } => {
                        assert_eq!(name, "reset");
                        assert!(args.is_empty());
                    }
                    other => panic!("expected FuncCall, got: {other:?}"),
                }
            }
            other => panic!("expected ExprStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let ast = parse("say(add(1, 2));").unwrap();
        match &ast.statements[0] {
            SflStatement::ExprStmt(expr) => {
                match expr {
                    SflExpr::FuncCall { name, args } => {
                        assert_eq!(name, "say");
                        assert_eq!(args.len(), 1);
                        match &args[0] {
                            SflExpr::FuncCall { name: inner_name, args: inner_args } => {
                                assert_eq!(inner_name, "add");
                                assert_eq!(inner_args.len(), 2);
                            }
                            other => panic!("expected inner FuncCall, got: {other:?}"),
                        }
                    }
                    other => panic!("expected FuncCall, got: {other:?}"),
                }
            }
            other => panic!("expected ExprStmt, got: {other:?}"),
        }
    }

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", SflToken::Number(42.0)), "Number(42)");
        assert_eq!(format!("{}", SflToken::Identifier("x".into())), "Identifier(x)");
        assert_eq!(format!("{}", SflToken::String("hi".into())), "String(\"hi\")");
        assert_eq!(format!("{}", SflToken::Operator("+".into())), "Operator(+)");
        assert_eq!(format!("{}", SflToken::Keyword("let".into())), "Keyword(let)");
        assert_eq!(format!("{}", SflToken::Punctuation("(".into())), "Punctuation(()");
        assert_eq!(format!("{}", SflToken::Eof), "Eof");
    }
}
