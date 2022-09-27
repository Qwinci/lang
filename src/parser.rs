use crate::{DiagnosticEmitter, Lexer, Token};
use crate::ast::Expr;

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
		None
	}

	pub fn parse(&mut self) -> Option<Expr> {
		while let Some(token) = self.lexer.next() {}
		None
	}
}