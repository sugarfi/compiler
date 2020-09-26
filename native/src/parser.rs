/*
 * Copyright (C) 2020 GiraffeKey
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::lexer::{Lexer, Token};
use crate::nodes::*;
use cow_rc_str::CowRcStr;

pub struct Parser<'a> {
	lexer: Lexer<'a>,
	nodes: Vec<Node<'a>>,
}

impl<'a> Parser<'a> {
	#[inline]
	pub fn new(input: &'a str) -> Parser<'a> {
		Parser {
			lexer: Lexer::new(input),
			nodes: Vec::new(),
		}
	}

	#[inline]
	pub fn parse(&mut self) -> &Vec<Node<'a>> {
		loop {
			match self.next() {
				None => break,
				Some(node) => self.nodes.push(node),
			}
		}
		&self.nodes
	}

	#[inline]
	fn next(&mut self) -> Option<Node<'a>> {
		match self.lexer.next() {
			None => None,
			Some(token) => match token {
				Token::MultilineComment(comment) => Some(Node::Comment(comment)),
				Token::Ident(selector) => Some(self.selector(selector, 0)),
				Token::Colon => Some(self.selector(":".into(), 0)),
				Token::Function(func) => Some(self.mixin(func)),
				Token::Newline |
				Token::Indent => self.next(),
				t => panic!("{:?}", t),
			},
		}
	}

	#[inline]
	fn expect_indent(&mut self, indent: u32) -> bool {
		let mut s = "".to_owned();
		for _ in 0..indent {
			s.push('\t');
		}
		if self.lexer.starts_with(s.as_bytes()) && self.lexer.byte_at(indent as usize) != Some(b'\t') {
			self.lexer.advance(indent as usize);
			true
		} else {
			false
		}
	}

	#[inline]
	fn skip_newline(&mut self) {
		match self.lexer.next_byte() {
			None => (),
			Some(b) => match b {
				b'\n' | b'\r' | b'\x0C' => self.lexer.advance(1),
				_ => (),
			},
		}
	}

	#[inline]
	fn value(&mut self) -> Value<'a> {
		let value = match self.lexer.next() {
			None => panic!("Expected value"),
			Some(token) => match token {
				Token::Ident(i) => Value::Keyword(i),
				Token::Number(n) => Value::Number(n),
				Token::String(s) => Value::String(s),
				Token::Hex(h) => Value::Hex(h),
				Token::Dimension{ value, unit } => Value::Dimension(value, unit),
				Token::OpenBracket => {
					let expr = self.expr();
					match self.lexer.next() {
						None => panic!("Expected }"),
						Some(token) => match token {
							Token::CloseBracket => (),
							_ => panic!("Expected }"),
						}
					}
					Value::Interop(expr)
				}
				_ => panic!("Not a value"),
			},
		};

		self.skip_newline();

		value
	}

	#[inline]
	fn expr(&mut self) -> Expr<'a> {
		let pos = self.lexer.position();
		match self.lexer.next() {
			None => panic!("Expected expression"),
			Some(token) => match token {
				Token::Ident(ident) => Expr::Variable(ident),
				_ => {
					self.lexer.set_position(pos);
					Expr::Value(Box::new(self.value()))
				},
			},
		}
	}

	#[inline]
	fn selector(&mut self, selstr: CowRcStr<'a>, indent: u32) -> Node<'a> {
		let mut current_sel: String = selstr.to_string();
		let mut sels = Vec::new();
		let mut props = Vec::new();
		let mut calls = Vec::new();
		let mut nested = Vec::new();

		// Find all comma separated selectors
		loop {
			if !self.expect_indent(indent) {
				sels.push(current_sel.into());
				break;
			}
			match self.lexer.next() {
				None => {
					sels.push(current_sel.into());
					break;
				},
				Some(token) => match token {
					Token::Comma => {
						sels.push(current_sel.into());
						// Find next selector in list, ignoring newlines
						loop {
							match self.lexer.next() {
								None => panic!("Expected selector"),
								Some(token2) => match token2 {
									Token::Ident(ident) => {
										current_sel = ident.to_string();
										break;
									},
									Token::Newline => (),
									_ => panic!("Expected selector"),
								}
							}
						}
					},
					Token::Newline => {
						sels.push(current_sel.into());
						break;
					},
					Token::Ident(ident) => {
						current_sel += &ident;
					},
					Token::Colon => {
						current_sel.push(':');
					},
					t => panic!("Expected comma or line break, found {:?}", t),
				},
			}
		}

		// Find all properties and nested selectors
		loop {
			if !self.expect_indent(indent + 1) {
				break;
			}
			match self.lexer.next() {
				None => break,
				Some(token) => match token {
					Token::Ident(ident) => {
						match self.lexer.next() {
							None => panic!("Expected :"),
							Some(token2) => match token2 {
								Token::Colon => props.push(Property {
									name: ident,
									value: self.value(),
								}),
								Token::Newline => if let Node::Selector(selector) = self.selector(ident, indent + 1) {
									nested.push(selector);
								},
								Token::Indent => (),
								_ => panic!("Expected :"),
							},
						}
					},
					Token::Function(name) => {
						let mut args = Vec::new();

						'b: loop {
							args.push(self.value());
							'c: loop {
								match self.lexer.next() {
									None => panic!("Expected , or )"),
									Some(token) => match token {
										Token::Comma => break 'c,
										Token::CloseParen => break 'b,
										Token::Newline |
										Token::Indent => (),
										_ => panic!("Expected , or )"),
									},
								}
							}
						}

						self.skip_newline();

						calls.push(MixinCall {
							name,
							args,
						});
					},
					t => panic!("Expected property or selector, found {:?}", t),
				},
			}
		}

		Node::Selector(Selector {
			sels,
			props,
			calls,
			nested,
		})
	}

	#[inline]
	fn mixin(&mut self, mixin: CowRcStr<'a>) -> Node<'a> {
		let name = mixin;
		let mut params = Vec::new();
		let mut props = Vec::new();

		'b: loop {
			match self.lexer.next() {
				None => panic!("Unmatched parentheses"),
				Some(token) => match token {
					Token::Ident(ident) => {
						params.push(ident);
						'c: loop {
							match self.lexer.next() {
								None => panic!("Expected , or )"),
								Some(token) => match token {
									Token::Comma => break 'c,
									Token::CloseParen => break 'b,
									Token::Newline |
									Token::Indent => (),
									t => panic!("Expected , or ), found {:?}", t),
								},
							}
						}
					},
					Token::Newline |
					Token::Indent => (),
					_ => panic!("Expected parameter"),
				},
			}
		}

		self.skip_newline();

		loop {
			if !self.expect_indent(1) {
				break;
			}
			match self.lexer.next() {
				None => break,
				Some(token) => match token {
					Token::Ident(ident) => match self.lexer.next() {
						None => panic!("Expected :"),
						Some(token) => match token {
							Token::Colon => props.push(Property {
								name: ident,
								value: self.value(),
							}),
							_ => panic!("Expected :"),
						},
					},
					t => panic!("Expected property, found {:?}", t),
				},
			}
		}

		Node::Mixin(Mixin {
			name,
			params,
			props,
		})
	}
}
