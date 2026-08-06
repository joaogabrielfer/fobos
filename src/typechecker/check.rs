use crate::{
    ast::{
        BinaryOp, Block, CallArgument, ConstDecl, Expr, ExprKind, FunctionDecl, Item, Program,
        Stmt, TypeAnnotation, UnaryOp,
    },
    errors::{ArgumentError, ArgumentErrorKind, TypeError, TypeErrorKind},
    interpreter::values::{RangeValue, Value, normalize_arguments},
    module::{ExportedSymbol, ExportedSymbolKind, ModuleInterface, ResolvedImport},
    source::Span,
    typechecker::{
        CheckedModule, CheckedProgram, TypeChecker, TypeResult,
        env::TypeBinding,
        ty::{
            ParameterType,
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
        Ok(self
            .check_module(program, &[], &std::collections::HashMap::new())?
            .program)
    }

    pub fn check_module(
        &mut self,
        program: Program,
        imports: &[ResolvedImport],
        module_interfaces: &std::collections::HashMap<crate::module::ModuleId, ModuleInterface>,
    ) -> TypeResult<CheckedModule> {
        self.env.load_builtins();
        self.module_interfaces = module_interfaces.clone();
        self.install_imports(imports)?;

        for item in &program.items {
            match item {
                Item::Function(decl) => self.declare_function(decl)?,
                Item::Const(decl) => self.declare_const(decl)?,
                Item::Import(_) => {}
            }
        }

        for item in &program.items {
            match item {
                Item::Const(decl) => self.check_const_decl(decl)?,
                Item::Function(decl) => self.check_function_body(decl)?,
                Item::Import(_) => {}
            }
        }

        let constants = self.evaluate_constants(&program)?;

        let mut exports = std::collections::HashMap::new();
        for item in &program.items {
            match item {
                Item::Function(decl) if decl.public => {
                    let ty = self
                        .env
                        .get(&decl.name)
                        .map_err(|kind| self.error_at(decl.span, kind))?;
                    exports.insert(
                        decl.name.clone(),
                        ExportedSymbol {
                            ty,
                            kind: ExportedSymbolKind::Function,
                        },
                    );
                }
                Item::Const(decl) if decl.public => {
                    let ty = decl.type_annotation.resolve_type_expr();
                    let value = constants
                        .get(&decl.name)
                        .expect("every checked constant is evaluated")
                        .clone();
                    exports.insert(
                        decl.name.clone(),
                        ExportedSymbol {
                            ty,
                            kind: ExportedSymbolKind::Const { value },
                        },
                    );
                }
                _ => {}
            }
        }

        Ok(CheckedModule {
            program: CheckedProgram { program },
            interface: ModuleInterface { exports },
            constants,
        })
    }

    fn install_imports(&mut self, imports: &[ResolvedImport]) -> TypeResult<()> {
        for import in imports {
            let name = import.local_name();
            if self.env.get_current(name).is_some() {
                return Err(self.error_at(
                    import.span(),
                    TypeErrorKind::NameCollision(name.to_string()),
                ));
            }
            match import {
                ResolvedImport::Module { module, .. } => {
                    self.env
                        .define_binding(name.to_string(), TypeBinding::Module(module.clone()));
                }
                ResolvedImport::Member {
                    module,
                    export_name,
                    ..
                } => {
                    let interface = self
                        .module_interfaces
                        .get(module)
                        .expect("resolved imports must have a dependency interface");
                    let export = interface.exports.get(export_name).ok_or_else(|| {
                        self.error_at(
                            import.span(),
                            TypeErrorKind::UnknownModuleExport {
                                module: module.to_string(),
                                member: export_name.clone(),
                            },
                        )
                    })?;
                    self.env.define_binding(
                        name.to_string(),
                        TypeBinding::ImportedMember {
                            module: module.clone(),
                            export_name: export_name.clone(),
                            ty: export.ty.clone(),
                            kind: export.kind.clone(),
                        },
                    );
                }
            }
        }
        Ok(())
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
                if self.env.at_module_scope() && self.env.get_current(name).is_some() {
                    return Err(
                        self.error_at(stmt.span(), TypeErrorKind::NameCollision(name.clone()))
                    );
                }
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

                    ExprKind::Path(segments) => Err(self.error_at(
                        target.span,
                        TypeErrorKind::ModuleMemberAssignment(segments.join("::")),
                    )),

                    _ => Err(self.error_at(
                        target.span,
                        TypeErrorKind::InvalidAssignmentTarget(target.kind.to_string()),
                    )),
                }
            }
            Stmt::Function(decl) => {
                self.declare_function(decl)?;
                self.check_function_body(decl)?;
                Ok(StmtCheck::normal(Type::Unit))
            }
        }
    }

    fn declare_const(&mut self, decl: &ConstDecl) -> TypeResult<()> {
        if self.env.get_current(&decl.name).is_some() {
            return Err(self.error_at(decl.span, TypeErrorKind::NameCollision(decl.name.clone())));
        }
        self.env
            .define_const(decl.name.clone(), decl.type_annotation.resolve_type_expr());
        Ok(())
    }

    fn check_const_decl(&mut self, decl: &ConstDecl) -> TypeResult<()> {
        self.validate_const_expr(&decl.value)?;
        let found = self.infer_expr(&decl.value)?;
        let expected = decl.type_annotation.resolve_type_expr();
        if TypeChecker::types_compatible(&expected, &found) {
            Ok(())
        } else {
            Err(self.error_at(
                decl.span,
                TypeErrorKind::MismatchedType {
                    expected: expected.to_string(),
                    found: found.to_string(),
                },
            ))
        }
    }

    fn validate_const_expr(&self, expr: &Expr) -> TypeResult<()> {
        match &expr.kind {
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit => Ok(()),
            ExprKind::Ident(name) => match self
                .env
                .get_binding(name)
                .map_err(|kind| self.error_at(expr.span, kind))?
            {
                TypeBinding::Const(_) => Ok(()),
                TypeBinding::ImportedMember {
                    kind: ExportedSymbolKind::Const { .. },
                    ..
                } => Ok(()),
                _ => Err(self.error_at(expr.span, TypeErrorKind::NotAConstant(name.clone()))),
            },
            ExprKind::Path(segments) => {
                let Some((module_name, member_path)) = segments.split_first() else {
                    unreachable!("paths always have at least two segments")
                };
                let TypeBinding::Module(module) = self
                    .env
                    .get_binding(module_name)
                    .map_err(|kind| self.error_at(expr.span, kind))?
                else {
                    return Err(
                        self.error_at(expr.span, TypeErrorKind::NotAConstant(segments.join("::")))
                    );
                };
                let [member] = member_path else {
                    return Err(
                        self.error_at(expr.span, TypeErrorKind::NotAConstant(segments.join("::")))
                    );
                };
                let interface = self
                    .module_interfaces
                    .get(&module)
                    .expect("module bindings have an interface");
                match interface.exports.get(member) {
                    Some(ExportedSymbol {
                        kind: ExportedSymbolKind::Const { .. },
                        ..
                    }) => Ok(()),
                    Some(_) => {
                        Err(self
                            .error_at(expr.span, TypeErrorKind::NotAConstant(segments.join("::"))))
                    }
                    None => Err(self.error_at(
                        expr.span,
                        TypeErrorKind::UnknownModuleExport {
                            module: module.to_string(),
                            member: member.clone(),
                        },
                    )),
                }
            }
            ExprKind::Tuple(items) | ExprKind::Array(items) => {
                for item in items {
                    self.validate_const_expr(item)?;
                }
                Ok(())
            }
            ExprKind::Unary { operand, .. } => self.validate_const_expr(operand),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.validate_const_expr(lhs)?;
                self.validate_const_expr(rhs)
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.validate_const_expr(condition)?;
                self.validate_const_expr(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.validate_const_expr(else_branch)?;
                }
                Ok(())
            }
            ExprKind::Index { target, index } => {
                self.validate_const_expr(target)?;
                self.validate_const_expr(index)
            }
            ExprKind::Block(_)
            | ExprKind::Call { .. }
            | ExprKind::While { .. }
            | ExprKind::For { .. }
            | ExprKind::Lambda { .. } => Err(self.error_at(
                expr.span,
                TypeErrorKind::ConstExpressionNotAllowed(expr.kind.to_string()),
            )),
        }
    }

    fn evaluate_constants(
        &self,
        program: &Program,
    ) -> TypeResult<std::collections::HashMap<String, Value>> {
        let names = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Const(decl) => Some(decl.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let declarations = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Const(decl) => Some((decl.name.clone(), decl)),
                _ => None,
            })
            .collect::<std::collections::HashMap<_, _>>();
        let mut values = std::collections::HashMap::new();
        let mut visiting = Vec::new();
        for name in names {
            self.evaluate_const_by_name(name, &declarations, &mut values, &mut visiting)?;
        }
        Ok(values)
    }

    fn evaluate_const_by_name(
        &self,
        name: &str,
        declarations: &std::collections::HashMap<String, &ConstDecl>,
        values: &mut std::collections::HashMap<String, Value>,
        visiting: &mut Vec<String>,
    ) -> TypeResult<Value> {
        if let Some(value) = values.get(name) {
            return Ok(value.clone());
        }
        if let Some(index) = visiting.iter().position(|item| item == name) {
            let mut cycle = visiting[index..].to_vec();
            cycle.push(name.to_string());
            let span = declarations
                .get(name)
                .expect("the cycle consists of local constants")
                .span;
            return Err(self.error_at(span, TypeErrorKind::ConstantCycle(cycle.join(" -> "))));
        }
        let decl = declarations
            .get(name)
            .expect("only declared constants are evaluated");
        visiting.push(name.to_string());
        let value = self.evaluate_const_expr(&decl.value, declarations, values, visiting);
        visiting.pop();
        let value = value?;
        values.insert(name.to_string(), value.clone());
        Ok(value)
    }

    fn evaluate_const_expr(
        &self,
        expr: &Expr,
        declarations: &std::collections::HashMap<String, &ConstDecl>,
        values: &mut std::collections::HashMap<String, Value>,
        visiting: &mut Vec<String>,
    ) -> TypeResult<Value> {
        let invalid =
            |message: String| self.error_at(expr.span, TypeErrorKind::ConstantEvaluation(message));
        match &expr.kind {
            ExprKind::Int(value) => Ok(Value::Int(*value)),
            ExprKind::Float(value) => Ok(Value::Float(*value)),
            ExprKind::String(value) => Ok(Value::String(value.clone())),
            ExprKind::Bool(value) => Ok(Value::Bool(*value)),
            ExprKind::Unit => Ok(Value::Unit),
            ExprKind::Ident(name) => {
                if declarations.contains_key(name) {
                    self.evaluate_const_by_name(name, declarations, values, visiting)
                } else {
                    match self
                        .env
                        .get_binding(name)
                        .map_err(|kind| self.error_at(expr.span, kind))?
                    {
                        TypeBinding::ImportedMember {
                            kind: ExportedSymbolKind::Const { value },
                            ..
                        } => Ok(value),
                        _ => {
                            Err(self.error_at(expr.span, TypeErrorKind::NotAConstant(name.clone())))
                        }
                    }
                }
            }
            ExprKind::Path(segments) => {
                let (module_name, member_path) =
                    segments.split_first().expect("paths are non-empty");
                let TypeBinding::Module(module) = self
                    .env
                    .get_binding(module_name)
                    .map_err(|kind| self.error_at(expr.span, kind))?
                else {
                    return Err(
                        self.error_at(expr.span, TypeErrorKind::NotAConstant(segments.join("::")))
                    );
                };
                let [member] = member_path else {
                    return Err(
                        self.error_at(expr.span, TypeErrorKind::NotAConstant(segments.join("::")))
                    );
                };
                let interface = self
                    .module_interfaces
                    .get(&module)
                    .expect("module bindings have an interface");
                match interface.exports.get(member) {
                    Some(ExportedSymbol {
                        kind: ExportedSymbolKind::Const { value },
                        ..
                    }) => Ok(value.clone()),
                    _ => {
                        Err(self
                            .error_at(expr.span, TypeErrorKind::NotAConstant(segments.join("::"))))
                    }
                }
            }
            ExprKind::Tuple(items) => items
                .iter()
                .map(|item| self.evaluate_const_expr(item, declarations, values, visiting))
                .collect::<TypeResult<Vec<_>>>()
                .map(Value::Tuple),
            ExprKind::Array(items) => items
                .iter()
                .map(|item| self.evaluate_const_expr(item, declarations, values, visiting))
                .collect::<TypeResult<Vec<_>>>()
                .map(Value::Array),
            ExprKind::Unary { op, operand } => {
                let value = self.evaluate_const_expr(operand, declarations, values, visiting)?;
                match (op, value) {
                    (UnaryOp::Negate, Value::Int(value)) => Ok(Value::Int(-value)),
                    (UnaryOp::Negate, Value::Float(value)) => Ok(Value::Float(-value)),
                    (UnaryOp::Not, Value::Bool(value)) => Ok(Value::Bool(!value)),
                    (_, value) => Err(invalid(format!(
                        "invalid unary operation on {}",
                        value.type_name()
                    ))),
                }
            }
            ExprKind::Binary { lhs, op, rhs } => {
                let lhs = self.evaluate_const_expr(lhs, declarations, values, visiting)?;
                let rhs = self.evaluate_const_expr(rhs, declarations, values, visiting)?;
                self.evaluate_const_binary(*op, lhs, rhs, expr.span)
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition =
                    self.evaluate_const_expr(condition, declarations, values, visiting)?;
                match condition {
                    Value::Bool(true) => {
                        self.evaluate_const_expr(then_branch, declarations, values, visiting)
                    }
                    Value::Bool(false) => match else_branch {
                        Some(branch) => {
                            self.evaluate_const_expr(branch, declarations, values, visiting)
                        }
                        None => Ok(Value::Unit),
                    },
                    value => Err(invalid(format!(
                        "if condition must be Bool, found {}",
                        value.type_name()
                    ))),
                }
            }
            ExprKind::Index { target, index } => {
                let target = self.evaluate_const_expr(target, declarations, values, visiting)?;
                let index = self.evaluate_const_expr(index, declarations, values, visiting)?;
                match (target, index) {
                    (Value::Array(items), Value::Int(index)) if index >= 0 => items
                        .get(index as usize)
                        .cloned()
                        .ok_or_else(|| invalid(format!("index {index} is out of bounds"))),
                    (Value::Array(_), Value::Int(index)) => {
                        Err(invalid(format!("index {index} is out of bounds")))
                    }
                    (Value::Array(_), value) => Err(invalid(format!(
                        "array index must be Int, found {}",
                        value.type_name()
                    ))),
                    (value, _) => Err(invalid(format!("{} is not indexable", value.type_name()))),
                }
            }
            _ => Err(self.error_at(
                expr.span,
                TypeErrorKind::ConstExpressionNotAllowed(expr.kind.to_string()),
            )),
        }
    }

    fn evaluate_const_binary(
        &self,
        op: BinaryOp,
        lhs: Value,
        rhs: Value,
        span: Span,
    ) -> TypeResult<Value> {
        let invalid =
            |message: String| self.error_at(span, TypeErrorKind::ConstantEvaluation(message));
        match op {
            BinaryOp::Add => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => a
                    .checked_add(b)
                    .map(Value::Int)
                    .ok_or_else(|| invalid("integer overflow".to_string())),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (a, b) => Err(invalid(format!(
                    "cannot add {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::Sub => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => a
                    .checked_sub(b)
                    .map(Value::Int)
                    .ok_or_else(|| invalid("integer overflow".to_string())),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (a, b) => Err(invalid(format!(
                    "cannot subtract {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::Mul => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => a
                    .checked_mul(b)
                    .map(Value::Int)
                    .ok_or_else(|| invalid("integer overflow".to_string())),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (a, b) => Err(invalid(format!(
                    "cannot multiply {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::Div => match (lhs, rhs) {
                (Value::Int(_), Value::Int(0)) => Err(invalid("division by zero".to_string())),
                (Value::Int(a), Value::Int(b)) => a
                    .checked_div(b)
                    .map(Value::Int)
                    .ok_or_else(|| invalid("integer overflow".to_string())),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (a, b) => Err(invalid(format!(
                    "cannot divide {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::Eq => Ok(Value::Bool(lhs == rhs)),
            BinaryOp::NotEq => Ok(Value::Bool(lhs != rhs)),
            BinaryOp::Greater => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                (a, b) => Err(invalid(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::GreaterEq => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                (a, b) => Err(invalid(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::Less => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                (a, b) => Err(invalid(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::LessEq => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                (a, b) => Err(invalid(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::Combine => Ok(Value::String(format!("{lhs}{rhs}"))),
            BinaryOp::InclusiveRange => match (lhs, rhs) {
                (Value::Int(start), Value::Int(end)) => Ok(Value::Range(RangeValue {
                    start,
                    end,
                    inclusive: true,
                    step: 1,
                })),
                (a, b) => Err(invalid(format!(
                    "cannot build a range from {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinaryOp::ExclusiveRange => match (lhs, rhs) {
                (Value::Int(start), Value::Int(end)) => Ok(Value::Range(RangeValue {
                    start,
                    end,
                    inclusive: false,
                    step: 1,
                })),
                (a, b) => Err(invalid(format!(
                    "cannot build a range from {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
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
            ExprKind::Path(segments) => self.infer_module_path(segments, span),
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
                    (BinaryOp::Combine, Type::String, _) => Ok(Type::String),
                    (BinaryOp::Combine, _, Type::String | Type::Any) => Ok(Type::String),
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
            ExprKind::Call { callee, args } => {
                let Type::Function {
                    parameter_overloads,
                    return_type,
                } = self.infer_expr(callee)?
                else {
                    panic!("should be a function call")
                };

                let argument_types = args
                    .iter()
                    .map(|argument| {
                        Ok(CallArgument {
                            name: argument.name.clone(),
                            value: self.infer_expr(&argument.value)?,
                            span: argument.span,
                        })
                    })
                    .collect::<TypeResult<Vec<_>>>()?;

                let mut matches = 0;
                let mut last_argument_error = None;
                let mut last_type_mismatch = None;
                for parameters in &parameter_overloads {
                    let normalized = match normalize_arguments(parameters, &argument_types) {
                        Ok(arguments) => arguments,
                        Err(error) => {
                            last_argument_error = Some(error);
                            continue;
                        }
                    };

                    let mismatch =
                        parameters
                            .iter()
                            .zip(normalized)
                            .find(|(parameter, argument)| {
                                !TypeChecker::types_compatible(&parameter.ty, argument)
                            });

                    if let Some((parameter, argument)) = mismatch {
                        last_type_mismatch = Some((parameter.ty.clone(), argument));
                    } else {
                        matches += 1;
                    }
                }

                if matches == 0 {
                    if let Some((expected, found)) = last_type_mismatch {
                        return Err(self.error_at(
                            callee.span,
                            TypeErrorKind::MismatchedType {
                                expected: expected.to_string(),
                                found: found.to_string(),
                            },
                        ));
                    }

                    if let Some(error) = last_argument_error {
                        let span = error.span.unwrap_or(callee.span);
                        return Err(self.error_at(span, TypeErrorKind::ArgumentError { e: error }));
                    }

                    return Err(self.error_at(
                        callee.span,
                        TypeErrorKind::MismatchedType {
                            expected: "a matching function signature".to_string(),
                            found: "the provided argument types".to_string(),
                        },
                    ));
                }

                if matches > 1 {
                    return Err(self.error_at(
                        callee.span,
                        TypeErrorKind::ArgumentError {
                            e: Box::new(ArgumentError {
                                kind: ArgumentErrorKind::Ambiguous,
                                span: None,
                            }),
                        },
                    ));
                }

                Ok(*return_type)
            }
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
                        parameters_types.push(ParameterType {
                            name: parameter.name.clone(),
                            ty: Type::Any,
                        });
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
        let array_t = match target {
            Type::Array(t) => *t,
            other => {
                return Err(self.error_at(
                    target_span,
                    TypeErrorKind::InvalidIndexingTarget(other.to_string()),
                ));
            }
        };

        let index_span = index.span;
        let index = self.infer_expr(index)?;
        match index {
            Type::Int => Ok(array_t),
            other => Err(self.error_at(
                index_span,
                TypeErrorKind::InvalidIndexType(other.to_string()),
            )),
        }
    }

    fn declare_function(&mut self, decl: &FunctionDecl) -> TypeResult<()> {
        let FunctionDecl {
            name,
            parameters,
            return_type,
            ..
        } = decl;

        let mut parameters_types = Vec::new();

        for param in parameters {
            parameters_types.push(ParameterType {
                name: param.name.clone(),
                ty: param.t.resolve_type_annotation(),
            });
        }

        let return_type = Box::new(return_type.resolve_type_annotation());

        let fun = match self.env.get_current(name) {
            Some(TypeBinding::Local(Type::Function {
                mut parameter_overloads,
                return_type: defined_return_type,
            })) => {
                if defined_return_type != return_type {
                    return Err(self.error_at(
                        decl.span,
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
            }
            Some(_) => {
                return Err(self.error_at(decl.span, TypeErrorKind::NameCollision(name.clone())));
            }
            None => Type::Function {
                parameter_overloads: vec![parameters_types],
                return_type,
            },
        };

        self.env.define(name.clone(), fun);

        Ok(())
    }

    fn infer_module_path(&self, segments: &[String], span: Span) -> TypeResult<Type> {
        let Some((module_name, member_path)) = segments.split_first() else {
            unreachable!("paths have at least two segments");
        };
        let TypeBinding::Module(module) = self
            .env
            .get_binding(module_name)
            .map_err(|kind| self.error_at(span, kind))?
        else {
            return Err(self.error_at(span, TypeErrorKind::NotAValue(module_name.clone())));
        };
        let [member] = member_path else {
            return Err(self.error_at(
                span,
                TypeErrorKind::UnknownModuleExport {
                    module: module.to_string(),
                    member: member_path.join("::"),
                },
            ));
        };
        let interface = self
            .module_interfaces
            .get(&module)
            .expect("module namespace bindings must refer to a dependency interface");
        interface
            .exports
            .get(member)
            .map(|symbol| symbol.ty.clone())
            .ok_or_else(|| {
                self.error_at(
                    span,
                    TypeErrorKind::UnknownModuleExport {
                        module: module.to_string(),
                        member: member.clone(),
                    },
                )
            })
    }

    fn check_function_body(&mut self, decl: &FunctionDecl) -> TypeResult<()> {
        let FunctionDecl {
            parameters,
            return_type,
            body,
            ..
        } = decl;

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
            && !TypeChecker::types_compatible(&expected_return, &returned)
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

    pub fn types_compatible(expected: &Type, found: &Type) -> bool {
        match (expected, found) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Array(i1), other) => {
                if let Type::Any = **i1 {
                    true
                } else if let Type::Array(i2) = other {
                    if let Type::Any = **i2 {
                        true
                    } else {
                        *expected == *found
                    }
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
