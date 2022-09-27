use std::collections::HashMap;
use crate::diagnostics::Span;

pub type Spanned<T> = (T, Span);

#[derive(Debug, Clone)]
pub enum Expr {
	Num(u64),

	Add(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
	Sub(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
	Mul(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
	Div(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
	Mod(Box<Spanned<Expr>>, Box<Spanned<Expr>>),

	Struct {
		fields: HashMap<String, String>
	}
}