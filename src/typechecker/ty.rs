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
        parameter_overloads: Vec<ParameterTypes>,
        return_type: Box<Type>,
    },

    Any,
    Unknown,
    TypeVar(String),
}

pub type ParameterTypes = Vec<Type>;

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
            Type::Array(t) => write!(f, "Arr<{t}>"),
            Type::Function {
                parameter_overloads: overloaded_parameters,
                return_type,
            } => {
                write!(f, "(")?;
                for (i, parameters_types) in overloaded_parameters.iter().enumerate() {
                    for (j, param) in parameters_types.iter().enumerate() {
                        write!(f, "{param}")?;
                        if j < parameters_types.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }
                    if i < overloaded_parameters.len() - 1 {
                        write!(f, "| ")?;
                    }
                }
                write!(f, ") -> {return_type}")?;
                Ok(())
            }
            Type::Any => write!(f, "Any"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Range => write!(f, "Range"),
            Type::TypeVar(t) => write!(f, "TypeVar[{t}]"),
        }
    }
}
