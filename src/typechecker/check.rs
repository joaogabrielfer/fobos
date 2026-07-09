use crate::{
    ast::{BinaryOp, Block, Expr, ExprKind, Program, Stmt, TypeAnnotation, UnaryOp},
    errors::{TypeError, TypeErrorKind},
    interpreter::values::Value,
    source::Span,
    typechecker::{
        CheckedProgram, TypeChecker, TypeResult,
        ty::{
            ParameterTypes,
            Type::{self},
        },
    },
};

#[derive(Debug, Clone)]
pub struct StmtCheck {
    pub normal_type: Type,
    pub yielded_type: Option<Type>,
    pub returned_type: Option<Type>,
}

impl StmtCheck {
    pub fn normal(ty: Type) -> Self {
        Self {
            normal_type: ty,
            yielded_type: None,
            returned_type: None,
        }
    }

    pub fn yields(ty: Type) -> Self {
        Self {
            normal_type: Type::Unit,
            yielded_type: Some(ty),
            returned_type: None,
        }
    }

    pub fn returns(ty: Type) -> Self {
        Self {
            normal_type: Type::Unit,
            yielded_type: None,
            returned_type: Some(ty),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockCheck {
    pub ty: Type,
    pub returned_type: Option<Type>,
}

impl TypeChecker {
    pub fn check_program(&mut self, program: Program) -> TypeResult<CheckedProgram> {
        self.env.load_builtins();

        for stmt in &program.statements {
            self.declare_function(stmt)?;
        }

        for stmt in &program.statements {
            match stmt {
                Stmt::FunDecl { .. } => {
                    self.check_function_body(stmt)?;
                }

                _ => {
                    let check = self.check_stmt(stmt)?;

                    if check.returned_type.is_some() {
                        return Err(
                            self.error_at(stmt.span(), TypeErrorKind::ReturnOutsideFunction)
                        );
                    }

                    if check.yielded_type.is_some() {
                        return Err(self.error_at(stmt.span(), TypeErrorKind::YieldOutsideHandler));
                    }
                }
            }
        }

        Ok(CheckedProgram { program })
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> TypeResult<StmtCheck> {
        match stmt {
            Stmt::Expr(expr) => Ok(StmtCheck::normal(self.infer_expr(expr)?)),
            Stmt::Yield(expr) => Ok(StmtCheck::yields(self.infer_expr(expr)?)),
            Stmt::Return(expr) => {
                let found = self.infer_expr(expr)?;
                let expected = self.current_function_return.clone().ok_or_else(|| {
                    self.error_at(stmt.span(), TypeErrorKind::ReturnOutsideFunction)
                })?;

                if TypeChecker::types_compatible(&expected, &found) {
                    Ok(StmtCheck::returns(found))
                } else {
                    Err(self.error_at(
                        stmt.span(),
                        TypeErrorKind::MismatchedType {
                            expected: expected.to_string(),
                            found: found.to_string(),
                        },
                    ))
                }
            }
            Stmt::Bind {
                name,
                type_annotation,
                value,
                ..
            } => {
                let value_type = self.infer_expr(value)?;

                match type_annotation {
                    TypeAnnotation::Inferred => {
                        self.env.define(name.clone(), value_type);
                        Ok(StmtCheck::normal(Type::Unit))
                    }
                    TypeAnnotation::Explicit(bind_type) => {
                        let bind_type = bind_type.resolve_type_expr();
                        if TypeChecker::types_compatible(&bind_type, &value_type) {
                            self.env.define(name.clone(), bind_type);
                            Ok(StmtCheck::normal(Type::Unit))
                        } else {
                            Err(self.error_at(
                                stmt.span(),
                                TypeErrorKind::MismatchedType {
                                    expected: bind_type.to_string(),
                                    found: value_type.to_string(),
                                },
                            ))
                        }
                    }
                }
            }
            Stmt::Assignment { target, value } => {
                let value_type = self.infer_expr(value)?;

                match &target.kind {
                    ExprKind::Ident(name) => {
                        let target_type = self
                            .env
                            .get(name)
                            .map_err(|kind| self.error_at(target.span, kind))?;

                        if !TypeChecker::types_compatible(&target_type, &value_type) {
                            return Err(self.error_at(
                                value.span,
                                TypeErrorKind::MismatchedType {
                                    expected: target_type.to_string(),
                                    found: value_type.to_string(),
                                },
                            ));
                        }

                        Ok(StmtCheck::normal(Type::Unit))
                    }

                    ExprKind::Index { target, index } => {
                        // Optional if you support arr[i] = value already.
                        self.check_index_assignment(target, index)?;
                        Ok(StmtCheck::normal(Type::Unit))
                    }

                    _ => Err(self.error_at(
                        target.span,
                        TypeErrorKind::InvalidAssignmentTarget(target.kind.to_string()),
                    )),
                }
            }
            Stmt::FunDecl { .. } => Ok(StmtCheck::normal(Type::Unit)),
        }
    }
    fn infer_expr(&mut self, expr: &Expr) -> TypeResult<Type> {
        let span = expr.span;
        match &expr.kind {
            ExprKind::Int(_) => Ok(Type::Int),
            ExprKind::Float(_) => Ok(Type::Float),
            ExprKind::String(_) => Ok(Type::String),
            ExprKind::Bool(_) => Ok(Type::Bool),
            ExprKind::Ident(value) => {
                let value = self
                    .env
                    .get(value)
                    .map_err(|kind| self.error_at(span, kind))?;
                Ok(value)
            }
            ExprKind::Unit => Ok(Type::Unit),
            ExprKind::Block(block) => self.check_block(block).map(|b| b.ty),
            ExprKind::Tuple(exprs) => {
                let mut tys = vec![];
                for expr in exprs {
                    tys.push(self.infer_expr(expr)?);
                }
                Ok(Type::Tuple(tys))
            }
            ExprKind::Array(exprs) => {
                let Some(fst) = exprs.first() else {
                    return Ok(Type::Array(Box::new(Type::Any)));
                };
                let t = self.infer_expr(fst)?;
                for expr in exprs {
                    let span = expr.span;
                    let current_t = self.infer_expr(expr)?;
                    if current_t != t {
                        return Err(self.error_at(
                            span,
                            TypeErrorKind::MismatchedArrayType {
                                expected: t.to_string(),
                                found: current_t.to_string(),
                            },
                        ));
                    }
                }
                Ok(Type::Array(Box::new(t)))
            }
            ExprKind::Unary { op, operand } => match op {
                UnaryOp::Negate => self.infer_expr(operand),
                UnaryOp::Not => self.infer_expr(operand),
            },
            ExprKind::Binary { lhs, op, rhs } => {
                let (lhs_span, rhs_span) = (lhs.span, rhs.span);
                let (lhs, rhs) = (self.infer_expr(lhs)?, self.infer_expr(rhs)?);
                match (op, lhs, rhs) {
                    (
                        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div,
                        Type::Int,
                        Type::Int,
                    ) => Ok(Type::Int),
                    (
                        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div,
                        Type::Float,
                        Type::Float,
                    ) => Ok(Type::Float),
                    (
                        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div,
                        Type::Float | Type::Int | Type::Any,
                        Type::Any,
                    ) => Ok(Type::Any),
                    (
                        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div,
                        Type::Any,
                        Type::Float | Type::Int,
                    ) => Ok(Type::Any),
                    (BinaryOp::Eq | BinaryOp::NotEq, t1, t2) if t1 == t2 => Ok(Type::Bool),
                    (BinaryOp::Eq | BinaryOp::NotEq, _, Type::Any) => Ok(Type::Bool),
                    (BinaryOp::Eq | BinaryOp::NotEq, Type::Any, _) => Ok(Type::Bool),
                    (
                        BinaryOp::Greater | BinaryOp::GreaterEq | BinaryOp::Less | BinaryOp::LessEq,
                        Type::Int,
                        Type::Int,
                    ) => Ok(Type::Bool),
                    (
                        BinaryOp::Greater | BinaryOp::GreaterEq | BinaryOp::Less | BinaryOp::LessEq,
                        Type::Float,
                        Type::Float,
                    ) => Ok(Type::Bool),
                    (
                        BinaryOp::Greater | BinaryOp::GreaterEq | BinaryOp::Less | BinaryOp::LessEq,
                        Type::Float | Type::Int | Type::Any,
                        Type::Any,
                    ) => Ok(Type::Bool),
                    (
                        BinaryOp::Greater | BinaryOp::GreaterEq | BinaryOp::Less | BinaryOp::LessEq,
                        Type::Any,
                        Type::Float | Type::Int,
                    ) => Ok(Type::Bool),
                    (BinaryOp::Combine, Type::String, Type::String) => Ok(Type::String),
                    (BinaryOp::Combine, Type::Any, Type::String | Type::Any) => Ok(Type::String),
                    (BinaryOp::Combine, Type::String, Type::Any) => Ok(Type::String),
                    (
                        BinaryOp::InclusiveRange | BinaryOp::ExclusiveRange,
                        Type::Int | Type::Any,
                        Type::Int | Type::Any,
                    ) => Ok(Type::Range),
                    (other_op, other_lhs, other_rhs) => Err(self.error_at(
                        Span {
                            start: lhs_span.start,
                            end: rhs_span.end,
                        },
                        TypeErrorKind::MismatchedBinaryOpType {
                            op: *other_op,
                            lhs: other_lhs.to_string(),
                            rhs: other_rhs.to_string(),
                        },
                    )),
                }
            }
            ExprKind::Call { callee, args: _ } => self.infer_expr(callee),
            ExprKind::If {
                condition: _,
                then_branch,
                else_branch,
            } => {
                let then_span = then_branch.span;
                let then_type = self.infer_expr(then_branch)?;
                match else_branch {
                    Some(eb) => {
                        let else_span = eb.span;
                        let else_type = self.infer_expr(eb)?;
                        if else_type != then_type {
                            Err(self.error_at(
                                Span {
                                    start: then_span.start,
                                    end: else_span.end,
                                },
                                TypeErrorKind::MismatchedBranchTypes {
                                    expected: then_type.to_string(),
                                    found: else_type.to_string(),
                                },
                            ))
                        } else {
                            Ok(then_type)
                        }
                    }
                    None => Ok(then_type),
                }
            }
            ExprKind::While { condition, block } => {
                let condition_span = condition.span;
                let t = self.infer_expr(condition)?;
                if !TypeChecker::types_compatible(&Type::Bool, &t) {
                    return Err(self.error_at(
                        condition_span,
                        TypeErrorKind::MismatchedType {
                            expected: "Bool".to_string(),
                            found: t.to_string(),
                        },
                    ));
                }

                self.check_block(block).map(|b| b.ty)
            }
            ExprKind::For {
                binding,
                iterable,
                block,
            } => {
                let iterable_span = iterable.span;
                let binding_span = binding.span;
                let iterable = self.infer_expr(iterable)?;

                let binding_name = match binding.kind.clone() {
                    ExprKind::Ident(i) => i,
                    other => {
                        return Err(self.error_at(
                            binding_span,
                            TypeErrorKind::InvalidAssignmentTarget(other.to_string()),
                        ));
                    }
                };

                let binding_type = match iterable {
                    Type::Range => Type::Int,

                    Type::Array(inner) => *inner,

                    other => {
                        return Err(self.error_at(
                            iterable_span,
                            TypeErrorKind::NotIterable {
                                found: other.to_string(),
                            },
                        ));
                    }
                };

                self.env.push_scope();
                self.env.define(binding_name, binding_type);

                let block_check = self.check_block(block)?;

                self.env.pop_scope();

                Ok(block_check.ty)
            }
            ExprKind::Lambda { parameters, body } => {
                self.env.push_scope();

                let previous_return = self.current_function_return.clone();
                self.current_function_return = Some(Type::Any);

                let result = (|| {
                    let mut parameters_types = vec![];

                    for parameter in parameters {
                        parameters_types.push(Type::Any);
                        self.env.define(parameter.name.clone(), Type::Any);
                    }

                    let ret_type = match &body.kind {
                        ExprKind::Block(block) => {
                            let block_check = self.check_block(block)?;

                            match block_check.returned_type {
                                Some(returned_type) => returned_type,
                                None => block_check.ty,
                            }
                        }

                        _ => self.infer_expr(body)?,
                    };

                    Ok(Type::Function {
                        parameter_overloads: vec![parameters_types],
                        return_type: Box::new(ret_type),
                    })
                })();

                self.current_function_return = previous_return;
                self.env.pop_scope();

                result
            }
            ExprKind::Index { target, index } => self.check_index_assignment(target, index),
        }
    }

    fn check_block(&mut self, block: &Block) -> TypeResult<BlockCheck> {
        self.env.push_scope();

        let mut yielded_type: Option<Type> = None;
        let mut returned_type: Option<Type> = None;

        for stmt in &block.statements {
            let stmt_check = self.check_stmt(stmt)?;

            if let Some(stmt_yielded_type) = stmt_check.yielded_type {
                match &yielded_type {
                    Some(existing) if existing != &stmt_yielded_type => {
                        return Err(self.error_at(
                            Span::dummy(), // TODO: add span to statements
                            TypeErrorKind::MismatchedYieldTypes {
                                expected: existing.clone().to_string(),
                                found: stmt_yielded_type.to_string(),
                            },
                        ));
                    }

                    Some(_) => {}

                    None => {
                        yielded_type = Some(stmt_yielded_type);
                    }
                }
            }

            if let Some(stmt_returned_type) = stmt_check.returned_type {
                match &returned_type {
                    Some(existing) if existing != &stmt_returned_type => {
                        return Err(self.error_at(
                            Span::dummy(), // TODO: add span to statements
                            TypeErrorKind::MismatchedReturnTypes {
                                expected: existing.clone().to_string(),
                                found: stmt_returned_type.to_string(),
                            },
                        ));
                    }

                    Some(_) => {}

                    None => {
                        returned_type = Some(stmt_returned_type);
                    }
                }
            }
        }

        self.env.pop_scope();

        Ok(BlockCheck {
            ty: yielded_type.unwrap_or(Type::Unit),
            returned_type,
        })
    }

    fn check_index_assignment(&mut self, target: &Expr, index: &Expr) -> TypeResult<Type> {
        let target_span = target.span;
        let target = self.infer_expr(target)?;
        match target {
            Type::Array(_) => {}
            other => {
                return Err(self.error_at(
                    target_span,
                    TypeErrorKind::InvalidIndexingTarget(other.to_string()),
                ));
            }
        }

        let index_span = index.span;
        let index = self.infer_expr(index)?;
        match index {
            Type::Int => Ok(Type::Int),
            other => Err(self.error_at(
                index_span,
                TypeErrorKind::InvalidIndexType(other.to_string()),
            )),
        }
    }

    fn declare_function(&mut self, stmt: &Stmt) -> TypeResult<()> {
        let Stmt::FunDecl {
            name,
            parameters,
            return_type,
            ..
        } = stmt
        else {
            return Ok(());
        };

        let mut parameters_types = Vec::new();

        for param in parameters {
            parameters_types.push(param.t.resolve_type_annotation());
        }

        let return_type = Box::new(return_type.resolve_type_annotation());

        let fun = if let Ok(Type::Function {
            mut parameter_overloads,
            return_type: defined_return_type,
        }) = self.env.get(name)
        {
            if defined_return_type != return_type {
                return Err(self.error_at(
                    stmt.span(),
                    TypeErrorKind::MismatchedReturnTypes {
                        expected: defined_return_type.to_string(),
                        found: return_type.to_string(),
                    },
                ));
            }

            parameter_overloads.push(parameters_types);
            Type::Function {
                parameter_overloads,
                return_type,
            }
        } else {
            Type::Function {
                parameter_overloads: vec![parameters_types],
                return_type,
            }
        };

        self.env.define(name.clone(), fun);

        Ok(())
    }

    fn check_function_body(&mut self, stmt: &Stmt) -> TypeResult<()> {
        let Stmt::FunDecl {
            parameters,
            return_type,
            body,
            ..
        } = stmt
        else {
            return Ok(());
        };

        let expected_return = return_type.resolve_type_annotation();
        let previous_return = self.current_function_return.clone();
        self.current_function_return = Some(expected_return.clone());

        self.env.push_scope();

        for param in parameters {
            let param_type = param.t.resolve_type_annotation();
            self.env.define(param.name.clone(), param_type);
        }

        let body_check = self.check_block(body)?;

        self.env.pop_scope();

        self.current_function_return = previous_return;

        if let Some(returned) = body_check.returned_type
            && !TypeChecker::types_compatible(
                &expected_return,
                &return_type.resolve_type_annotation(),
            )
        {
            return Err(self.error_at(
                body.span(),
                TypeErrorKind::MismatchedType {
                    expected: expected_return.to_string(),
                    found: returned.to_string(),
                },
            ));
        }

        Ok(())
    }

    // returns the matched index, not parameter length
    pub fn check_function_call(
        parameter_overloads: &[ParameterTypes],
        args_values: &Vec<Value>,
    ) -> Option<usize> {
        for (i, parameter_list) in parameter_overloads.iter().enumerate() {
            let mut matched = true;
            for (parameter, value) in parameter_list.iter().zip(args_values) {
                if !TypeChecker::types_compatible(parameter, &value.get_type()) {
                    matched = false
                }
            }
            if matched {
                return Some(i);
            }
        }

        None
    }

    fn types_compatible(expected: &Type, found: &Type) -> bool {
        match (expected, found) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Array(inner), _) | (_, Type::Array(inner)) => {
                if let Type::Any = **inner {
                    true
                } else {
                    *expected == *found
                }
            }
            _ => *expected == *found,
        }
    }

    fn error_at(&self, span: Span, kind: TypeErrorKind) -> Box<TypeError> {
        Box::new(TypeError {
            kind,
            span,
            file_path: self.file_path.clone(),
        })
    }
}
