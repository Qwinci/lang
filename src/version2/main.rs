use std::fs::read_to_string;
use crate::diagnostics::DiagnosticEmitter;
use crate::lexer::{Lexer, SourceMap, Token};
use crate::parser::Parser;

mod lexer;
mod parser;
mod ast;
mod diagnostics;
mod tests;

fn main() {
	let src = read_to_string("../../tests/test2.lang").unwrap();
	let map = SourceMap::new("tests/test2.lang", &src);
	let emitter = diagnostics::with_stderr(&map);
	let lexer = Lexer::new(&src, &emitter);
	let mut parser = Parser::new(lexer, &emitter);
	let result = parser.parse();
	println!("{:?}", result);
}