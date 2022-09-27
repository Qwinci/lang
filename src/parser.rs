use crate::{DiagnosticEmitter, Lexer, Token};
use crate::ast::Expr;
use crate::lexer::{BinOp, TokenType};

pub struct Parser<'source> {
	lexer: Lexer<'source>,
	emitter: &'source DiagnosticEmitter<'source>
}

impl<'source> Parser<'source> {
	pub fn new(lexer: Lexer<'source>,
	           emitter: &'source DiagnosticEmitter<'source>) -> Self {
		Self {lexer, emitter}
	}

	fn next(&mut self) -> Option<Token> {
		self.lexer.next()
	}

	fn peek(&mut self) -> Option<Token> {
		self.lexer.peek()
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
		let mut next = self.peek();
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
					Expr::Error
				}
			};

			next = self.peek();

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
				next = self.peek();
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
		while let Some(token) = self.peek() {
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

		let primary_token = self.peek()?;

		match primary_token.kind {
			TokenType::Num(num) => {
				self.next();
				Some(
					minus_stack.into_iter()
						.fold(Expr::Num(num), |e, _| Expr::Neg(Box::new(e)))
				)
			},
			TokenType::LParen => {
				self.next();
				let expr = self.parse()?;
				let next = self.peek(); // todo check for )
				if let Some(next) = next {
					if next.kind != TokenType::RParen {
						self.emitter.error()
							.with_label(format!("expected ')' but got {}", next.kind))
							.with_span(next.span)
							.emit();
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
				}
				Some(expr)
			}
			_ => None
		}
	}

	pub fn parse(&mut self) -> Option<Expr> {
		let primary = self.parse_primary()?;
		let a = self.parse_binexp(primary, 0);
		Some(a)
	}
}