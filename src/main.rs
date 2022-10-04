use std::fs::read_to_string;
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::SimpleReason;
use chumsky::Stream;
use logos::Logos;
use crate::lexer::Token;
use chumsky::Parser;

mod parser;
mod lexer;

fn main() {
	let src = read_to_string("tests/test2.lang").unwrap();
	let lex = Token::lexer(&src);

	for (token, _) in lex.clone().spanned() {
		println!("{:?}", token);
	}
	println!("-----------------------");

	let eoi_span = src.len()..src.len();

	let stream = Stream::from_iter(eoi_span, lex.spanned());

	//let result = parser::parser().parse(stream);
	let result = parser::parser().parse(stream);

	eprintln!("{:?}", result);

	if let Err(errors) = result {
		for error in errors {
			let mut report = Report::build(ReportKind::Error, "tests/test2.lang", error.span().start)
				.with_label(Label::new(("tests/test2.lang", error.span()))
					.with_message("note: error occurred here").with_color(Color::Cyan));

				report.set_message(match error.reason() {
					SimpleReason::Unexpected => {
						if let Some(error_label) = error.label() {
							format!("expected {}", error_label)
						}
						else {
							"unexpected token".to_string()
						}
					}
					SimpleReason::Unclosed {delimiter, ..} => {
						format!("unclosed delimiter {:?}", delimiter)
					}
					SimpleReason::Custom(msg) => msg.clone()
				});

				report.finish()
					.eprint(("tests/test2.lang", Source::from(&src)))
					.unwrap();
		}
	}
}