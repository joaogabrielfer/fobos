use crate::{
    ast::{self, BinaryOp, Expr, ExprKind, Parameter, TypeAnnotation, TypeExpr},
    errors::{ParserError, ParserErrorKind},
    lexer::{
        Token, TokenKind,
        TokenTag::{self, RParen},
    },
    source::Span,
};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    file_path: &'a PathBuf,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>, file_path: &'a PathBuf) -> Self {
        Self {
            tokens,
            file_path,
            index: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<ast::Program, Box<ParserError>> {
        let mut statements = vec![];
        self.consume_newlines();

        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.expect(TokenTag::NewLine)?;
            self.consume_newlines();
        }
        Ok(ast::Program { statements })
    }

    fn parse_statement(&mut self) -> Result<ast::Stmt, Box<ParserError>> {
        match self.current().kind {
            TokenKind::Let => self.parse_binding(false),
            TokenKind::Var => self.parse_binding(true),
            TokenKind::Fun => self.parse_fun_decl(),
            TokenKind::Return => {
                self.advance();
                Ok(ast::Stmt::Return(self.parse_expr()?))
            }
            TokenKind::Yield => {
                self.advance();
                Ok(ast::Stmt::Yield(self.parse_expr()?))
            }
            _ => {
                let starting_span = self.current().span;
                let expr = self.parse_expr()?;
                if self.check(TokenTag::Equals) {
                    if !matches!(expr.kind, ast::ExprKind::Ident(_) | ast::ExprKind::Tuple(_)) {
                        return Err(Box::new(ParserError {
                            kind: ParserErrorKind::InvalidAssignmentTarget {
                                expr: expr.kind.to_string(),
                            },
                            file_path: self.file_path.clone(),
                            pos: starting_span,
                        }));
                    }
                    self.advance();
                    let value = self.parse_expr()?;
                    Ok(ast::Stmt::Assignment {
                        target: expr,
                        value,
                    })
                } else {
                    Ok(ast::Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_binding(&mut self, mutable: bool) -> Result<ast::Stmt, Box<ParserError>> {
        let start_span = self.current().span;
        if mutable {
            self.expect(TokenTag::Var)?;
        } else {
            self.expect(TokenTag::Let)?;
        }
        let name = self.expect_ident()?;
        self.expect(TokenTag::Colon)?;
        let type_annotation = self.parse_option_type()?;
        self.expect(TokenTag::Equals)?;
        let value = if self.check(TokenTag::NewLine) {
            let kind = ExprKind::Block(self.parse_block()?);
            self.new_expr(start_span, kind)
        } else {
            self.parse_expr()?
        };

        let value_span = value.span;
        Ok(ast::Stmt::Bind {
            mutable,
            name,
            type_annotation,
            value,
            span: Span {
                start: start_span.start,
                end: value_span.end,
            },
        })
    }

    fn parse_while(&mut self) -> Result<Expr, Box<ParserError>> {
        self.expect(TokenTag::While)?;
        let condition = Box::new(self.parse_expr()?);
        self.expect(TokenTag::Do)?;
        let block = self.parse_block()?;
        Ok(self.new_expr(self.current().span, ExprKind::While { condition, block }))
    }

    fn parse_for(&mut self) -> Result<Expr, Box<ParserError>> {
        self.expect(TokenTag::For)?;
        let binding = Box::new(self.parse_expr()?);
        self.expect(TokenTag::In)?;
        let iterable = Box::new(self.parse_expr()?);
        self.expect(TokenTag::Do)?;
        let block = self.parse_block()?;
        Ok(self.new_expr(
            self.current().span,
            ExprKind::For {
                binding,
                iterable,
                block,
            },
        ))
    }

    fn parse_fun_decl(&mut self) -> Result<ast::Stmt, Box<ParserError>> {
        let start_span = self.current().span;
        self.expect(TokenTag::Fun)?;
        let mut generics = vec![];
        if let TokenTag::LBrace = self.current_tag() {
            self.advance();
            while !self.check(TokenTag::RBrace) {
                let name = self.expect_ident()?;
                generics.push(name);
                if let TokenTag::Comma = self.current_tag() {
                    self.expect(TokenTag::Comma)?;
                }
            }
            self.expect(TokenTag::RBrace)?;
        }
        let name = self.expect_ident()?;
        self.expect(TokenTag::LParen)?;
        let mut parameters = vec![];
        while !self.check(TokenTag::NewLine) {
            if let TokenTag::RParen = self.current_tag() {
                self.advance();
                break;
            }
            let name = self.expect_ident()?;
            self.expect(TokenTag::Colon)?;
            let TypeAnnotation::Explicit(t) = self.parse_option_type()? else {
                return Err(self.error(ParserErrorKind::ExpectedTypeAnnotation));
            };
            parameters.push(ast::Parameter {
                name,
                t: TypeAnnotation::Explicit(t),
            });
            match self.current_tag() {
                TokenTag::Comma => {
                    self.expect(TokenTag::Comma)?;
                }
                TokenTag::RParen => {
                    self.expect(RParen)?;
                    break;
                }
                other => {
                    return Err(self.error(ParserErrorKind::ExpectedTokens {
                        expected: vec![")".to_string(), ",".to_string()],
                        found: other.to_string(),
                    }));
                }
            }
        }
        self.expect(TokenTag::Colon)?;
        let return_type = self.parse_option_type()?;
        let end_span = self.current().span;
        self.expect(TokenTag::Equals)?;
        let body = self.parse_block()?;
        Ok(ast::Stmt::FunDecl {
            name,
            generics,
            parameters,
            body,
            return_type,
            span: Span {
                start: start_span.start,
                end: end_span.end,
            },
        })
    }

    fn parse_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        self.parse_lambda()
    }

    fn parse_lambda(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        let expr = self.parse_binary_expr(0)?;
        if let TokenTag::RArrow = self.current_tag() {
            let mut parameters = vec![];
            match expr.kind {
                ExprKind::Ident(name) => parameters.push(Parameter {
                    name,
                    t: TypeAnnotation::Inferred,
                }),
                ExprKind::Tuple(t) => {
                    for e in t {
                        match e.kind {
                            ExprKind::Ident(name) => parameters.push(Parameter {
                                name,
                                t: TypeAnnotation::Inferred,
                            }),
                            other => {
                                return Err(self.error(ParserErrorKind::InvalidParameter {
                                    expr: other.to_string(),
                                }));
                            }
                        }
                    }
                }
                other => {
                    return Err(self.error(ParserErrorKind::InvalidParameter {
                        expr: other.to_string(),
                    }));
                }
            }
            self.expect(TokenTag::RArrow)?;
            let span = self.current().span;
            let body = if self.check(TokenTag::NewLine) {
                let block = self.parse_block()?;
                Box::new(self.new_expr(span, ExprKind::Block(block)))
            } else {
                Box::new(self.parse_expr()?)
            };
            Ok(self.new_expr(start_span, ExprKind::Lambda { parameters, body }))
        } else {
            Ok(expr)
        }
    }

    fn parse_binary_expr(&mut self, min_level: u8) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        let mut lhs = self.parse_unary_expr()?;

        while let Some((op, level)) = self.current_tag().precedence_level()
            && level >= min_level
        {
            self.advance();
            let rhs = self.parse_binary_expr(level + 1)?;
            if matches!(op, BinaryOp::InclusiveRange | BinaryOp::ExclusiveRange)
                && matches!(
                    rhs.kind,
                    ExprKind::Binary {
                        op: BinaryOp::InclusiveRange | BinaryOp::ExclusiveRange,
                        ..
                    }
                )
            {
                return Err(self.error(ParserErrorKind::ChainingRanges));
            }
            lhs = self.new_expr(
                start_span,
                ExprKind::Binary {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                },
            );
        }

        Ok(lhs)
    }

    fn parse_block(&mut self) -> Result<ast::Block, Box<ParserError>> {
        let mut statements = vec![];

        if !self.check(TokenTag::NewLine) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            return Ok(ast::Block { statements });
        }

        self.consume_newlines();
        while !self.check(TokenTag::End) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.expect_many(vec![TokenTag::NewLine])?;
            self.consume_newlines();
        }

        self.advance();

        Ok(ast::Block { statements })
    }

    fn parse_unary_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        if self.check(TokenTag::Bang) {
            self.advance();

            let expr = self.parse_unary_expr()?;
            return Ok(self.new_expr(
                start_span,
                ExprKind::Unary {
                    op: ast::UnaryOp::Not,
                    operand: Box::new(expr),
                },
            ));
        }

        if self.check(TokenTag::Minus) {
            self.advance();

            let expr = self.parse_unary_expr()?;
            return Ok(self.new_expr(
                start_span,
                ExprKind::Unary {
                    op: ast::UnaryOp::Negate,
                    operand: Box::new(expr),
                },
            ));
        }

        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        let mut expr = self.parse_primary_expr()?;

        loop {
            match self.current_tag() {
                TokenTag::LBrace => {
                    self.expect(TokenTag::LBrace)?;
                    let index = Box::new(self.parse_expr()?);
                    self.expect(TokenTag::RBrace)?;

                    expr = self.new_expr(
                        start_span,
                        ExprKind::Index {
                            target: Box::new(expr),
                            index,
                        },
                    );

                    continue;
                }
                TokenTag::LParen => {
                    let args = self.parse_call_args()?;

                    expr = self.new_expr(
                        start_span,
                        ExprKind::Call {
                            callee: Box::new(expr),
                            args,
                        },
                    );

                    continue;
                }
                TokenTag::Dot => {
                    self.advance();

                    let method_name = self.expect_ident()?;
                    let end_span = self.current().span;

                    if !self.check(TokenTag::LParen) {
                        return Err(self.error(ParserErrorKind::ExpectedToken {
                            expected: "(".to_string(),
                            found: self.current().kind.tag().to_string(),
                        }));
                    }

                    let mut args = self.parse_call_args()?;

                    if let ExprKind::Tuple(mut t) = expr.kind {
                        while let Some(e) = t.pop() {
                            args.insert(0, e);
                        }
                    } else {
                        args.insert(0, expr);
                    }

                    expr = self.new_expr(
                        start_span,
                        ExprKind::Call {
                            callee: Box::new(Expr {
                                kind: ExprKind::Ident(method_name),
                                span: Span {
                                    start: start_span.start,
                                    end: end_span.end,
                                },
                            }),
                            args,
                        },
                    );

                    continue;
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<ast::Expr>, Box<ParserError>> {
        self.expect(TokenTag::LParen)?;
        let mut args = vec![];

        if self.check(TokenTag::RParen) {
            self.advance();
            return Ok(args);
        }

        loop {
            let expr = self.parse_expr()?;
            args.push(expr);

            if self.check(TokenTag::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenTag::RParen)?;
        Ok(args)
    }

    fn parse_primary_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        match self.current().kind.clone() {
            TokenKind::Int(n) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Int(n)))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Float(n)))
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Bool(b)))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::String(s.to_string())))
            }
            TokenKind::Ident(i) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Ident(i.to_string())))
            }
            TokenKind::If => self.parse_if_expr(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Do => {
                self.expect(TokenTag::Do)?;
                let block = self.parse_block()?;
                Ok(self.new_expr(start_span, ExprKind::Block(block)))
            }
            TokenKind::LParen => {
                self.advance();
                let mut exprs = vec![];
                if self.check(TokenTag::RParen) {
                    self.advance();
                    return Ok(self.new_expr(start_span, ExprKind::Unit));
                }
                let mut is_tuple = self.check(TokenTag::LParen);
                exprs.push(self.parse_expr()?);
                while self.check(TokenTag::Comma) {
                    is_tuple = true;
                    self.advance();
                    exprs.push(self.parse_expr()?);
                }
                self.expect(TokenTag::RParen)?;
                if is_tuple {
                    Ok(self.new_expr(start_span, ExprKind::Tuple(exprs)))
                } else {
                    Ok(exprs[0].clone())
                }
            }
            TokenKind::LBrace => {
                self.expect(TokenTag::LBrace)?;
                let mut exprs = vec![];
                if self.check(TokenTag::RBrace) {
                    self.advance();
                    return Ok(self.new_expr(start_span, ExprKind::Array(exprs)));
                }
                exprs.push(self.parse_expr()?);
                while self.check(TokenTag::Comma) {
                    self.advance();
                    exprs.push(self.parse_expr()?);
                }
                self.expect(TokenTag::RBrace)?;
                Ok(self.new_expr(start_span, ExprKind::Array(exprs)))
            }
            other => Err(self.error(ParserErrorKind::ExpectedExpression {
                found: other.tag().to_string(),
            })),
        }
    }

    fn parse_option_type(&mut self) -> Result<ast::TypeAnnotation, Box<ParserError>> {
        match self.current_tag() {
            TokenTag::Equals => Ok(ast::TypeAnnotation::Inferred),
            TokenTag::Ident => {
                let t = self.expect_ident()?;
                if t == "Arr" {
                    self.expect(TokenTag::LAngle)?;
                    let inner = self.expect_ident()?;
                    self.expect(TokenTag::RAngle)?;
                    Ok(ast::TypeAnnotation::Explicit(ast::TypeExpr::Array(inner)))
                } else {
                    Ok(ast::TypeAnnotation::Explicit(ast::TypeExpr::Named(
                        t.to_string(),
                    )))
                }
            }
            TokenTag::LParen => {
                self.expect(TokenTag::LParen)?;
                match self.current_tag() {
                    TokenTag::RParen => {
                        self.expect(TokenTag::RParen)?;
                        Ok(ast::TypeAnnotation::Explicit(ast::TypeExpr::Unit))
                    }
                    TokenTag::Ident => {
                        let i = self.expect_ident()?;
                        let mut idents = vec![i];
                        while let TokenTag::Comma = self.current_tag() {
                            self.advance();
                            idents.push(self.expect_ident()?);
                        }
                        self.expect(TokenTag::RParen)?;
                        if self.check(TokenTag::RArrow) {
                            self.expect(TokenTag::RArrow)?;
                            let TypeAnnotation::Explicit(return_type) = self.parse_option_type()?
                            else {
                                return Err(self.error(ParserErrorKind::ExpectedTypeAnnotation));
                            };
                            Ok(TypeAnnotation::Explicit(TypeExpr::Function {
                                parameters: idents,
                                return_type: Box::new(return_type),
                            }))
                        } else {
                            Ok(TypeAnnotation::Explicit(TypeExpr::Tuple(idents)))
                        }
                    }
                    other => Err(self.error(ParserErrorKind::ExpectedTokens {
                        expected: vec![")".to_string(), "type".to_string()],
                        found: other.to_string(),
                    })),
                }
            }
            other => Err(self.error(ParserErrorKind::ExpectedTokens {
                expected: vec!["=".to_string(), "type".to_string()],
                found: format!("{other:?}"),
            })),
        }
    }

    fn parse_if_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        self.expect(TokenTag::If)?;
        let condition = Box::new(self.parse_expr()?);
        let then_branch = Box::new(self.parse_expr()?);
        let else_branch = if self.check(TokenTag::Else) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        Ok(self.new_expr(
            start_span,
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
        ))
    }

    fn new_expr(&self, start_span: Span, kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span {
                start: start_span.start,
                end: self.current().span.end,
            },
        }
    }

    fn current(&self) -> &Token<'a> {
        &self.tokens[self.index]
    }

    fn current_tag(&self) -> TokenTag {
        self.current().kind.tag()
    }

    fn check(&self, tag: TokenTag) -> bool {
        self.current_tag() == tag
    }

    fn is_at_end(&self) -> bool {
        self.check(TokenTag::Eof)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.index += 1
        }
    }

    fn _previous(&self) -> Option<&Token<'a>> {
        if self.index > 0 {
            Some(&self.tokens[self.index - 1])
        } else {
            None
        }
    }

    fn consume_newlines(&mut self) {
        while self.check(TokenTag::NewLine) {
            self.advance();
        }
    }

    fn error(&self, kind: ParserErrorKind) -> Box<ParserError> {
        Box::new(ParserError {
            kind,
            file_path: self.file_path.clone(),
            pos: self.current().span,
        })
    }

    fn expect(&mut self, tag: TokenTag) -> Result<(), Box<ParserError>> {
        if self.current_tag() == tag {
            self.advance();
            Ok(())
        } else {
            Err(self.error(ParserErrorKind::ExpectedToken {
                expected: tag.to_string(),
                found: self.current().origin.to_string(),
            }))
        }
    }
    fn expect_many(&mut self, tags: Vec<TokenTag>) -> Result<(), Box<ParserError>> {
        let mut any = false;
        for tag in &tags {
            if *tag == self.current_tag() {
                any = true
            }
        }
        if any {
            self.advance();
            Ok(())
        } else {
            Err(self.error(ParserErrorKind::ExpectedTokens {
                expected: tags.iter().map(|t| t.to_string()).collect(),
                found: self.current().origin.to_string(),
            }))
        }
    }
    fn expect_ident(&mut self) -> Result<String, Box<ParserError>> {
        match self.current().kind.clone() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s.to_string())
            }
            other => Err(self.error(ParserErrorKind::ExpectedIdentifier {
                found: format!("{other:?}"),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use std::{
        ffi::OsStr,
        fs::{read_dir, read_to_string},
    };

    use crate::{dump::create_expected_by_ext, parser};

    #[test]
    fn validate_expected_ast() {
        let cargo_dir = env!("CARGO_MANIFEST_DIR");
        let entries = read_dir(format!("{cargo_dir}/fixtures")).unwrap();

        for entry in entries {
            let current_file_path = entry.unwrap().path();

            if current_file_path.is_file()
                && current_file_path.extension() == Some(OsStr::new("blorp"))
            {
                let content = read_to_string(&current_file_path).unwrap();
                let tokens = Lexer::new(&current_file_path, &content).tokenize();

                let ast_str = match tokens {
                    Ok(tokens) => {
                        let ast = parser::Parser::new(tokens, &current_file_path).parse_program();

                        match ast {
                            Ok(program) => format!("{program:#?}"),
                            Err(e) => format!("{e:#?}"),
                        }
                    }

                    Err(e) => format!("{e:#?}"),
                };

                let ast_expected_path = create_expected_by_ext(&current_file_path, ".ast").unwrap();

                let expected_ast = match read_to_string(ast_expected_path.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("Expected ast file {ast_expected_path:?} not found. Skipping it");
                        continue;
                    }
                };

                assert_eq!(
                    ast_str.trim_end(),
                    expected_ast.trim_end(),
                    "failed to match ast output in file {ast_expected_path:?}"
                );
            }
        }
    }
}
