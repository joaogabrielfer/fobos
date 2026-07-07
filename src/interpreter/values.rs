#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Tuple(Vec<Value>),

    BuiltinFunction(BuiltinFunction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    Echo,
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
            Value::BuiltinFunction(_) => write!(f, "<builtin function>"),
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
                s
            }
            Value::BuiltinFunction(_) => "<builtin function>".to_string(),
        }
    }
}
