use std::env::args;
use std::fs::read_to_string;
#[cfg(test)]
use std::process::Command;
use crate::diagnostics::DiagnosticEmitter;
use crate::lexer::{Lexer, SourceMap, Token};
use crate::parser::Parser;

mod lexer;
mod parser;
mod ast;
mod diagnostics;

#[cfg(test)]
macro_rules! word_count_stderr {
    ($test_num:expr, $word:expr, $count:expr) => {{
	    let output = Command::new("target/debug/lang")
	        .arg("--test")
	        .arg(($test_num).to_string())
	        .output().unwrap();
	    let output = unsafe { String::from_utf8_unchecked(output.stderr) };
		if output.matches($word).count() != $count {
		    panic!("{}", output);
	    }
    }};
}

#[test]
fn test() {
	word_count_stderr!(0, "error", 2);
	word_count_stderr!(1, "error", 1);
	word_count_stderr!(2, "error", 1);
}

const TESTS: &[&'static str] = &[
	r"struct {
	",
	r"Test = struct",
	r"Test = struct {"
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
		let test_num_str = "test".to_string() + test_num.to_string().as_str();
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