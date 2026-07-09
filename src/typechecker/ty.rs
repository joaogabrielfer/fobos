use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unit,
    Int,
    Float,
    Bool,
    String,

    Tuple(Vec<Type>),
    Array(Box<Type>),
    Range,

    Function {
        parameters_types: Vec<Type>,
        return_type: Box<Type>,
    },

    Any,
    Unknown,
    TypeVar(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "Unit"),
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
            Type::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    write!(f, "{item}")?;
                    if i < items.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            Type::Array(t) => write!(f, "Arr<{t}"),
            Type::Function {
                parameters_types,
                return_type,
            } => {
                write!(f, "(")?;
                for (i, param) in parameters_types.iter().enumerate() {
                    write!(f, "{param}")?;
                    if i < parameters_types.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") -> {return_type}")?;
                Ok(())
            }
            Type::Any => write!(f, "Any"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Range => write!(f, "Range"),
            Type::TypeVar(t) => write!(f, "<{t}>"),
        }
    }
}
