use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Eq, Hash)]
pub enum Token {
	#[error]
	#[regex(r"[ \t\n\f]+", logos::skip)]
	#[regex("//[^\n]*", logos::skip)]
	#[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", logos::skip)]
	Error,

	#[token("=")]
	Equals,

	#[token("{")]
	LBrace,

	#[token("}")]
	RBrace,

	#[token("(")]
	LParen,

	#[token(")")]
	RParen,

	#[token(":")]
	Colon,

	#[token(";")]
	Semicolon,

	#[token(".")]
	Dot,

	#[token(",")]
	Comma,

	#[token("+")]
	Plus,

	#[token("->")]
	Arrow,

	#[token("-")]
	Minus,

	#[token("*")]
	Multiply,

	#[token("/")]
	Divide,

	#[token("%")]
	Modulo,

	#[token("struct")]
	Struct,

	#[token("ret")]
	Ret,

	#[regex("[a-zA-Z][a-zA-Z0-9]*", |lex| lex.slice().to_string())]
	Identifier(String),

	#[regex("[0-9]+", |lex| lex.slice().parse())]
	Number(u64)
}