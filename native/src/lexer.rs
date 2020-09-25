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

use self::Token::*;
use cow_rc_str::CowRcStr;
use std::char;

#[derive(Debug)]
pub enum Token<'a> {
	// https://drafts.csswg.org/css-syntax/#comment-diagram
	// Excludes /* and */
	MultilineComment(CowRcStr<'a>),

	// https://drafts.csswg.org/css-syntax/#newline-diagram
	Newline,

	// Tab or quadrupal-space
	Indent,

	// https://drafts.csswg.org/css-syntax/#ident-token-diagram
	Ident(CowRcStr<'a>),

	// A valid CSS selector
	Selector(CowRcStr<'a>),

	// https://drafts.csswg.org/css-syntax/#function-token-diagram
	// Excludes (
	Function(CowRcStr<'a>),

	// https://drafts.csswg.org/css-syntax/#at-keyword-token-diagram
	// Excludes @
	AtRule(CowRcStr<'a>),

	// https://drafts.csswg.org/css-syntax/#hash-token-diagram
	// Excludes #
	Hash(CowRcStr<'a>),

	// https://drafts.csswg.org/css-syntax/#string-token-diagram
	// Excludes quotes
	String(CowRcStr<'a>),

	// https://drafts.csswg.org/css-syntax/#url-token-diagram
	Number(f32),

	// https://drafts.csswg.org/css-syntax/#dimension-token-diagram
	Dimension {
		value: f32,
		unit: CowRcStr<'a>,
	},

	// https://drafts.csswg.org/css-syntax/#percentage-token-diagram
	// Excludes %
	Percentage(f32),

	// !important rule
	Important,

	// Operator
	Operator(CowRcStr<'a>),

	// :
	Colon,

	// ,
	Comma,

	// )
	CloseParen,
}

pub struct Lexer<'a> {
	input: &'a [u8],
	position: usize,
	lineno: u32,
}

impl<'a> Lexer<'a> {
	#[inline]
	pub fn new(input: &'a str) -> Lexer<'a> {
		Lexer {
			input: input.as_bytes(),
			position: 0,
			lineno: 0,
		}
	}

	#[inline]
	pub fn next(&mut self) -> Option<Token<'a>> {
		match self.next_byte() {
			None => None,
			Some(b) => match b {
				b'\t' => {
					self.advance(1);
					Some(Indent)
				},
				b' ' => if self.starts_with(b"   ") {
					self.advance(4);
					Some(Indent)
				} else {
					self.advance(1);
					self.next()
				},
				b'\n' | b'\r' | b'\x0C' => {
					self.advance(1);
					Some(Newline)
				},
				b':' => {
					self.advance(1);
					Some(Colon)
				},
				b',' => {
					self.advance(1);
					Some(Comma)
				},
				b')' => {
					self.advance(1);
					Some(CloseParen)
				},
				b'/' => {
					if self.starts_with(b"/*") {
						self.advance(2);
						let mut comment = "".to_owned();
						loop {
							if self.starts_with(b"*/") {
								self.advance(2);
								break;
							} else {
								match self.next_byte() {
									None => panic!("Could not find closing */"),
									Some(b) => comment.push(b.into()),
								}
							}
							self.advance(1);
						}
						Some(MultilineComment(comment.into()))
					} else if self.starts_with(b"//") {
						// Single line comment, skip to next line
						self.advance(2);
						loop {
							match self.next_byte() {
								None => break,
								Some(b) => match b {
									b'\n' | b'\r' | b'\x0C' => {
										self.advance(1);
										break;
									},
									_ => (),
								}
							}
							self.advance(1);
						}
						self.next()
					} else {
						unimplemented!()
					}
				},
				c @ b'"' | c @ b'\'' => {
					let mut string = "".to_owned();
					loop {
						self.advance(1);
						match self.next_byte() {
							None => panic!("Could not find closing {}", c),
							Some(b) => if b == c {
								self.advance(1);
								break;
							} else {
								string.push(b.into());
							},
						}
					}
					Some(String(string.into()))
				},
				b'-' => match self.next_byte() {
					None => panic!("Unexpected eof"),
					Some(b) => match b {
						b'-' => Some(Ident(self.ident("--"))),
						c @ b'a'..=b'z' | c @ b'A'..=b'Z' | c @ b'_' => Some(Ident(self.ident(&format!("{}{}", "-", c as char)))),
						c @ b'0'..=b'9' => Some(self.number(&format!("{}{}", "-", c as char))),
						_ => panic!("Unexpected token"),
					},
				},
				c @ b'a'..=b'z' | c @ b'A'..=b'Z' | c @ b'_' => {
					let ident = self.ident(&format!("{}", c as char));
			    	Some(
						match self.next_byte() {
							None => Ident(ident),
							Some(b) => match b {
								b'(' => {
									self.advance(1);
									Function(ident)
								},
								_ => Ident(ident),
							},
						}
					)
				},
				c @ b'0'..=b'9' => Some(self.number(&format!("{}", c as char))),
				_ => panic!("Unrecognized token"),
			},
		}
	}

	#[inline]
	pub fn advance(&mut self, n: usize) {
		self.position += n;
	}

	#[inline]
	pub fn next_byte(&self) -> Option<u8> {
		if self.has_at_least(0) {
            Some(self.input[self.position])
        } else {
            None
        }
	}

	#[inline]
	pub fn byte_at(&self, offset: usize) -> u8 {
        self.input[self.position + offset]
    }

	#[inline]
    pub fn starts_with(&self, needle: &[u8]) -> bool {
        self.input[self.position..].starts_with(needle)
    }

	#[inline]
    fn has_at_least(&self, n: usize) -> bool {
        self.position + n < self.input.len()
    }

	#[inline]
    fn ident(&mut self, start: &str) -> CowRcStr<'a> {
    	let mut ident = start.to_owned();
    	self.advance(start.len());

    	loop {
    		match self.next_byte() {
    			None => break,
    			Some(b) => match b {
    				c @ b'a'..=b'z' | c @ b'A'..=b'Z' | c @ b'0'..=b'9' | c @ b'_' => ident.push(c.into()),
    				_ => break,
    			},
    		}
    		self.advance(1);
    	}

    	ident.into()
    }

	#[inline]
    fn number(&mut self, start: &str) -> Token<'a> {
    	let mut number = start.to_owned();
    	let mut periods = 0;
    	self.advance(start.len());

    	loop {
    		match self.next_byte() {
    			None => break,
    			Some(b) => match b {
    				c @ b'0'..=b'9' => number.push(c.into()),
    				b'.' => {
    					if periods == 0 {
    						number.push('.');
    					} else {
    						break;
    					}
    					periods += 1;
    				},
    				_ => break,
    			},
    		}
    		self.advance(1);
    	}

    	Number(number.parse::<f32>().unwrap())
    }
}
