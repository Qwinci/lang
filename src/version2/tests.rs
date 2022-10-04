use crate::diagnostics;
use crate::lexer::{Lexer, SourceMap};
use crate::parser::Parser;

#[cfg(test)]
macro_rules! test {
    ($src:expr) => {{
	    let mut output = String::new();

		let map = SourceMap::new("test", $src);
		let emitter = diagnostics::with_string(&map, &mut output);
		let lexer = Lexer::new($src, &emitter);
		let mut parser = Parser::new(lexer, &emitter);
		let _ = parser.parse();

	    output
    }};
}
#[cfg(test)]
macro_rules! word_count {
    ($haystack:expr, $word:expr, $count:expr) => {{
	    if $haystack.matches($word).count() != $count {
		    panic!("{}", $haystack);
	    }
    }};
}
#[cfg(test)]
macro_rules! error_count {
    ($haystack:expr, $count:expr) => {
	    word_count!($haystack, "error", $count);
    };
}
#[cfg(test)]
macro_rules! test_error {
    ($src:expr, $count:expr) => {{
	    let output = test!($src);
	    error_count!(output, $count);
    }};
}

#[test]
fn test_struct_missing_lbrace_0() {
	test_error!(r"a = struct", 1);
}

#[test]
fn test_struct_missing_lbrace_1() {
	test_error!(r"a = struct }", 1);
}

#[test]
fn test_struct_missing_rbrace() {
	test_error!(r"a = struct {", 1);
}

#[test]
fn test_struct_missing_rbrace_with_other_expr() {
	test_error!(r"a = struct { ; a = 10;", 1);
}

#[test]
fn test_struct_invalid_type() {
	test_error!(r"a = struct {whatever: 10}", 1);
}

#[test]
fn test_struct_invalid_name() {
	test_error!(r"a = struct {10: 10}", 1);
}

#[test]
fn test_function_missing_lparen() {
	test_error!(r"a = ) {}", 1);
}

#[test]
fn test_function_missing_rparen() {
	test_error!(r"a = ( {}", 1);
}