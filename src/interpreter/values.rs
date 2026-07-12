use itertools::Itertools;

use crate::ast::{CallArgument, CallParameter, ValueArgument};
use crate::errors::{ArgumentError, ArgumentErrorKind};
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
                            .map(|p| crate::typechecker::ty::ParameterType {
                                name: p.name.clone(),
                                ty: p.t.resolve_type_annotation(),
                            })
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

pub struct MatchedCall<T> {
    pub variant_index: usize,
    pub arguments: Vec<T>,
}

pub fn normalize_arguments<P, T>(
    parameters: &[P],
    arguments: &[CallArgument<T>],
) -> Result<Vec<T>, Box<ArgumentError>>
where
    P: CallParameter,
    T: Clone,
{
    let mut bound: Vec<Option<T>> = vec![None; parameters.len()];
    let mut positional_index = 0;
    let mut saw_named_argument = false;

    for argument in arguments {
        match &argument.name {
            Some(name) => {
                saw_named_argument = true;

                let parameter_index = parameters
                    .iter()
                    .position(|parameter| parameter.name() == name.value)
                    .ok_or_else(|| {
                        Box::new(ArgumentError {
                            kind: ArgumentErrorKind::UnknownName {
                                name: name.value.clone(),
                            },
                            span: Some(name.span),
                        })
                    })?;

                if bound[parameter_index].is_some() {
                    return Err(Box::new(ArgumentError {
                        kind: ArgumentErrorKind::Duplicate {
                            name: name.value.clone(),
                        },
                        span: Some(argument.span),
                    }));
                }

                bound[parameter_index] = Some(argument.value.clone());
            }

            None => {
                if saw_named_argument {
                    return Err(Box::new(ArgumentError {
                        kind: { ArgumentErrorKind::PositionalAfterNamed },
                        span: Some(argument.span),
                    }));
                }

                while positional_index < bound.len() && bound[positional_index].is_some() {
                    positional_index += 1;
                }

                if positional_index >= parameters.len() {
                    return Err(Box::new(ArgumentError {
                        kind: ArgumentErrorKind::TooMany,
                        span: Some(argument.span),
                    }));
                }

                bound[positional_index] = Some(argument.value.clone());
                positional_index += 1;
            }
        }
    }

    parameters
        .iter()
        .zip(bound)
        .map(|(parameter, argument)| {
            argument.ok_or_else(|| {
                Box::new(ArgumentError {
                    kind: ArgumentErrorKind::Missing {
                        name: parameter.name().to_string(),
                    },
                    span: None,
                })
            })
        })
        .collect()
}

pub fn arguments_match(parameters: &[Type], arguments: &[Type]) -> bool {
    parameters
        .iter()
        .zip(arguments)
        .all(|(parameter, argument)| TypeChecker::types_compatible(parameter, argument))
}

impl FunctionValue {
    pub fn match_variant(
        &self,
        arguments: Vec<ValueArgument>,
    ) -> Result<MatchedCall<Value>, Box<ArgumentError>> {
        let mut matches = Vec::new();
        let mut last_error = None;

        for (variant_index, variant) in self.overload_variants.iter().enumerate() {
            let normalized = match normalize_arguments(&variant.parameters, &arguments) {
                Ok(arguments) => arguments,
                Err(error) => {
                    last_error = Some(error);
                    continue;
                }
            };

            let parameter_types = variant
                .parameters
                .iter()
                .map(|parameter| parameter.t.resolve_type_annotation())
                .collect_vec();
            let argument_types = normalized.iter().map(Value::get_type).collect_vec();

            if arguments_match(&parameter_types, &argument_types) {
                matches.push(MatchedCall {
                    variant_index,
                    arguments: normalized,
                });
            } else if let Some((parameter, found)) = variant
                .parameters
                .iter()
                .zip(argument_types)
                .find(|(parameter, found)| {
                    !TypeChecker::types_compatible(&parameter.t.resolve_type_annotation(), found)
                })
            {
                last_error = Some(Box::new(ArgumentError {
                    kind: ArgumentErrorKind::TypeMismatch {
                        parameter: parameter.name.clone(),
                        expected: parameter.t.resolve_type_annotation(),
                        found,
                    },
                    span: None,
                }));
            }
        }

        match matches.len() {
            1 => Ok(matches.remove(0)),

            0 => Err(last_error.unwrap_or_else(|| {
                Box::new(ArgumentError {
                    kind: ArgumentErrorKind::Missing {
                        name: "<matching overload>".to_string(),
                    },
                    span: None,
                })
            })),

            _ => Err(Box::new(ArgumentError {
                kind: ArgumentErrorKind::Ambiguous,
                span: None,
            })),
        }
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
