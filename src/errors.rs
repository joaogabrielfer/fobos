use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use thiserror::Error;

use crate::{
    ast::{BinaryOp, UnaryOp},
    diagnostic::render_source_span,
    interpreter::values::Value,
    source::Span,
    typechecker::ty::ParameterTypes,
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
                let error_red = "tokenizer error".red();
                write!(
                    f,
                    "{error_red}: {}\n{spaces}{arrow} {}:{line}:{col}:\n{spaces} {pipe}\n{} {pipe}   {snippet}",
                    self.kind,
                    self.file_path.display(),
                    line.to_string().cyan(),
                )
            }
            Err(_) => {
                let arrow = "-->".cyan();
                let error_red = "error".red();
                write!(
                    f,
                    "{error_red}: {}\n {arrow} {}:{}:{}",
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
    #[error("unrecognized char '{0}'")]
    UnknownChar(char),
    #[error("unrecognized token '{0}'")]
    UnknownToken(String),
    #[error("invalid number '{0}'")]
    InvalidNumber(String),
    #[error("unterminated string")]
    UnterminatedString,
    #[error("expected chars {}, found {found}", render_vec_tokens(expected.clone()))]
    ExpectedChars {
        expected: Vec<String>,
        found: String,
    },
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
                let error_red = "parser error".red();
                write!(
                    f,
                    "{error_red}: {}\n{spaces}{arrow} {}:{line}:{col}:\n{spaces} {pipe}\n{} {pipe}   {snippet}",
                    self.kind,
                    self.file_path.display(),
                    line.to_string().cyan(),
                )
            }
            Err(_) => {
                let arrow = "-->".cyan();
                let error_red = "error".red();
                write!(
                    f,
                    "{error_red}: {}\n {arrow} {}:{}:{}",
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
    #[error("cannot chain more than one range operations")]
    ChainingRanges,
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

pub fn render_types_from_value_vector(values: Vec<Value>) -> String {
    let mut s = String::new();
    s = format!("{s}(");
    for (i, v) in values.iter().enumerate() {
        s = format!("{s}{}", v.get_type());
        if i < values.len() - 1 {
            s = format!("{s}, ");
        }
    }
    s = format!("{s})");
    s
}

pub fn render_parameter_types(overloaded_parameters: Vec<ParameterTypes>) -> String {
    let mut s = String::new();
    s = format!("{s}(");
    for (i, parameters_types) in overloaded_parameters.iter().enumerate() {
        for (j, param) in parameters_types.iter().enumerate() {
            s = format!("{s}{param}");
            if j < parameters_types.len() - 1 {
                s = format!("{s}, ");
            }
        }
        if i < overloaded_parameters.len() - 1 {
            s = format!("{s}| ");
        }
    }
    s = format!("{s})");
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
                let error_red = "runtime error".red();
                write!(
                    f,
                    "{error_red}: {}\n{spaces}{arrow} {}:{line}:{col}:\n{spaces} {pipe}\n{} {pipe}   {snippet}",
                    self.kind,
                    self.file_path.display(),
                    line.to_string().cyan(),
                )
            }
            Err(_) => {
                let error_red = "error".red();
                write!(f, "{error_red}: {}", self.kind)
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
    #[error("invalid function parameter '{0}'")]
    InvalidFunctionParameter(String),
    #[error("invalid function parameters '{0}'")]
    InvalidFunctionParameters(String),
    #[error("expected boolean, found '{found}'")]
    ExpectedBool { found: String },
    #[error("'{0}' is not callable")]
    NotCallable(String),
    #[error("index {0} was outside of the bound of the array")]
    OutOfBounds(i64),
    #[error("expected signature of {expected}, but found {found}")]
    SignatureMismatch { expected: String, found: String },
    #[error("missing 'else' branch for 'if' condition that yields a value")]
    ElseBranchMissing,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("not implemented")]
    NotImplemented,
    #[error("yield used outside of an effect handler")]
    YieldOutsideHandler,
    #[error("{found} is not an iterable value")]
    NotIterable { found: String },
    #[error("{found} is not a valid range step")]
    BadRangeStep { found: String },
    #[error("expected array, but found {found}")]
    ExpectedArray { found: String },
    #[error("mismatched returned types, expected '{expected}', but got '{found}'")]
    MismatchedReturnTypes { expected: String, found: String },
}

#[derive(Debug, Error)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub span: Span,
    pub file_path: PathBuf,
}

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match render_source_span(&self.file_path, self.span) {
            Ok((line, col, snippet)) => {
                let spaces = (0..line.to_string().len()).map(|_| ' ').collect::<String>();
                let pipe = "|".cyan();
                let arrow = "-->".cyan();
                let error_red = "type error".red();
                write!(
                    f,
                    "{error_red}: {}\n{spaces}{arrow} {}:{line}:{col}:\n{spaces} {pipe}\n{} {pipe}   {snippet}",
                    self.kind,
                    self.file_path.display(),
                    line.to_string().cyan(),
                )
            }
            Err(_) => {
                let error_red = "error".red();
                write!(f, "{error_red}: {}", self.kind)
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum TypeErrorKind {
    #[error("mismatched types, expected '{expected}' but got '{found}'")]
    MismatchedType { expected: String, found: String },
    #[error("mismatched array members types, expected '{expected} for array' but got '{found}'")]
    MismatchedArrayType { expected: String, found: String },
    #[error("mismatched yielded types, expected '{expected}', but got '{found}'")]
    MismatchedYieldTypes { expected: String, found: String },
    #[error("mismatched returned types, expected '{expected}', but got '{found}'")]
    MismatchedReturnTypes { expected: String, found: String },
    #[error("mismatched types on branches, expected '{expected}', but got '{found}'")]
    MismatchedBranchTypes { expected: String, found: String },
    #[error("'{lhs}' and '{rhs}' are not valid types for operation '{op}'")]
    MismatchedBinaryOpType {
        op: BinaryOp,
        lhs: String,
        rhs: String,
    },
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("{found} is not an iterable type")]
    NotIterable { found: String },
    #[error("{0} is not a valid assignment target")]
    InvalidAssignmentTarget(String),
    #[error("{0} is not a valid indexing target")]
    InvalidIndexingTarget(String),
    #[error("{0} is not a valid indexing type")]
    InvalidIndexType(String),
    #[error("cannot return outside a function")]
    ReturnOutsideFunction,
    #[error("yield used outside of an effect handler")]
    YieldOutsideHandler,
}
