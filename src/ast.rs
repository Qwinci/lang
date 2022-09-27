use std::collections::HashMap;
use crate::diagnostics::Span;

pub type Spanned<T> = (T, Span);

#[derive(Debug, Clone)]
pub enum Expr {
	Error,
	Num(u64),

	Neg(Box<Expr>),
	Add(Box<Expr>, Box<Expr>),
	Sub(Box<Expr>, Box<Expr>),
	Mul(Box<Expr>, Box<Expr>),
	Div(Box<Expr>, Box<Expr>),
	Mod(Box<Expr>, Box<Expr>),
	And(Box<Expr>, Box<Expr>),
	Or(Box<Expr>, Box<Expr>),

	Struct {
		fields: HashMap<String, String>
	}
}