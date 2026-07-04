use std::path::PathBuf;

use anyhow::Result;
use thiserror::Error;

use crate::lexer::{Token, TokenKind, TokenTag};
use crate::source::Span;

#[derive(Debug, Clone)]
pub struct Program {
    statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Bind {
        mutable: bool,
        name: String,
        type_annotation: Option<Type>,
        value: Expr,
    },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    file_path: &'a PathBuf,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>, file_path: &'a PathBuf) -> Self {
        Self {
            tokens,
            file_path,
            index: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let mut statements = vec![];
        self.consume_newlines();

        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            // eprintln!("{:#?}", stmt.clone());
            statements.push(stmt);
            self.consume_newlines();
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParserError> {
        match self.current().kind {
            TokenKind::Let => self.parse_binding(false),
            TokenKind::Var => self.parse_binding(true),
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
        }
    }

    fn parse_binding(&mut self, mutable: bool) -> Result<Stmt, ParserError> {
        if mutable {
            self.expect(TokenTag::Var)?;
        } else {
            self.expect(TokenTag::Let)?;
        }
        let name = self.expect_ident()?;
        let type_annotation = self.parse_type()?;
        self.expect(TokenTag::Equals)?;
        let value = self.parse_expr()?;

        Ok(Stmt::Bind {
            mutable,
            name,
            type_annotation,
            value,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        let value = match self.current().kind.clone() {
            TokenKind::Int(n) => Ok(Expr::Int(n)),
            TokenKind::Float(n) => Ok(Expr::Float(n)),
            TokenKind::Bool(b) => Ok(Expr::Bool(b)),
            TokenKind::String(s) => Ok(Expr::String(s.to_string())),
            TokenKind::Ident(i) => Ok(Expr::Ident(i.to_string())),
            other => Err(self.error(ParserErrorKind::ExpectedExpression {
                found: other.tag().to_string(),
            })),
        };
        self.advance();
        value
    }

    fn parse_type(&mut self) -> Result<Option<Type>, ParserError> {
        self.expect(TokenTag::Colon)?;
        match self.current().kind.clone() {
            TokenKind::Equals => Ok(None),
            TokenKind::Ident(t) => {
                self.advance();
                Ok(Some(Type::Named(t.to_string())))
            }
            other => Err(self.error(ParserErrorKind::ExpectedTokens {
                expected: vec!["=".to_string(), "type".to_string()],
                found: format!("{other:?}"),
            })),
        }
    }

    fn current(&self) -> &Token<'a> {
        &self.tokens[self.index]
    }

    fn current_tag(&self) -> TokenTag {
        self.current().kind.tag()
    }

    fn check(&self, tag: TokenTag) -> bool {
        self.current_tag() == tag
    }

    fn is_at_end(&self) -> bool {
        self.check(TokenTag::Eof)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.index += 1
        }
    }

    fn _previous(&self) -> Option<&Token<'a>> {
        if self.index > 0 {
            Some(&self.tokens[self.index - 1])
        } else {
            None
        }
    }

    fn consume_newlines(&mut self) {
        while self.check(TokenTag::NewLine) {
            self.advance();
        }
    }

    fn error(&self, kind: ParserErrorKind) -> ParserError {
        ParserError {
            kind,
            file_path: self.file_path.clone(),
            pos: self.current().span,
        }
    }

    fn expect(&mut self, tag: TokenTag) -> Result<(), ParserError> {
        if self.current_tag() == tag {
            self.advance();
            Ok(())
        } else {
            Err(self.error(ParserErrorKind::ExpectedToken {
                expected: tag.to_string(),
                found: self.current().origin.to_string(),
            }))
        }
    }
    fn expect_ident(&mut self) -> Result<String, ParserError> {
        match self.current().kind.clone() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s.to_string())
            }
            other => Err(self.error(ParserErrorKind::ExpectedIdentifier {
                found: format!("{other:?}"),
            })),
        }
    }
}

#[derive(Error, Debug)]
#[error("{file_path}:{pos}: ERROR: {kind}")]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub file_path: PathBuf,
    pub pos: Span,
}

#[derive(Error, Debug)]
pub enum ParserErrorKind {
    #[error("expected token '{expected}', found '{found}'")]
    ExpectedToken { expected: String, found: String },
    #[error("expected tokens '{expected:?}', found '{found}'")]
    ExpectedTokens {
        expected: Vec<String>,
        found: String,
    },
    #[error("expected identifier, found '{found}'")]
    ExpectedIdentifier { found: String },
    #[error("expected expression, found '{found}'")]
    ExpectedExpression { found: String },
    #[error("unexpected token '{found}'")]
    UnexpectedToken { found: String },
    #[error("unexpected EOF")]
    UnexpectedEof,
}
