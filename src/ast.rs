use crate::diagnostics::Span;

pub type Spanned<T> = (T, Span);

#[derive(Debug, Clone)]
pub enum Expr {
	Error,
	Var(Spanned<String>),
	Num(Spanned<u64>),
	CharLiteral(Spanned<String>),
	StringLiteral(Spanned<String>),

	Neg(Box<Expr>),
	Add(Box<Expr>, Box<Expr>),
	Sub(Box<Expr>, Box<Expr>),
	Mul(Box<Expr>, Box<Expr>),
	Div(Box<Expr>, Box<Expr>),
	Mod(Box<Expr>, Box<Expr>),
	And(Box<Expr>, Box<Expr>),
	Or(Box<Expr>, Box<Expr>),

	Assign {
		target: Box<Expr>,
		value: Box<Expr>
	},

	Struct {
		fields: Vec<(Spanned<String>, Spanned<String>)>
	},

	Function {
		args: Vec<(Spanned<String>, Spanned<String>)>,
		ret_type: Spanned<String>,
		body: Vec<Expr>
	},

	FunctionDecl {
		args: Vec<(Spanned<String>, Spanned<String>)>,
		ret_type: Spanned<String>
	}
}