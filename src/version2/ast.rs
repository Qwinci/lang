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
		name: Spanned<String>,
		fields: Vec<(Spanned<String>, Spanned<String>)>
	},

	Function {
		name: Spanned<String>,
		args: Vec<(Spanned<String>, Spanned<String>)>,
		ret_type: Spanned<String>,
		body: Option<Vec<Expr>>
	},

	VarDecl {
		name: Spanned<String>,
		r#type: Spanned<String>,
		value: Option<Box<Expr>>
	},

	Construct {
		name: Spanned<String>,
		fields: Vec<(Spanned<String>, Box<Expr>)>
	},

	FieldAccess {
		name: Spanned<String>,
		field: Spanned<String>
	},

	Ret {
		value: Option<Box<Expr>>
	}
}