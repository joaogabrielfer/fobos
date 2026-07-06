use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Tuple(Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "Unit"),
            Value::Int(v) => write!(f, "Int '{v}'"),
            Value::Float(v) => write!(f, "Float '{v}'"),
            Value::Bool(v) => write!(f, "Bool '{v}'"),
            Value::String(v) => write!(f, "String '{v}'"),
            Value::Tuple(v) => {
                write!(f, "Tuple '(")?;
                for (i, item) in v.iter().enumerate() {
                    write!(f, "{item}")?;
                    if i < v.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                Ok(())
            }
        }
    }
}
