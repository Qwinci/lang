use std::io::Write;
use crate::{DiagnosticEmitter, Lexer, Token};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::Span;
use crate::lexer::{BinOp, PeekCount, TokenType};

pub struct Parser<'source, W: Write> {
	lexer: Lexer<'source, W>,
	emitter: &'source DiagnosticEmitter<'source, W>,
	has_error: bool
}

#[derive(Debug)]
enum Recovery {
	Continue(Span),
	Break(Span),
	TopLevel(Span),
	Eof
}

impl<'source, W: Write> Parser<'source, W> {
	pub fn new(lexer: Lexer<'source, W>,
	           emitter: &'source DiagnosticEmitter<'source, W>) -> Self {
		Self {lexer, emitter, has_error: false}
	}

	fn next(&mut self) -> Option<Token> {
		let token = self.lexer.next();
		self.has_error |= self.lexer.has_error();
		token
	}

	fn peek(&mut self, count: PeekCount) -> Option<Token> {
		let token = self.lexer.peek(count);
		self.has_error |= self.lexer.has_error();
		token
	}

	fn peek_one(&mut self) -> Option<Token> {
		self.peek(PeekCount::One)
	}

	fn get_prec(token: &Token) -> Option<u32> {
		match &token.kind {
			TokenType::BinOp(op) => match op {
				BinOp::Add | BinOp::Minus => Some(10),
				BinOp::Multiply | BinOp::Divide | BinOp::Modulo => Some(20),
				BinOp::And | BinOp::Or => Some(5),
				BinOp::Not => None
			}
			_ => None
		}
	}

	fn parse_binexp(&mut self, mut lhs: Expr, min_precedence: u32) -> Expr {
		let mut next = self.peek_one();
		while let Some(token) = next {
			let op_prec;
			if let Some(prec) = Self::get_prec(&token) {
				if prec < min_precedence {
					break;
				}
				op_prec = prec;
			}
			else {
				break;
			}

			let op = self.next().unwrap();

			let mut rhs = match self.parse_primary() {
				Some(primary) => primary,
				None => {
					let op_len = op.span.end - op.span.start;
					self.emitter.error()
						.with_label(format!("expected a primary expression after {}", op.kind))
						.with_span(op.span.start+op_len..op.span.end+op_len)
						.emit();
					self.has_error = true;
					Expr::Error
				}
			};

			next = self.peek_one();

			while let Some(token) = &next {
				if let Some(prec) = Self::get_prec(&token) {
					if prec <= op_prec {
						break;
					}
				}
				else {
					break;
				}

				let is_greater = match &next {
					Some(token) => {
						if let Some(prec) = Self::get_prec(token) {
							if prec > op_prec {
								1
							}
							else {
								0
							}
						}
						else {
							0
						}
					}
					None => 0
				};
				rhs = self.parse_binexp(rhs, op_prec + is_greater);
				next = self.peek_one();
			}

			let op = match op.kind {
				TokenType::BinOp(op) => {
					match op {
						BinOp::Add => Expr::Add,
						BinOp::Minus => Expr::Sub,
						BinOp::Multiply => Expr::Mul,
						BinOp::Divide => Expr::Div,
						BinOp::Modulo => Expr::Mod,
						BinOp::And => Expr::And,
						BinOp::Or => Expr::Or,
						_ => unreachable!()
					}
				}
				_ => unreachable!()
			};

			lhs = op(Box::new(lhs), Box::new(rhs));
		}

		return lhs;
	}

	fn parse_primary(&mut self) -> Option<Expr> {
		let mut minus_stack = Vec::new();
		while let Some(token) = self.peek_one() {
			if let TokenType::BinOp(op) = token.kind {
				if op == BinOp::Minus {
					minus_stack.push(BinOp::Minus);
					self.next();
				}
				else {
					break;
				}
			}
			else {
				break
			}
		}

		let primary_token = self.peek_one()?;

		match primary_token.kind {
			TokenType::Num(num) => {
				self.next();
				Some(
					minus_stack.into_iter()
						.fold(Expr::Num((num, primary_token.span)),
						      |e, _| Expr::Neg(Box::new(e)))
				)
			},
			TokenType::Identifier(ident) => {
				self.next();
				if let Some(next) = self.peek_one() {
					if next.kind == TokenType::LBrace {
						self.next();

						let mut fields = Vec::new();
						while let Some(token) = self.peek_one() {
							if token.kind == TokenType::RBrace {
								break;
							}

							if self.expect(&[TokenType::Dot]).is_none() {
								break;
							}

							let name = match self.parse_ident("a field name") {
								Some(ident) => ident,
								None => break
							};

							if self.expect(&[TokenType::Equals]).is_none() {
								break;
							}

							let value = self.parse_atom();

							fields.push((name, Box::new(value)));
						}

						self.expect(&[TokenType::RBrace]);

						Some(Expr::Construct {name: (ident, primary_token.span), fields})
					}
					else if next.kind == TokenType::Dot {
						self.next();
						let name = match self.parse_ident("a field name") {
							Some(ident) => ident,
							None => return None
						};

						Some(Expr::FieldAccess {name: (ident, primary_token.span), field: name})
					}
					else {
						Some(Expr::Var((ident, primary_token.span)))
					}
				}
				else {
					Some(Expr::Var((ident, primary_token.span)))
				}
			},
			TokenType::CharLiteral(literal) => {
				self.next();
				Some(Expr::CharLiteral((literal, primary_token.span)))
			},
			TokenType::StringLiteral(literal) => {
				self.next();
				Some(Expr::StringLiteral((literal, primary_token.span)))
			}
			TokenType::LParen => {
				self.next();
				let expr = self.parse_atom();
				let next = self.peek_one();
				if let Some(next) = next {
					if next.kind != TokenType::RParen {
						self.emitter.error()
							.with_label(format!("expected ')' but got {}", next.kind))
							.with_span(next.span)
							.emit();
						self.has_error = true;
					}
					else {
						self.next();
					}
				}
				else {
					self.emitter.error()
						.with_label("expected ')'")
						.with_eoi_span()
						.emit();
					self.has_error = true;
				}
				Some(expr)
			}
			_ => None
		}
	}

	fn expect(&mut self, expected: &[TokenType]) -> Option<Token> {
		let label = move || {
			let mut label = "expected ".to_string();
			let len = expected.len();
			if len == 1 {
				label += expected[0].to_string().as_str();
				return label;
			}
			else if len == 2 {
				label += (expected[0].to_string() + " or " + expected[1].to_string().as_str()).as_str();
				return label;
			}
			for (i, e) in expected.into_iter().enumerate() {
				if i < len.saturating_sub(1) {
					label += format!("{}", e).as_str();
				}
				else {
					label += format!(" or {}", e).as_str();
				}

				if i < len.saturating_sub(2) {
					label += ", ";
				}
			}
			label
		};
		match self.peek_one() {
			Some(token) => {
				if expected.contains(&token.kind) {
					self.next();
					Some(token)
				}
				else {
					let label = label() + format!(" but got {}", token.kind).as_str();
					self.emitter.error()
						.with_label(label)
						.with_span(token.span)
						.emit();
					self.has_error = true;
					None
				}
			}
			None => {
				let label = label() + " but found eof";
				self.emitter.error()
					.with_label(label)
					.with_eoi_span()
					.emit();
				self.has_error = true;
				None
			}
		}
	}

	fn parse_ident(&mut self, name: &str) -> Option<Spanned<String>> {
		match self.peek_one() {
			Some(token) => match token.kind {
				TokenType::Identifier(ident) => {
					self.next();
					Some((ident, token.span))
				},
				_ => {
					self.emitter.error()
						.with_label(format!("expected {} but got {}", name, token.kind))
						.with_span(token.span)
						.emit();
					self.has_error = true;
					None
				}
			}
			None => {
				self.emitter.error()
					.with_label("expected an identifier but found eof")
					.with_eoi_span()
					.emit();
				self.has_error = true;
				None
			}
		}
	}

	fn parse_ident_type(&mut self) -> Option<(Spanned<String>, Spanned<String>)> {
		let name = self.parse_ident("an identifier")?;

		self.expect(&[TokenType::Colon])?;

		let r#type = self.parse_ident("a type")?;

		Some((name, r#type))
	}

	fn skip_until(&mut self, until: &[(TokenType, usize)]) {
		while let Some(token) = self.peek_one() {
			if let Some(next) = self.peek(PeekCount::Two) {
				for (t, amount) in until.iter() {
					if *amount == 2 && next.kind == *t {
						return;
					}
				}
			}
			for (t, amount) in until.iter() {
				if token.kind == *t {
					if *amount == 0 {
						self.next();
						return;
					}
					else if *amount == 1 {
						return;
					}
				}
			}

			self.next();
		}
	}

	fn recover(&mut self, next_elem_mark: Option<TokenType>, clause_end_mark: Option<TokenType>) -> Recovery {
		let mut braces = 0;
		while let Some(token) = self.peek_one() {
			if token.kind == TokenType::LBrace || token.kind == TokenType::LParen {
				braces += 1;
			}
			else if token.kind == TokenType::RBrace || token.kind == TokenType::RParen {
				braces -= 1;
			}
			if let Some(next_elem_mark) = next_elem_mark.clone() {
				if token.kind == next_elem_mark && braces == 0 {
					return Recovery::Continue(token.span);
				}
			}
			if let Some(clause_end_mark) = clause_end_mark.clone() {
				if token.kind == clause_end_mark && braces == 0 {
					return Recovery::Break(token.span);
				}
			}

			self.next();
		}

		if braces > 0 {
			self.emitter.error()
				.with_eoi_span()
				.with_label("mismatched braces")
				.emit();
		}

		Recovery::Eof
	}

	fn parse_assign(&mut self, target: Expr) -> Expr {
		// =
		let equals = self.next().unwrap();

		let token = match self.peek_one() {
			Some(token) => token,
			None => {
				self.emitter.error()
					.with_label("expected an expression")
					.with_eoi_span()
					.emit();
				self.has_error = true;
				Token::new(TokenType::Num(0), 0..0)
			}
		};

		let mut name = (String::new(), 0..0);
		if token.kind == TokenType::Struct || token.kind == TokenType::LParen {
			name = match &target {
				Expr::Var(ident) => ident.clone(),
				_ => {
					self.emitter.error()
						.with_label("expected an identifier")
						.with_span(equals.span)
						.emit();
					self.has_error = true;
					(String::new(), 0..0)
				}
			};
		}

		if token.kind == TokenType::Struct {
			self.next();

			if self.expect(&[TokenType::LBrace]).is_none() {
				let mut good = false;
				if let Some(token) = self.peek_one() {
					if token.kind == TokenType::RBrace {
						good = true;
					}
					else if let TokenType::Identifier(_) = token.kind {
						good = true;
					}
				}

				if !good {
					match self.peek(PeekCount::Two) {
						Some(token) => {
							if token.kind != TokenType::Colon {
								return Expr::Error;
							}
						}
						None => {
							return Expr::Error;
						}
					}
				}
			}

			let mut fields = Vec::new();
			let mut is_good = false;
			while let Some(token) = self.peek_one() {
				if token.kind == TokenType::RBrace {
					self.next();
					is_good = true;
					break;
				}

				let name_type = match self.parse_ident_type() {
					Some(name_type) => name_type,
					None => {
						self.skip_until(&[(TokenType::Semicolon, 0)]);
						return Expr::Error;
					}
				};

				fields.push(name_type);

				match self.expect(&[TokenType::Comma, TokenType::RBrace]) {
					Some(token) => {
						if token.kind == TokenType::RBrace {
							is_good = true;
							break;
						}
					}
					None => {
						return Expr::Struct {name, fields};
					}
				}
			}

			if !is_good {
				self.emitter.error()
					.with_label("expected '}' but found eof")
					.with_eoi_span()
					.emit();
				self.has_error = true;
			}

			return Expr::Struct {name, fields};
		}
		else if token.kind == TokenType::LParen {
			self.next();

			let mut skip_signature = false;
			if let Some(token) = self.peek_one() {
				if let TokenType::Identifier(_) = token.kind {}
				else if token.kind == TokenType::LBrace {
					self.emitter.error()
						.with_label(format!("expected ')' but got {}", token.kind))
						.with_span(token.span)
						.emit();
					self.has_error = true;
					skip_signature = true;
				}
			}

			let mut args = Vec::new();
			if !skip_signature {
				while let Some(token) = self.peek_one() {
					if token.kind == TokenType::RParen {
						self.next();
						break;
					}

					let name_type = match self.parse_ident_type() {
						Some(name_type) => name_type,
						None => return Expr::Error
					};

					args.push(name_type);

					match self.expect(&[TokenType::Comma, TokenType::RParen]) {
						Some(token) => {
							if token.kind == TokenType::RParen {
								break;
							}
						}
						None => {
							if let Some(token) = self.peek_one() {
								if let TokenType::Identifier(_) = token.kind {}
								else {
									break;
								}
							}
						}
					}
				}
			}

			let mut ret_type = (String::new(), 0..0);
			if let Some(token) = self.peek_one() {
				if token.kind == TokenType::Arrow {
					self.next();

					let r#type = match self.parse_ident("a type") {
						Some(ident) => ident,
						None => {
							if let Some(token) = self.peek_one() {
								if token.kind != TokenType::Comma {
									return Expr::Assign {target: Box::new(target),
										value: Box::new(Expr::Error)}
								}
								else {
									(String::new(), 0..0)
								}
							}
							else {
								return Expr::Assign {target: Box::new(target),
									value: Box::new(Expr::Error)}
							}
						}
					};

					ret_type = r#type;
				}
			}

			let s = self.expect(&[TokenType::LBrace, TokenType::Semicolon]);
			match s {
				Some(s) => {
					if s.kind == TokenType::Semicolon {
						return Expr::Function {name, args, ret_type, body: None};
					}
				}
				None => {
					return Expr::Function {name, args, ret_type, body: None};
				}
			}

			let mut body = Vec::new();
			while let Some(token) = self.peek_one() {
				if token.kind == TokenType::RBrace {
					break;
				}

				body.push(self.parse_expression());
			}

			self.expect(&[TokenType::RBrace]);

			return Expr::Function {name, args, ret_type, body: Some(body)};
		}
		else if token.kind == TokenType::RParen {
			self.next();
			self.emitter.error()
				.with_label(format!("expected '(' but got {}", token.kind))
				.with_span(token.span)
				.emit();
			self.has_error = true;
			if let Some(token) = self.expect(&[TokenType::Semicolon, TokenType::LBrace]) {
				if token.kind == TokenType::Semicolon {
					return Expr::Error;
				}
				else {
					self.skip_until(&[(TokenType::RBrace, 0)]);
					return Expr::Error;
				}
			}
			else {
				self.skip_until(&[(TokenType::RBrace, 0)]);
				return Expr::Error;
			}
		}
		else {
			let value = self.parse_atom();
			self.expect(&[TokenType::Semicolon]);
			Expr::Assign {target: Box::new(target), value: Box::new(value)}
		}

	}

	fn parse_vardecl(&mut self, name: Spanned<String>) -> Expr {
		self.next();

		let r#type = match self.parse_ident("a type") {
			Some(ident) => ident,
			None => {
				return Expr::Error;
			}
		};

		let s = self.expect(&[TokenType::Equals, TokenType::Semicolon]);
		if let Some(s) = s {
			if s.kind == TokenType::Equals {
				let value = self.parse_atom();
				self.expect(&[TokenType::Semicolon]);
				Expr::VarDecl {name, r#type, value: Some(Box::new(value))}
			}
			else {
				Expr::VarDecl {name, r#type, value: None}
			}
		}
		else {
			Expr::VarDecl {name, r#type, value: None}
		}
	}

	fn parse_atom(&mut self) -> Expr {
		let primary = match self.parse_primary() {
			Some(expr) => expr,
			None => {
				match self.peek_one() {
					Some(token) => {
						self.next();
						self.emitter.error()
							.with_label(format!("expected a primary expression but got {}", token.kind))
							.with_span(token.span)
							.emit();
						self.has_error = true;
						return Expr::Error;
					}
					None => {
						self.emitter.error()
							.with_label("expected a primary expression but found eof")
							.with_eoi_span()
							.emit();
						self.has_error = true;
						return Expr::Error;
					}
				}
			}
		};

		let token = match self.peek_one() {
			Some(token) => token,
			None => {
				return primary;
			}
		};

		if let TokenType::BinOp(_) = token.kind {
			self.parse_binexp(primary, 0)
		}
		else {
			primary
		}
	}

	fn parse_expression(&mut self) -> Expr {
		let primary = match self.parse_primary() {
			Some(token) => token,
			None => {
				match self.peek_one() {
					Some(token) => {
						if token.kind == TokenType::Ret {
							self.next();
							if let Some(token) = self.peek_one() {
								if token.kind == TokenType::Semicolon {
									self.next();
									return Expr::Ret {value: None};
								}
							}
							let value = self.parse_atom();
							self.expect(&[TokenType::Semicolon]);
							return Expr::Ret {value: Some(Box::new(value))};
						}

						self.next();
						self.emitter.error()
							.with_label(format!("expected a primary expression but got {}", token.kind))
							.with_span(token.span)
							.emit();
						self.has_error = true;
						return Expr::Error;
					}
					None => {
						self.emitter.error()
							.with_label("expected a primary expression but found eof")
							.with_eoi_span()
							.emit();
						self.has_error = true;
						return Expr::Error;
					}
				}
			}
		};

		let token = match self.peek_one() {
			Some(token) => token,
			None => {
				self.emitter.error()
					.with_label("expected an expression but found eof")
					.with_eoi_span()
					.emit();
				self.has_error = true;
				return Expr::Error;
			}
		};

		match token.kind {
			TokenType::BinOp(_) => self.parse_binexp(primary, 0),
			TokenType::Equals => self.parse_assign(primary),
			TokenType::Colon => {
				if let Expr::Var(var) = primary {
					self.parse_vardecl(var)
				}
				else {
					self.emitter.error()
						.with_label("expected an identifier before ':'")
						.with_span(token.span)
						.emit();
					self.has_error = true;
					Expr::Error
				}
			},
			t => todo!("{}", t)
		}
	}

	fn has_eof(&mut self) -> bool {
		self.lexer.peek(PeekCount::One).is_none()
	}

	fn parse_toplevel_decl(&mut self) -> Expr {
		self.parse_expression()
	}

	pub fn parse(&mut self) -> Vec<Expr> {
		let mut ast = Vec::new();
		while !self.has_eof() {
			ast.push(self.parse_toplevel_decl());
		}

		ast
	}
}