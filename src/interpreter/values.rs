use itertools::{EitherOrBoth, Itertools};

use crate::interpreter::builtins::BuiltinFunction;
use crate::typechecker::TypeChecker;
use crate::typechecker::ty::Type;
use crate::{
    ast::{Block, Expr, Parameter, TypeAnnotation},
    errors::RuntimeErrorKind,
    interpreter::env::EnvRef,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Tuple(Vec<Value>),
    Array(Vec<Value>),

    BuiltinFunction(BuiltinFunction),
    Function(FunctionValue),

    Range(RangeValue),
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::Unit => Type::Unit,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Bool(_) => Type::Bool,
            Value::String(_) => Type::String,
            Value::Tuple(values) => Type::Tuple(values.iter().map(|v| v.get_type()).collect()),
            Value::Array(values) => Type::Array(
                values
                    .first()
                    .map(|v| Box::new(v.get_type()))
                    .unwrap_or(Box::new(Type::Any)),
            ),
            Value::BuiltinFunction(builtin_function) => builtin_function.get_type(),
            Value::Function(function_value) => Type::Function {
                parameter_overloads: function_value
                    .overload_variants
                    .iter()
                    .map(|v| {
                        v.parameters
                            .iter()
                            .map(|p| p.t.resolve_type_annotation())
                            .collect()
                    })
                    .collect(),
                return_type: Box::new(function_value.return_type.resolve_type_annotation()),
            },
            Value::Range(_) => Type::Range,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeValue {
    pub start: i64,
    pub end: i64,
    pub inclusive: bool,
    pub step: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub name: Option<String>,
    pub overload_variants: Vec<OverloadFunctionVariant>,
    pub return_type: TypeAnnotation,
}

impl FunctionValue {
    pub fn match_variant(&self, caller_parameters: &[Value]) -> Option<usize> {
        for (i, variant) in self.overload_variants.iter().enumerate() {
            if variant
                .parameters
                .iter()
                .map(|p| p.t.resolve_type_annotation())
                .zip_longest(
                    caller_parameters
                        .iter()
                        .map(|v| v.get_type())
                        .collect::<Vec<Type>>(),
                )
                .all(|pair| match pair {
                    EitherOrBoth::Both(a, b) => TypeChecker::types_compatible(&a, &b),
                    _ => false,
                })
            {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Block(Block),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverloadFunctionVariant {
    pub parameters: Vec<Parameter>,
    pub body: FunctionBody,
    pub captured_env: EnvRef,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Tuple(values) => {
                write!(f, "(")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{value}")?;
                }
                write!(f, ")")
            }
            Value::Array(values) => {
                write!(f, "[")?;

                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{value}")?;
                }
                write!(f, "]")
            }
            Value::BuiltinFunction(_) => write!(f, "<builtin function>"),
            Value::Function(_) => write!(f, "<function>"),
            Value::Range(r) => write!(
                f,
                "{}..{}{}",
                r.start,
                if r.inclusive { "=" } else { "<" },
                r.end
            ),
        }
    }
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Unit => "Unit".to_string(),
            Value::Int(v) => format!("Int '{v}'"),
            Value::Float(v) => format!("Float '{v}'"),
            Value::Bool(v) => format!("Bool '{v}'"),
            Value::String(v) => format!("String '{v}'"),
            Value::Tuple(v) => {
                let mut s = "Tuple '(".to_string();
                for (i, item) in v.iter().enumerate() {
                    s = format!("{s}{item}");
                    if i < v.len() - 1 {
                        s = format!("{s}, ");
                    }
                }
                format!("{s})")
            }
            Value::Array(v) => {
                let mut s = "Array '[".to_string();
                for (i, item) in v.iter().enumerate() {
                    s = format!("{s}{item}");
                    if i < v.len() - 1 {
                        s = format!("{s}, ");
                    }
                }
                format!("{s}]")
            }
            Value::BuiltinFunction(_) => "<builtin function>".to_string(),
            Value::Function(_) => "<function>".to_string(),
            Value::Range(r) => format!(
                "Range '{}..{}{}'",
                r.start,
                if r.inclusive { "=" } else { "<" },
                r.end
            ),
        }
    }
}

pub enum RuntimeIterator {
    Range {
        current: i64,
        end: i64,
        inclusive: bool,
        step: i64,
    },
    Array {
        values: Vec<Value>,
        index: usize,
    },
}

impl RuntimeIterator {
    pub fn from_value(value: Value) -> Result<Self, RuntimeErrorKind> {
        match value {
            Value::Range(range) => Ok(Self::Range {
                current: range.start,
                end: range.end,
                inclusive: range.inclusive,
                step: range.step,
            }),

            Value::Array(values) => Ok(Self::Array { values, index: 0 }),

            other => Err(RuntimeErrorKind::NotIterable {
                found: other.type_name().to_string(),
            }),
        }
    }

    pub fn next_value(&mut self) -> Option<Value> {
        match self {
            RuntimeIterator::Range {
                current,
                end,
                inclusive,
                step,
            } => {
                if (*current >= *end && !*inclusive) || (*current > *end) {
                    None
                } else {
                    let result = *current;
                    *current += *step;
                    Some(Value::Int(result))
                }
            }
            RuntimeIterator::Array { values, index } => {
                if *index >= values.len() {
                    None
                } else {
                    let result = values[*index].clone();
                    *index += 1;
                    Some(result)
                }
            }
        }
    }
}
