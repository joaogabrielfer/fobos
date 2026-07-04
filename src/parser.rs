use std::path::PathBuf;

use thiserror::Error;

use crate::lexer::Token;
use crate::source::SrcPos;

pub struct Program {
    statements: Vec<Stmt>,
}

pub enum Stmt {
    Let {
        mutable: bool,
        name: String,
        value: Expr,
    },
    Expr(Expr),
}

pub enum Expr {
    Int(i64),
    String(String),
    Bool(bool),
    Ident(String),
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    file_path: &'a PathBuf,
}

impl<'a> Parser<'a> {
    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let statements = vec![];
        Ok(Program { statements })
    }
}

#[derive(Error, Debug)]
#[error("{file_path}:{pos}: ERROR: {kind}")]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub file_path: PathBuf,
    pub pos: SrcPos,
}

#[derive(Error, Debug)]
pub enum ParserErrorKind {
    #[error("expected token '{expected}', found '{found}'")]
    ExpectedToken { expected: String, found: String },
    #[error("expected identifier, found '{found}'")]
    ExpectedIdentifier { found: String },
    #[error("expected expression, found '{found}'")]
    ExpectedExpression { found: String },
    #[error("unexpected token '{found}'")]
    UnexpectedToken { found: String },
    #[error("unexpected EOF")]
    UnexpectedEof,
}
