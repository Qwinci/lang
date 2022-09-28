use std::env::{args, current_exe};
use std::fs::read_to_string;
use std::process::Command;
use crate::diagnostics::DiagnosticEmitter;
use crate::lexer::{Lexer, SourceMap, Token};
use crate::parser::Parser;

mod lexer;
mod parser;
mod ast;
mod diagnostics;

macro_rules! word_count_stderr {
    ($test_num:expr, $word:expr) => {{
	    let output = Command::new("target/debug/lang")
	        .arg("--test")
	        .arg(($test_num).to_string())
	        .output().unwrap();
	    let output = unsafe { String::from_utf8_unchecked(output.stderr) };
		output.matches($word).count()
    }};
}

#[test]
fn test() {
	let count = word_count_stderr!(0, "error");
	assert_eq!(count, 1)
}

const TESTS: &[&'static str] = &[
	r"struct {
	"
];

fn main() {
	let args: Vec<String> = args().collect();
	let mut next_is_num = false;
	let mut test_num = 0usize;
	let mut test = false;
	for arg in args {
		if arg == "--test" {
			next_is_num = true;
			test = true;
		}
		else if next_is_num {
			test_num = arg.parse().unwrap();
			break;
		}
	}

	if test {
		let test_num_str = test_num.to_string();
		let map = SourceMap::new(test_num_str.as_str(), TESTS[test_num]);
		let emitter = DiagnosticEmitter::new(&map);
		let lexer = Lexer::new(TESTS[test_num], &emitter);
		let mut parser = Parser::new(lexer, &emitter);
		println!("{:?}", parser.parse());
		return;
	}

	let src = read_to_string("tests/test2.lang").unwrap();
	let map = lexer::SourceMap::new("tests/test2.lang", &src);
	let emitter = DiagnosticEmitter::new(&map);
	let lexer = Lexer::new(&src, &emitter);
	let mut parser = Parser::new(lexer, &emitter);
	let result = parser.parse();
	println!("{:?}", result);
}