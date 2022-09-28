use std::fs::read_to_string;
use chumsky::Stream;
use logos::Logos;
use crate::lexer::Token;
use chumsky::Parser;

mod parser;
mod lexer;

fn main() {
	let src = read_to_string("../../tests/test2.lang").unwrap();
	let lex = Token::lexer(&src);

	for (token, _) in lex.clone().spanned() {
		println!("{:?}", token);
	}
	println!("-----------------------");

	let eoi_span = src.len()..src.len();

	let stream = Stream::from_iter(eoi_span, lex.spanned());

	//let result = parser::parser().parse(stream);
	let result = parser::parser().parse_recovery(stream);

	println!("{:?}", result);
}