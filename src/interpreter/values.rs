use crate::{
    ast::{Block, Expr, Parameter, TypeAnnotation},
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    Echo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub name: Option<String>,
    pub parameters: Vec<Parameter>,
    pub body: FunctionBody,
    pub captured_env: EnvRef,
    pub return_type: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Block(Block),
    Expr(Expr),
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
        }
    }
}
