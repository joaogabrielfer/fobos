use std::fmt::Display;

use crate::{
    source::Span,
    typechecker::{TypeResult, ty::Type},
};

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Return(Expr),
    Yield(Expr),

    Bind {
        mutable: bool,
        name: String,
        type_annotation: TypeAnnotation,
        value: Expr,
        span: Span,
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
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Expr(expr) => expr.span,
            Stmt::Return(expr) => expr.span,
            Stmt::Yield(expr) => expr.span,
            Stmt::Bind { span, .. } => *span,
            Stmt::Assignment { target, value } => Span {
                start: target.span.start,
                end: value.span.end,
            },
            Stmt::FunDecl { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Inferred,
    Explicit(TypeExpr),
}

impl TypeAnnotation {
    pub fn resolve_type_annotation(&self) -> TypeResult<Type> {
        let TypeAnnotation::Explicit(t) = self else {
            return Ok(Type::Any);
        };
        match t {
            TypeExpr::Named(name) if name == "Int" => Ok(Type::Int),
            TypeExpr::Named(name) if name == "Float" => Ok(Type::Float),
            TypeExpr::Named(name) if name == "Bool" => Ok(Type::Bool),
            TypeExpr::Named(name) if name == "String" => Ok(Type::String),
            TypeExpr::Named(name) => Ok(Type::TypeVar(name.clone())), // temporary for generics
            TypeExpr::Unit => Ok(Type::Unit),
            // TODO: Array isnt
            // implemented in the
            // tokenizer yet
            TypeExpr::Array(inner) => Ok(Type::Array(Box::new(
                TypeAnnotation::Explicit(TypeExpr::Named(inner.clone()))
                    .resolve_type_annotation()?,
            ))),
            TypeExpr::Tuple(items) => {
                let mut types = Vec::new();

                for item in items {
                    types.push(
                        TypeAnnotation::Explicit(TypeExpr::Named(item.clone()))
                            .resolve_type_annotation()?,
                    );
                }

                Ok(Type::Tuple(types))
            }
            TypeExpr::Function {
                parameters,
                return_type,
            } => {
                let parameters = parameters
                    .iter()
                    .map(|p| {
                        TypeAnnotation::Explicit(TypeExpr::Named(p.clone()))
                            .resolve_type_annotation()
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let ret =
                    TypeAnnotation::Explicit(*return_type.clone()).resolve_type_annotation()?;

                Ok(Type::Function {
                    parameters_types: parameters,
                    return_type: Box::new(ret),
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Named(String),
    Tuple(Vec<String>),
    Array(String),
    Function {
        parameters: Vec<String>,
        return_type: Box<TypeExpr>,
    },
    Unit,
}

impl TypeExpr {
    pub fn resolve_type_expr(&self) -> TypeResult<Type> {
        match self {
            TypeExpr::Named(name) if name == "Int" => Ok(Type::Int),
            TypeExpr::Named(name) if name == "Float" => Ok(Type::Float),
            TypeExpr::Named(name) if name == "Bool" => Ok(Type::Bool),
            TypeExpr::Named(name) if name == "String" => Ok(Type::String),
            TypeExpr::Named(name) => Ok(Type::TypeVar(name.clone())), // temporary for generics
            TypeExpr::Unit => Ok(Type::Unit),
            // TODO: Array isnt
            // implemented in the
            // tokenizer yet
            TypeExpr::Array(inner) => Ok(Type::Array(Box::new(
                TypeAnnotation::Explicit(TypeExpr::Named(inner.clone()))
                    .resolve_type_annotation()?,
            ))),
            TypeExpr::Tuple(items) => {
                let mut types = Vec::new();

                for item in items {
                    types.push(
                        TypeAnnotation::Explicit(TypeExpr::Named(item.clone()))
                            .resolve_type_annotation()?,
                    );
                }

                Ok(Type::Tuple(types))
            }
            TypeExpr::Function {
                parameters,
                return_type,
            } => {
                let parameters = parameters
                    .iter()
                    .map(|p| {
                        TypeAnnotation::Explicit(TypeExpr::Named(p.clone()))
                            .resolve_type_annotation()
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let ret =
                    TypeAnnotation::Explicit(*return_type.clone()).resolve_type_annotation()?;

                Ok(Type::Function {
                    parameters_types: parameters,
                    return_type: Box::new(ret),
                })
            }
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub t: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
    Unit,

    Block(Block),

    Tuple(Vec<Expr>),
    Array(Vec<Expr>),

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

    For {
        binding: Box<Expr>,
        iterable: Box<Expr>,
        block: Block,
    },

    Lambda {
        parameters: Vec<Parameter>,
        body: Box<Expr>,
    },

    Index {
        target: Box<Expr>,
        index: Box<Expr>,
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
            ExprKind::Array(_) => write!(f, "array"),
            ExprKind::Unary { .. } => write!(f, "unary operation"),
            ExprKind::Binary { .. } => write!(f, "binary operation"),
            ExprKind::Call { .. } => write!(f, "function calling"),
            ExprKind::If { .. } => write!(f, "if condition"),
            ExprKind::While { .. } => write!(f, "while loop"),
            ExprKind::For { .. } => write!(f, "for loop"),
            ExprKind::Lambda { .. } => write!(f, "lambda"),
            ExprKind::Index { .. } => write!(f, "index"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

impl Block {
    pub fn span(&self) -> Span {
        let start = self.statements.first();
        let end = self.statements.last();

        match (start, end) {
            (Some(s), Some(e)) => Span {
                start: s.span().start,
                end: e.span().end,
            },
            _ => Span::dummy(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    Combine,
    InclusiveRange,
    ExclusiveRange,
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
            BinaryOp::Combine => write!(f, "<>"),
            BinaryOp::InclusiveRange => write!(f, "..="),
            BinaryOp::ExclusiveRange => write!(f, "..<"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
