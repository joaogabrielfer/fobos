use std::fmt::Display;

use crate::source::Span;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Return(Expr),
    Yield(Expr),

    Bind {
        mutable: bool,
        name: String,
        type_annotation: TypeAnnotation,
        value: Expr,
    },

    Assignment {
        target: Expr,
        value: Expr,
    },

    FunDecl {
        name: String,
        generics: Vec<String>,
        parameters: Vec<Parameter>,
        return_type: TypeAnnotation,
        body: Block,
    },
}

#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    Inferred,
    Explicit(Type),
}

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
    Unit,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub t: Type,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
    Unit,

    Block(Block),

    Tuple(Vec<Expr>),

    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    Binary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },

    While {
        condition: Box<Expr>,
        block: Block,
    },

    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprKind::Int(i) => write!(f, "integer '{i}'"),
            ExprKind::Float(d) => write!(f, "float '{d}'"),
            ExprKind::String(s) => write!(f, "string '{s}'"),
            ExprKind::Bool(b) => write!(f, "bool '{b}'"),
            ExprKind::Ident(i) => write!(f, "identifier '{i}'"),
            ExprKind::Unit => write!(f, "unit"),
            ExprKind::Block(_) => write!(f, "block"),
            ExprKind::Tuple(_) => write!(f, "tuple"),
            ExprKind::Unary { .. } => write!(f, "unary operation"),
            ExprKind::Binary { .. } => write!(f, "binary operation"),
            ExprKind::Call { .. } => write!(f, "function calling"),
            ExprKind::If { .. } => write!(f, "if condition"),
            ExprKind::While { .. } => write!(f, "while loop"),
            ExprKind::Lambda { .. } => write!(f, "lambda"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Eq => write!(f, "="),
            BinaryOp::NotEq => write!(f, "!="),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::GreaterEq => write!(f, ">="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::LessEq => write!(f, "<="),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Negate => write!(f, "-"),
        }
    }
}
