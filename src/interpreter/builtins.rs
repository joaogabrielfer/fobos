use crate::{
    errors::{RuntimeError, RuntimeErrorKind},
    interpreter::{
        Interpreter,
        env::Env,
        eval::EvalFlow,
        values::{self, RangeValue, Value},
    },
    typechecker::{env::TypeEnv, ty::Type},
};

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    Echo,
    Range,
}

impl Env {
    pub fn load_builtins(&mut self) {
        self.define(
            "echo".to_string(),
            false,
            values::Value::BuiltinFunction(BuiltinFunction::Echo),
        );
        self.define(
            "range".to_string(),
            false,
            values::Value::BuiltinFunction(BuiltinFunction::Range),
        );
    }
}

impl TypeEnv {
    pub fn load_builtins(&mut self) {
        self.define(
            "echo".to_string(),
            Type::Function {
                parameters_types: vec![Type::String],
                return_type: Box::new(Type::Unit),
            },
        );
        self.define(
            "range1".to_string(),
            Type::Function {
                parameters_types: vec![Type::Int],
                return_type: Box::new(Type::Range),
            },
        );
        self.define(
            "range2".to_string(),
            Type::Function {
                parameters_types: vec![Type::Int, Type::Int],
                return_type: Box::new(Type::Range),
            },
        );
        self.define(
            "range3".to_string(),
            Type::Function {
                parameters_types: vec![Type::Int, Type::Int, Type::Int],
                return_type: Box::new(Type::Range),
            },
        );
    }
}
impl<'a, W: std::io::Write> Interpreter<'a, W> {
    pub fn call_builtin(
        &mut self,
        builtin: BuiltinFunction,
        args_values: Vec<Value>,
        span: crate::source::Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        match builtin {
            BuiltinFunction::Echo => {
                if args_values.len() != 1 {
                    Err(self.error_at(
                        span,
                        RuntimeErrorKind::ArityMismatch {
                            expected: 1,
                            found: args_values.len(),
                        },
                    ))
                } else {
                    self.writeln_output(format!("{}", args_values[0]))?;
                    Ok(EvalFlow::Continue(Value::Unit))
                }
            }
            BuiltinFunction::Range => match args_values.len() {
                1 => {
                    let Value::Int(end) = args_values[0] else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidRangeParameter(args_values[0].to_string()),
                        ));
                    };

                    Ok(EvalFlow::Continue(Value::Range(RangeValue {
                        start: 0,
                        end,
                        inclusive: false,
                        step: 1,
                    })))
                }
                2 => {
                    let Value::Int(start) = args_values[0] else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidRangeParameter(args_values[0].to_string()),
                        ));
                    };

                    let Value::Int(end) = args_values[1] else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidRangeParameter(args_values[0].to_string()),
                        ));
                    };

                    Ok(EvalFlow::Continue(Value::Range(RangeValue {
                        start,
                        end,
                        inclusive: false,
                        step: 1,
                    })))
                }
                3 => {
                    let Value::Int(start) = args_values[0] else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidRangeParameter(args_values[0].to_string()),
                        ));
                    };

                    let Value::Int(end) = args_values[1] else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidRangeParameter(args_values[0].to_string()),
                        ));
                    };

                    let Value::Int(step) = args_values[2] else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidRangeParameter(args_values[0].to_string()),
                        ));
                    };

                    if step == 0 {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::BadRangeStep {
                                found: step.to_string(),
                            },
                        ));
                    }

                    Ok(EvalFlow::Continue(Value::Range(RangeValue {
                        start,
                        end,
                        inclusive: false,
                        step,
                    })))
                }
                other => Err(self.error_at(
                    span,
                    RuntimeErrorKind::ArityMismatch {
                        expected: 1,
                        found: other,
                    },
                )),
            },
        }
    }
}
