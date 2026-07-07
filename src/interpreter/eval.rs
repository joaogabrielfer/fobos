use anyhow::Result;

use crate::{
    ast::{
        BinaryOp, Block, Expr, ExprKind, Program,
        Stmt::{self},
    },
    errors::{RuntimeError, RuntimeErrorKind},
    interpreter::{
        Interpreter,
        values::{BuiltinFunction, Value},
    },
    source::Span,
};

pub enum EvalFlow {
    Continue(Value),
    Return(Value),
    Yield(Value),
}

#[derive(Debug, Clone, Copy)]
enum YieldMode {
    Bubble,
    Capture,
}

macro_rules! value_or_flow {
    ($flow:expr) => {
        match $flow {
            EvalFlow::Continue(value) => value,
            EvalFlow::Return(value) => return Ok(EvalFlow::Return(value)),
            EvalFlow::Yield(value) => return Ok(EvalFlow::Yield(value)),
        }
    };
}

impl<'a> Interpreter<'a> {
    pub fn eval_program(&mut self, program: &'a Program) -> Result<Value, Box<RuntimeError>> {
        for stmt in &program.statements {
            match self.eval_statement(stmt)? {
                EvalFlow::Continue(_value) => {}
                EvalFlow::Yield(_value) => {}
                EvalFlow::Return(value) => {
                    self.env.pop_scope();
                    return Ok(value);
                }
            }
        }
        Ok(Value::Unit)
    }

    fn eval_statement(&mut self, statement: &'a Stmt) -> Result<EvalFlow, Box<RuntimeError>> {
        match statement {
            Stmt::Expr(expr) => {
                let value = self.eval_expr(expr, YieldMode::Bubble)?;
                Ok(EvalFlow::Continue(value_or_flow!(value)))
            }
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr, YieldMode::Capture)?;
                Ok(EvalFlow::Return(value_or_flow!(value)))
            }
            Stmt::Yield(expr) => {
                let value = self.eval_expr(expr, YieldMode::Capture)?;
                Ok(EvalFlow::Yield(value_or_flow!(value)))
            }
            Stmt::Bind {
                mutable,
                name,
                value,
                ..
            } => {
                let value = value_or_flow!(self.eval_expr(value, YieldMode::Capture)?);
                self.env.define(name, *mutable, value);
                Ok(EvalFlow::Continue(Value::Unit))
            }
            Stmt::Assignment { target, value } => {
                let value_span = value.span;
                let value = self.eval_expr(value, YieldMode::Capture)?;
                if let ExprKind::Ident(name) = &target.kind {
                    self.env
                        .assign(name, value_or_flow!(value))
                        .map_err(|kind| {
                            self.error_at(
                                Span {
                                    start: target.span.start,
                                    end: value_span.end,
                                },
                                kind,
                            )
                        })?;
                    Ok(EvalFlow::Continue(Value::Unit))
                } else {
                    Err(self.error_at(
                        target.span,
                        RuntimeErrorKind::InvalidAssignmentTarget(target.kind.to_string()),
                    ))
                }
            }
            #[allow(unused_variables)]
            Stmt::FunDecl {
                name,
                generics,
                parameters,
                return_type,
                body,
            } => todo!(),
        }
    }

    fn eval_block(
        &mut self,
        block: &'a Block,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        self.env.push_scope();

        let result = self.eval_block_inner(block, yield_mode);

        self.env.pop_scope();

        result
    }

    fn eval_block_inner(
        &mut self,
        block: &'a Block,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let mut last_value = Value::Unit;

        for stmt in &block.statements {
            match self.eval_statement(stmt)? {
                EvalFlow::Continue(value) => {
                    last_value = value;
                }

                EvalFlow::Return(value) => {
                    return Ok(EvalFlow::Return(value));
                }

                EvalFlow::Yield(value) => {
                    return match yield_mode {
                        YieldMode::Capture => Ok(EvalFlow::Continue(value)),
                        YieldMode::Bubble => Ok(EvalFlow::Yield(value)),
                    };
                }
            }
        }

        Ok(EvalFlow::Continue(last_value))
    }

    fn eval_expr(
        &mut self,
        expr: &'a Expr,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        match &expr.kind {
            ExprKind::Int(val) => Ok(EvalFlow::Continue(Value::Int(*val))),
            ExprKind::Float(val) => Ok(EvalFlow::Continue(Value::Float(*val))),
            ExprKind::String(val) => Ok(EvalFlow::Continue(Value::String(val.clone()))),
            ExprKind::Bool(val) => Ok(EvalFlow::Continue(Value::Bool(*val))),
            ExprKind::Ident(i) => {
                let value = self.env.get(i).map_err(|kind| {
                    Box::new(RuntimeError {
                        kind,
                        span: expr.span,
                        file_path: self.file_path.clone(),
                    })
                })?;
                Ok(EvalFlow::Continue(value))
            }
            ExprKind::Unit => Ok(EvalFlow::Continue(Value::Unit)),
            ExprKind::Block(block) => self.eval_block(block, yield_mode),
            ExprKind::Tuple(exprs) => {
                let mut values = vec![];

                for expr in exprs {
                    let value = value_or_flow!(self.eval_expr(expr, YieldMode::Capture)?);
                    values.push(value);
                }

                Ok(EvalFlow::Continue(Value::Tuple(values)))
            }
            ExprKind::Unary { op, operand } => {
                let value = value_or_flow!(self.eval_expr(operand, YieldMode::Capture)?);
                match op {
                    crate::ast::UnaryOp::Negate => match value {
                        Value::Int(val) => Ok(EvalFlow::Continue(Value::Int(-val))),
                        Value::Float(val) => Ok(EvalFlow::Continue(Value::Float(-val))),
                        other => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidUnaryOp {
                                op: *op,
                                operand: other.type_name(),
                            },
                        )),
                    },
                    crate::ast::UnaryOp::Not => match value {
                        Value::Bool(val) => Ok(EvalFlow::Continue(Value::Bool(!val))),
                        other => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidUnaryOp {
                                op: *op,
                                operand: other.type_name(),
                            },
                        )),
                    },
                }
            }
            ExprKind::Binary { lhs, op, rhs } => {
                let lhs = value_or_flow!(self.eval_expr(lhs, YieldMode::Capture)?);
                let rhs = value_or_flow!(self.eval_expr(rhs, YieldMode::Capture)?);
                match op {
                    BinaryOp::Add => match (lhs, rhs) {
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 + i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 + f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::Sub => match (lhs, rhs) {
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 - i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 - f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::Mul => match (lhs, rhs) {
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 * i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 * f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::Div => match (lhs, rhs) {
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 / i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 / f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::Eq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Bool(a), Value::Bool(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Tuple(a), Value::Tuple(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::String(a), Value::String(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Unit, Value::Unit) => Ok(EvalFlow::Continue(Value::Bool(true))),
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::NotEq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Bool(a), Value::Bool(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Tuple(a), Value::Tuple(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::String(a), Value::String(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Unit, Value::Unit) => Ok(EvalFlow::Continue(Value::Bool(false))),
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::Greater => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a > b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a > b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::GreaterEq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a >= b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a >= b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::Less => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a < b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a < b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::LessEq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a <= b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a <= b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                    BinaryOp::Combine => match (lhs, rhs) {
                        (Value::String(s1), s2) => Ok(EvalFlow::Continue(Value::String(
                            format!("{s1}{s2}").to_string(),
                        ))),
                        (s1, Value::String(s2)) => Ok(EvalFlow::Continue(Value::String(
                            format!("{s1}{s2}").to_string(),
                        ))),
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.type_name(),
                                rhs: other_rhs.type_name(),
                            },
                        )),
                    },
                }
            }
            #[allow(unused_variables)]
            ExprKind::Call { callee, args } => {
                let callee_value = value_or_flow!(self.eval_expr(callee, YieldMode::Capture)?);

                let mut args_values = vec![];
                for arg in args {
                    let value = value_or_flow!(self.eval_expr(arg, YieldMode::Capture)?);
                    args_values.push(value);
                }

                match callee_value {
                    Value::BuiltinFunction(builtin) => {
                        self.call_builtin(builtin, args_values, expr.span)
                    }

                    other => Err(self.error_at(
                        callee.span,
                        RuntimeErrorKind::NotCallable(other.type_name().to_string()),
                    )),
                }
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.eval_if(condition, then_branch, else_branch, yield_mode),
            ExprKind::While { condition, block } => self.eval_while(condition, block, yield_mode),
            #[allow(unused_variables)]
            ExprKind::Lambda { params, body } => todo!(),
        }
    }

    fn eval_if(
        &mut self,
        condition: &'a Expr,
        then_branch: &'a Expr,
        else_branch: &'a Option<Box<Expr>>,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let condition_bool = value_or_flow!(self.eval_expr(condition, YieldMode::Capture)?);

        let condition_bool = match condition_bool {
            Value::Bool(b) => b,
            other => {
                return Err(self.error_at(
                    condition.span,
                    RuntimeErrorKind::ExpectedBool {
                        found: other.type_name(),
                    },
                ));
            }
        };

        let flow = if condition_bool {
            self.eval_expr(then_branch, yield_mode)?
        } else if let Some(else_branch) = else_branch {
            self.eval_expr(else_branch, yield_mode)?
        } else if let YieldMode::Capture = yield_mode {
            return Err(self.error_at(
                Span {
                    start: condition.span.start,
                    end: then_branch.span.end,
                },
                RuntimeErrorKind::ElseBranchMissing,
            ));
        } else {
            EvalFlow::Continue(Value::Unit)
        };

        match (yield_mode, flow) {
            (YieldMode::Capture, EvalFlow::Yield(value)) => Ok(EvalFlow::Continue(value)),
            (_, other) => Ok(other),
        }
    }

    fn eval_while(
        &mut self,
        condition: &'a Expr,
        block: &'a Block,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        loop {
            let condition_value = value_or_flow!(self.eval_expr(condition, YieldMode::Capture)?);

            let condition_bool = match condition_value {
                Value::Bool(b) => b,
                other => {
                    return Err(self.error_at(
                        condition.span,
                        RuntimeErrorKind::ExpectedBool {
                            found: other.type_name(),
                        },
                    ));
                }
            };

            if !condition_bool {
                break;
            }

            let flow = self.eval_block(block, YieldMode::Bubble)?;

            match flow {
                EvalFlow::Continue(_) => {}

                EvalFlow::Return(value) => {
                    return Ok(EvalFlow::Return(value));
                }

                EvalFlow::Yield(value) => {
                    return match yield_mode {
                        YieldMode::Capture => Ok(EvalFlow::Continue(value)),
                        YieldMode::Bubble => Ok(EvalFlow::Yield(value)),
                    };
                }
            }
        }

        Ok(EvalFlow::Continue(Value::Unit))
    }

    fn error_at(&self, span: Span, kind: RuntimeErrorKind) -> Box<RuntimeError> {
        Box::new(RuntimeError {
            kind,
            span,
            file_path: self.file_path.clone(),
        })
    }

    fn call_builtin(
        &self,
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
                    println!("{}", args_values[0]);
                    Ok(EvalFlow::Continue(Value::Unit))
                }
            }
        }
    }
}
