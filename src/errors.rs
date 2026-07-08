use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use thiserror::Error;

use crate::{
    ast::{BinaryOp, UnaryOp},
    diagnostic::render_source_span,
    source::Span,
};

#[derive(Error, Debug)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub file_path: PathBuf,
    pub pos: Span,
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match render_source_span(&self.file_path, self.pos) {
            Ok((line, col, snippet)) => {
                let spaces = (0..line.to_string().len()).map(|_| ' ').collect::<String>();
                let pipe = "|".cyan();
                let arrow = "-->".cyan();
                let error_red = "error".red();
                write!(
                    f,
                    "{error_red}: {}\n{spaces}{arrow} {}:{line}:{col}:\n{spaces} {pipe}\n{} {pipe}   {snippet}",
                    self.kind,
                    self.file_path.display(),
                    line.to_string().cyan(),
                )
            }
            Err(_) => {
                write!(
                    f,
                    "error: {}\n --> {}:{}:{}",
                    self.kind,
                    self.file_path.display(),
                    self.pos.start.line,
                    self.pos.start.col,
                )
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum LexerErrorKind {
    #[error("Unrecognized char '{0}'")]
    UnknownChar(char),
    #[error("Unrecognized Token '{0}'")]
    UnknownToken(String),
    #[error("Invalid number '{0}'")]
    InvalidNumber(String),
    #[error("Unterminated String")]
    UnterminatedString,
}

#[derive(Error, Debug)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub file_path: PathBuf,
    pub pos: Span,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match render_source_span(&self.file_path, self.pos) {
            Ok((line, col, snippet)) => {
                let spaces = (0..line.to_string().len()).map(|_| ' ').collect::<String>();
                let pipe = "|".cyan();
                let arrow = "-->".cyan();
                let error_red = "error".red();
                write!(
                    f,
                    "{error_red}: {}\n{spaces}{arrow} {}:{line}:{col}:\n{spaces} {pipe}\n{} {pipe}   {snippet}",
                    self.kind,
                    self.file_path.display(),
                    line.to_string().cyan(),
                )
            }
            Err(_) => {
                write!(
                    f,
                    "error: {}\n --> {}:{}:{}",
                    self.kind,
                    self.file_path.display(),
                    self.pos.start.line,
                    self.pos.start.col,
                )
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ParserErrorKind {
    #[error("expected token '{expected}', found '{found}'")]
    ExpectedToken { expected: String, found: String },
    #[error("expected tokens {} but found '{found}'", render_vec_tokens(expected.clone()))]
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
    #[error("{expr} is not a valid assignment target")]
    InvalidAssignmentTarget { expr: String },
    #[error("{expr} is not a valid lamda parameter")]
    InvalidParameter { expr: String },
    #[error("expected type annotation, got nothing")]
    ExpectedTypeAnnotation,
}

fn render_vec_tokens(tks: Vec<String>) -> String {
    let mut s = String::new();
    for (i, tk) in tks.iter().enumerate() {
        s = format!("{s}'");
        s = format!("{s}{tk}");
        s = format!("{s}'");
        if i < tks.len() - 1 {
            s = format!("{s} or ");
        }
    }
    s
}

#[derive(Debug, Error)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub span: Span,
    pub file_path: PathBuf,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match render_source_span(&self.file_path, self.span) {
            Ok((line, col, snippet)) => {
                let spaces = (0..line.to_string().len()).map(|_| ' ').collect::<String>();
                let pipe = "|".cyan();
                let arrow = "-->".cyan();
                let error_red = "error".red();
                write!(
                    f,
                    "{error_red}: {}\n{spaces}{arrow} {}:{line}:{col}:\n{spaces} {pipe}\n{} {pipe}   {snippet}",
                    self.kind,
                    self.file_path.display(),
                    line.to_string().cyan(),
                )
            }
            Err(_) => {
                write!(
                    f,
                    "error: {}\n --> {}:{}:{}",
                    self.kind,
                    self.file_path.display(),
                    self.span.start.line,
                    self.span.start.col,
                )
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum RuntimeErrorKind {
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("cannot assign to immutable variable '{0}'")]
    CannotAssignImmutable(String),
    #[error("{0} is not a valid assignment target")]
    InvalidAssignmentTarget(String),
    #[error("{0} is not a valid indexing target")]
    InvalidIndexingTarget(String),
    #[error("{0} is not a valid index")]
    InvalidIndex(String),
    #[error("invalid binary operation '{lhs} {op} {rhs}'")]
    InvalidBinaryOp {
        op: BinaryOp,
        lhs: String,
        rhs: String,
    },
    #[error("invalid unary operation '{op}{operand}'")]
    InvalidUnaryOp { op: UnaryOp, operand: String },
    #[error("expected boolean, found '{found}'")]
    ExpectedBool { found: String },
    #[error("'{0}' is not callable")]
    NotCallable(String),
    #[error("index {0} was outside of the bound of the array")]
    OutOfBounds(i64),
    #[error("expected {expected} function parameters, found {found}")]
    ArityMismatch { expected: usize, found: usize },
    #[error("missing 'else' branch for 'if' condition that yields a value")]
    ElseBranchMissing,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("not implemented")]
    NotImplemented,
    #[error("yield used outside of an effect handler")]
    YieldOutsideHandler,
}
