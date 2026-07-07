use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use thiserror::Error;

use crate::{
    ast::{BinaryOp, UnaryOp},
    diagnostic::render_source_span,
    source::Span,
};

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
    #[error("expected {expected} function parameters, found {found}")]
    ArityMismatch { expected: usize, found: usize },
    #[error("missing 'else' branch for 'if' condition that yields a value")]
    ElseBranchMissing,
}
