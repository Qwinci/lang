use std::fs::read_to_string;
use crate::diagnostics::DiagnosticEmitter;
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;

mod lexer;
mod parser;
mod ast;
mod diagnostics;

fn main() {
	let src = read_to_string("test_src/test2.lang").unwrap();
	let map = lexer::SourceMap::new("test_src/test2.lang", &src);
	let emitter = DiagnosticEmitter::new(&map);
	let lexer = Lexer::new(&src, &emitter);
	let mut parser = Parser::new(lexer, &emitter);
	let result = parser.parse();
	println!("{:?}", result);
}