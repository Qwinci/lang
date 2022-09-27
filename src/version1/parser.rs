use std::collections::HashMap;
use chumsky::prelude::*;
use crate::Token;

#[derive(Debug)]
pub enum Expr {
	Var(String),

	Num(u64),

	Struct {
		fields: HashMap<String, String>
	},

	Function {
		name: String,
		args: Vec<(String, String)>,
		ret_type: Option<String>,
		body: Vec<Expr>
	},

	Assign {
		target: Box<Expr>,
		value: Box<Expr>
	},

	VarDecl {
		name: String,
		r#type: String,
		value: Option<Box<Expr>>
	},

	Construct {
		r#type: String,
		fields: HashMap<String, Box<Expr>>
	},

	Call {
		name: String,
		args: Vec<Expr>
	},

	FieldAccess {
		var_name: String,
		name: String
	},

	Ret(Box<Expr>),

	Neg(Box<Expr>),
	Add(Box<Expr>, Box<Expr>),
	Sub(Box<Expr>, Box<Expr>),
	Mul(Box<Expr>, Box<Expr>),
	Div(Box<Expr>, Box<Expr>),
	Mod(Box<Expr>, Box<Expr>)
}

pub fn parser() -> impl Parser<Token, Vec<Expr>, Error = Simple<Token>> {
	let field_access = ident()
		.then_ignore(just(Token::Dot))
		.then(ident())
		.map(|(var_name, name)| Expr::FieldAccess {
			var_name,
			name
		});

	let expr = recursive(|expr| {
		let int = num().map(|value| Expr::Num(value));

		let construct = ident()
			.then_ignore(just(Token::LBrace))
			.then(just(Token::Dot).ignore_then(ident())
				.then_ignore(just(Token::Equals))
				.then(expr.clone()).separated_by(just(Token::Comma)))
			.then_ignore(just(Token::RBrace))
			.map(|(r#type, fields)| Expr::Construct {
				r#type,
				fields: fields.into_iter().map(|(name, expr)| (name, Box::new(expr))).collect()
			});

		let call = ident()
			.then(expr.clone()
				.separated_by(just(Token::Comma))
				.delimited_by(just(Token::LParen), just(Token::RParen)))
			.map(|(name, args)| Expr::Call {
				name,
				args
			});

		let atom = int
			.or(expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)))
			.or(construct)
			.or(call)
			.or(field_access.clone())
			.or(ident().map(Expr::Var));


		// 1 + 2 * 2
		let unary = just(Token::Minus)
			.repeated()
			.then(atom)
			.foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

		let product = unary.clone()
			.then(just(Token::Multiply).to(Expr::Mul as fn(_, _) -> _)
				.or(just(Token::Divide).to(Expr::Div as fn(_, _) -> _))
				.or(just(Token::Modulo).to(Expr::Mod as fn(_, _) -> _))
				.then(unary)
				.repeated())
			.foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

		let sum = product.clone()
			.then(just(Token::Plus).to(Expr::Add as fn(_, _) -> _)
				.or(just(Token::Minus).to(Expr::Sub as fn(_, _) -> _))
				.then(product)
				.repeated())
			.foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

		sum
	});

	let decl = || {
		let semicolon = just(Token::Semicolon)
			.map(Ok)
			.or_else(|e| Ok(Err(e)))
			.validate(|out: Result<Token, Simple<Token>>, _, emit| match out {
				Ok(out) => out,
				Err(e) => {
					emit(e);
					Token::Semicolon
				}
			});

		let ret = just(Token::Ret)
			.ignore_then(expr.clone()).then_ignore(semicolon.clone())
			.map(|expr| Expr::Ret(Box::new(expr)));

		let keyword = ret;

		let assign_var = ident()
			.then_ignore(just(Token::Equals))
			.then(expr.clone())
			.then_ignore(just(Token::Semicolon))
			.map(|(name, value)| Expr::Assign {
				target: Box::new(Expr::Var(name)),
				value: Box::new(value)
			});

		let name_type = ident()
			.then_ignore(just(Token::Colon))
			.then(ident());

		let decl_var = name_type.clone()
			.then_ignore(just(Token::Semicolon))
			.map(|(name, r#type)| Expr::VarDecl {
				name,
				r#type,
				value: None
			});

		let var_decl_assign = name_type
			.then_ignore(just(Token::Equals))
			.then(expr.clone())
			.then_ignore(just(Token::Semicolon))
			.map(|((name, r#type), value)| Expr::VarDecl {
				name,
				r#type,
				value: Some(Box::new(value))
			});

		let field_assign = field_access.clone()
			.then_ignore(just(Token::Equals))
			.then(expr.clone())
			.then_ignore(just(Token::Semicolon))
			.map(|(field, value)| Expr::Assign {
				target: Box::new(field),
				value: Box::new(value)
			});

		let assign_struct = ident()
			.then_ignore(just(Token::Equals))
			.then(r#struct())
			.map(|(name, content)| Expr::Assign {
				target: Box::new(Expr::Var(name)),
				value: Box::new(content)
			});

		let block = just(Token::LBrace).ignore_then(
			choice((assign_var.clone(),
				 decl_var.clone(),
				 var_decl_assign.clone(),
				 keyword.clone(),
				 field_assign.clone())).repeated()
				.then_ignore(just(Token::RBrace))
		);

		let args = just(Token::Arrow)
			.ignore_then(ident());

		let assign_fn = ident()
			.then_ignore(just(Token::Equals))
			.then_ignore(just(Token::LParen))
			.then(ident().then_ignore(just(Token::Colon)).then(ident())
				.separated_by(just(Token::Comma)))
			.then_ignore(just(Token::RParen))
			.then(args.or_not())
			.then(block.clone())
			.map(|(((name, args), ret_type), body)| Expr::Function {
				name,
				args,
				ret_type,
				body
			});

		choice((assign_struct, assign_fn))
	};

	decl().repeated().then_ignore(end())
}

fn r#struct() -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
	let start =
		just(Token::Struct).ignore_then(just(Token::LBrace));

	let field = ident().then_ignore(just(Token::Colon)).then(ident());

	start.ignore_then(field.separated_by(just(Token::Comma)).allow_trailing())
		.then_ignore(just(Token::RBrace))
		.map(|fields| Expr::Struct { fields: fields.into_iter().collect() })
}

fn ident() -> impl Parser<Token, String, Error = Simple<Token>> + Clone {
	select! {
		Token::Identifier(text) => text
	}
}

fn num() -> impl Parser<Token, u64, Error = Simple<Token>> + Clone {
	select! {
		Token::Number(num) => num
	}
}