use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::iter::Peekable;
use std::str::Chars;
use logos::Source;
use crate::diagnostics::{DiagnosticEmitter, Span};

#[derive(Copy, Clone, Debug)]
pub struct Loc<'source> {
	pub file: &'source str,
	pub line: usize,
	pub column: usize
}

impl<'source> Loc<'source> {
	pub fn new(file: &'source str, line: usize, column: usize) -> Self {
		Self {file, line, column}
	}
}

impl<'source> Display for Loc<'source> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}:{}", self.file, self.line, self.column)
	}
}

pub struct SourceMap<'source> {
	file: &'source str,
	lines: Vec<(Span, &'source str)>
}

impl<'source> SourceMap<'source> {
	pub fn new(file: &'source str, src: &'source str) -> Self {
		let mut loc = 0usize;
		let mut lines = Vec::new();
		let mut line = String::new();
		let mut start = 0usize;
		for char in src.chars() {
			if char == '\n' {
				loc += line.len() + 1;
				lines.push((start..loc, src.slice(start..loc).unwrap()));
				start = loc;
				line.clear();
			}
			else {
				line.push(char);
			}
		}
		if !line.is_empty() {
			loc += line.len();
			lines.push((start..loc, src.slice(start..loc).unwrap()));
		}
		Self {file, lines}
	}

	pub fn span_to_loc(&self, span: Span) -> Loc {
		for (i, (range, _)) in self.lines.iter().enumerate() {
			if range.contains(&span.start) {
				let column = span.start - range.start;
				return Loc::new(self.file, i + 1, column + 1);
			}
		}
		let (range, _) = self.lines.last().unwrap();
		return Loc::new(self.file, self.lines.len(), span.start - range.start + 1);
	}

	pub fn eoi_span(&self) -> Span {
		let (range, _) = self.lines.last().unwrap_or(&(0..0, ""));
		range.end..range.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
	Add,
	Minus,
	Multiply,
	Divide,
	Modulo,
	And,
	Or,
	Not
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
	Struct,
	Ret,

	LBrace,
	RBrace,
	LParen,
	RParen,
	Colon,
	Semicolon,
	Dot,
	Comma,
	Arrow,

	BinOp(BinOp),
	Equals,
	BinOpEquals(BinOp),

	Identifier(String),
	CharLiteral(String),
	StringLiteral(String),
	Num(u64)
}

impl Display for TokenType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			TokenType::Struct => write!(f, "struct"),
			TokenType::Ret => write!(f, "ret"),
			TokenType::LBrace => write!(f, "'{{'"),
			TokenType::RBrace => write!(f, "'}}'"),
			TokenType::LParen => write!(f, "'('"),
			TokenType::RParen => write!(f, "')'"),
			TokenType::Colon => write!(f, "':'"),
			TokenType::Semicolon => write!(f, "';'"),
			TokenType::Dot => write!(f, "'.'"),
			TokenType::Comma => write!(f, "','"),
			TokenType::BinOp(_) => write!(f, "an operator"),
			TokenType::Equals => write!(f, "'='"),
			TokenType::BinOpEquals(_) => write!(f, "an operator"),
			TokenType::Identifier(_) => write!(f, "an identifier"),
			TokenType::Num(_) => write!(f, "a number"),
			TokenType::CharLiteral(_) => write!(f, "a character literal"),
			TokenType::StringLiteral(_) => write!(f, "a string literal"),
			TokenType::Arrow => write!(f, "'->'")
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
	pub kind: TokenType,
	pub span: Span
}

impl Token {
	pub fn new(kind: TokenType, span: Span) -> Self {
		Self {kind, span}
	}
}

pub struct Lexer<'source, W: Write> {
	src: Peekable<Chars<'source>>,
	read: usize,
	special_chars: HashMap<char, TokenType>,
	second_special_chars: HashSet<char>,
	keywords: HashMap<&'static str, TokenType>,
	next: [Option<Token>; 2],
	emitter: &'source DiagnosticEmitter<'source, W>,
	has_error: bool
}

pub enum PeekCount {
	One,
	Two
}

impl<'source, W: Write> Lexer<'source, W> {
	pub fn new(src: &'source str, emitter: &'source DiagnosticEmitter<'source, W>) -> Self {
		let special_chars = HashMap::from([
			('+', TokenType::BinOp(BinOp::Add)),
			('-', TokenType::BinOp(BinOp::Minus)),
			('*', TokenType::BinOp(BinOp::Multiply)),
			('/', TokenType::BinOp(BinOp::Divide)),
			('%', TokenType::BinOp(BinOp::Modulo)),
			('|', TokenType::BinOp(BinOp::Or)),
			('&', TokenType::BinOp(BinOp::And)),
			('!', TokenType::BinOp(BinOp::Not)),
			(';', TokenType::Semicolon),
			('.', TokenType::Dot),
			(',', TokenType::Comma),
			('{', TokenType::LBrace),
			('}', TokenType::RBrace),
			('(', TokenType::LParen),
			(')', TokenType::RParen),
			('=', TokenType::Equals),
			(':', TokenType::Colon)
		]);
		let second_special_chars = HashSet::from([
			'=', '>'
		]);
		let keywords = HashMap::from([
			("struct", TokenType::Struct),
			("ret", TokenType::Ret)
		]);
		Self {src: src.chars().peekable(), read: 0, special_chars, second_special_chars,
		keywords, next: [None, None], emitter, has_error: false}
	}

	pub fn peek(&mut self, count: PeekCount) -> Option<Token> {
		match count {
			PeekCount::One => {
				if let Some(token) = &self.next[0] {
					return Some(token.clone());
				}
				let token = self.next_internal();
				self.next[0] = token;
				self.next[0].clone()
			},
			PeekCount::Two => {
				if let Some(token) = &self.next[1] {
					return Some(token.clone());
				}
				if self.next[0].is_none() {
					self.next[0] = self.next_internal();
				}
				let token = self.next_internal();
				self.next[1] = token;
				self.next[1].clone()
			}
		}
	}

	pub fn next(&mut self) -> Option<Token> {
		if let Some(token) = self.next[0].take() {
			self.next[0] = self.next[1].take();
			return Some(token);
		}
		if let Some(token) = self.next[1].take() {
			return Some(token);
		}
		self.next_internal()
	}

	fn next_internal(&mut self) -> Option<Token> {
		loop {
			let start = self.read;

			let char = self.src.next()?;
			self.read += 1;

			if char.is_whitespace() {
				continue;
			}
			else if let Some(first) = self.special_chars.get(&char) {
				let mut token_type = first.clone();
				let mut text = String::from(char);
				if let Some(second) = self.src.peek() {
					if self.second_special_chars.contains(second) {
						if let TokenType::BinOp(op) = token_type {
							if *second == '=' {
								token_type = TokenType::BinOpEquals(op);
							}
							else {
								token_type = TokenType::Arrow;
							}
							text.push(*second);
							self.src.next();
							self.read += 1;
						}
					}
				}

				return Some(Token::new(token_type, start..self.read));
			}
			else if ['"', '\''].contains(&char) {
				let start_char = char;
				let mut text = String::new();
				while let Some(char) = self.src.next_if(|c| *c != start_char) {
					if char == '\\' {
						if let Some(next) = self.src.peek() {
							match *next {
								'n' => text.push('\n'),
								't' => text.push('\t'),
								'\\' => text.push('\\'),
								'0' => text.push('\0'),
								e => {
									self.emitter.error().with_label(
										format!("invalid escape sequence {}", e))
										.with_span(self.read..self.read+1)
										.emit();
									self.has_error = true;
								}
							}
							self.src.next();
							self.read += 1;
						}
					}
					else {
						text.push(char);
					}
					self.read += 1;
				}

				let is_char_literal = start_char == '\'';
				let len = text.len();

				let token_type =
					if is_char_literal { TokenType::CharLiteral } else { TokenType::StringLiteral };

				if self.src.peek().is_none() {
					if is_char_literal {
						self.emitter.error().with_label(format!("unterminated char literal '{}'", text))
							.with_span(start..self.read)
							.emit();
						self.has_error = true;
					}
					else {
						self.emitter.error().with_label(format!("unterminated string literal '{}'", text))
							.with_span(start..self.read)
							.emit();
						self.has_error = true;
					}
				}
				else {
					self.src.next();
				}

				if start_char == '\'' && len > 1 {
					self.emitter.error().with_label(format!("invalid character literal '{}'", text))
						.with_span(start..self.read)
						.emit();
					self.has_error = true;
				}

				return Some(Token::new(token_type(text), start..self.read));
			}
			else {
				let mut text = String::from(char);

				while let Some(char) = self.src.next_if(|c| {
					!c.is_whitespace() && !self.special_chars.contains_key(c)
				}) {
					text.push(char);
					self.read += 1;
				}

				let is_number = text.chars().all(|c| c.is_digit(10));

				let token_type;
				if is_number {
					token_type = TokenType::Num(text.parse().unwrap());
				}
				else if let Some(k) = self.keywords.get(text.as_str()) {
					token_type = k.clone();
				}
				else {
					token_type = TokenType::Identifier(text);
				}

				return Some(Token::new(token_type, start..self.read));
			}
		}
	}

	pub fn has_error(&self) -> bool {
		self.has_error
	}
}