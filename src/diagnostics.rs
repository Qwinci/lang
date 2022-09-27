use std::fmt::Display;
use std::ops::Range;
use crate::lexer::SourceMap;

pub type Span = Range<usize>;

pub struct DiagnosticEmitter<'a> {
	map: &'a SourceMap<'a>
}

#[macro_export]
macro_rules! colored {
    ($str:literal, $color:expr) => {concat!($str, $color)};
}

pub mod color {
	pub const RESET: &'static str = "\x1b[0m";
	pub const BLACK: &'static str = "\x1b[30m";
	pub const RED: &'static str = "\x1b[31m";
	pub const GREEN: &'static str = "\x1b[32m";
	pub const YELLOW: &'static str = "\x1b[33m";
	pub const BLUE: &'static str = "\x1b[34m";
	pub const MAGENTA: &'static str = "\x1b[35m";
	pub const CYAN: &'static str = "\x1b[36m";
	pub const WHITE: &'static str = "\x1b[37m";
	pub const BRIGHT_BLACK: &'static str = "\x1b[90m";
	pub const BRIGHT_RED: &'static str = "\x1b[91m";
	pub const BRIGHT_GREEN: &'static str = "\x1b[92m";
	pub const BRIGHT_YELLOW: &'static str = "\x1b[93m";
	pub const BRIGHT_BLUE: &'static str = "\x1b[94m";
	pub const BRIGHT_MAGENTA: &'static str = "\x1b[95m";
	pub const BRIGHT_CYAN: &'static str = "\x1b[96m";
	pub const BRIGHT_WHITE: &'static str = "\x1b[97m";
}

pub enum EmitType {
	Info,
	Warning,
	Error
}

pub struct Emit<'source> {
	label: String,
	span: Span,
	emit_type: EmitType,
	map: &'source SourceMap<'source>
}

impl<'source> Emit<'source> {
	fn new(map: &'source SourceMap<'source>) -> Self {
		Self {label: String::new(), span: 0..0, emit_type: EmitType::Info, map}
	}

	pub fn with_label<T: Display>(mut self, label: T) -> Self {
		self.label = label.to_string();
		self
	}

	pub fn with_span(mut self, span: Span) -> Self {
		self.span = span;
		self
	}

	pub fn with_type(mut self, emit_type: EmitType) -> Self {
		self.emit_type = emit_type;
		self
	}

	pub fn emit(self) {
		match self.emit_type {
			EmitType::Info => {
				eprintln!("{}info: {}{}", color::GREEN, color::RESET, self.label);
				eprintln!("  {}--> {}{}{}", color::CYAN, color::BLUE,
				self.map.span_to_loc(self.span), color::RESET);
			},
			EmitType::Warning => {
				eprintln!("{}warning: {}{}", color::YELLOW, color::RESET, self.label);
				eprintln!("  {}--> {}{}{}", color::CYAN, color::BLUE,
				          self.map.span_to_loc(self.span), color::RESET);
			}
			EmitType::Error => {
				eprintln!("{}error: {}{}", color::RED, color::RESET, self.label);
				eprintln!("  {}--> {}{}{}", color::CYAN, color::BLUE,
				          self.map.span_to_loc(self.span), color::RESET);
			}
		}
	}
}

impl<'a> DiagnosticEmitter<'a> {
	pub fn new(map: &'a SourceMap<'a>) -> Self {
		Self {map}
	}

	pub fn info(&self) -> Emit {
		Emit::new(self.map).with_type(EmitType::Info)
	}

	pub fn warning(&self) -> Emit {
		Emit::new(self.map).with_type(EmitType::Warning)
	}

	pub fn error(&self) -> Emit {
		Emit::new(self.map).with_type(EmitType::Error)
	}
}