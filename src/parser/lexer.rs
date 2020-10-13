/*
 *  Copyright (c) 2020, GiraffeKey
 *
 *  All rights reserved.
 *
 *  Redistribution and use in source and binary forms, with or without
 *  modification, are permitted provided that the following conditions are met:
 *
 *  Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 *
 *  Redistributions in binary form must reproduce the above
 *  copyright notice, this list of conditions and the following
 *  disclaimer in the documentation and/or other materials provided
 *  with the distribution.
 *
 *  Neither the name of GiraffeKey nor the names of other
 *  contributors may be used to endorse or promote products derived
 *  from this software without specific prior written permission.
 *
 *  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 *  "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 *  LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 *  A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 *  OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 *  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 *  LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 *  DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 *  THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 *  (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 *  OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use crate::error::throw_error;

static WHITESPACE: &[u8] = b" \t\n\r";
static DIGITS: &[u8] = b"0123456789";
static HEX: &[u8] = b"0123456789abcdefABCDEF";
static ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
static SYMBOL: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";

pub struct Lexer<'a> {
	input: &'a [u8],
	position: usize,
	line: u32,
	col: u32,
	binary_ops: &'a [&'a [u8]],
}

impl<'a> Lexer<'a> {
	pub fn new(input: &'a [u8]) -> Self {
		Self {
			input,
			position: 0,
			line: 1,
			col: 1,
			binary_ops: &[b"*", b"/", b"+", b"-", b"++", b"."],
		}
	}

	pub fn is_whitespace(&self, c: &u8) -> bool {
		WHITESPACE.contains(c)
	}

	pub fn position(&self) -> (u32, u32) {
		(self.line, self.col)
	}

	pub fn position_at(&self, n: usize) -> (u32, u32) {
		let mut line = self.line;
		let mut col = self.col;

		for i in 0..n {
			let next = self.at(i);

			if next == 0 {
				break;
			} else if next == b'\n' {
				line += 1;
				col = 1;
			} else {
				col += 1;
			}
		}

		(line, col)
	}

	pub fn at(&self, n: usize) -> u8 {
		if self.position + n < self.input.len() {
			self.input[self.position + n]
		} else {
			0
		}
	}

	pub fn char_at(&self, n: usize) -> char {
		self.at(n).into()
	}

	pub fn peek(&self, needle: &[u8]) -> bool {
		self.input[self.position..].starts_with(needle)
	}

	pub fn peek_at(&self, n: usize, needle: &[u8]) -> bool {
		if self.position + n < self.input.len() {
			self.input[self.position + n..].starts_with(needle)
		} else {
			false
		}
	}

	pub fn advance(&mut self, n: usize) {
		for _ in 0..n {
			let next = self.at(0);

			if next == 0 {
				break;
			} else if next == b'\n' {
				self.line += 1;
				self.col = 1;
			} else {
				self.col += 1;
			}

			self.position += 1;
		}
	}

	pub fn has_left(&self) -> bool {
		self.at(0) != 0
	}

	pub fn skip_whitespace(&mut self) {
		while self.is_whitespace(&self.at(0)) {
			self.advance(1);
		}
	}

	pub fn try_char(&mut self, c: char) -> bool {
		if self.char_at(0) == c {
			self.advance(1);
			true
		} else {
			false
		}
	}

	pub fn try_peek(&mut self, s: &[u8]) -> bool {
		if self.peek(s) {
			self.advance(s.len());
			true
		} else {
			false
		}
	}

	pub fn try_newline(&mut self) -> bool {
		let mut n = 0;

		while [b' ', b'\t'].contains(&self.at(n)) {
			n += 1;
		}

		if [b'\n', b'\r'].contains(&self.at(n)) {
			self.advance(n + 1);
			true
		} else {
			false
		}
	}

	pub fn try_indent(&mut self, indent: usize) -> bool {
		let mut n = 0;

		while self.at(n) == b'\t' {
			n += 1;
		}

		if n == indent {
			self.advance(n);
			return true;
		}

		n = 0;

		while self.peek_at(n, b"    ") {
			n += 1;
		}

		if n == indent {
			self.advance(n * 4);
			true
		} else {
			false
		}
	}

	pub fn try_number(&mut self) -> Option<f32> {
		let mut n = 0;
		let mut s = "".to_owned();

		while DIGITS.contains(&self.at(n)) {
			s.push(self.char_at(n));
			n += 1;
		}

		if n == 0 && self.at(0) != b'.' {
			return None;
		}

		if self.at(n) == b'.' {
			s.push('.');
			n += 1;

			if !DIGITS.contains(&self.at(n)) {
				throw_error("Trailing . not allowed", self.position_at(n));
				return None;
			}

			while DIGITS.contains(&self.at(n)) {
				s.push(self.char_at(n));
				n += 1;
			}
		}
		
		self.advance(n);
		Some(s.parse().unwrap())
	}

	pub fn try_string(&mut self) -> Option<String> {
		let mut n = 0;
		let mut s = "".to_owned();

		if self.at(0) == b'"' {
			n += 1;
		} else {
			return None;
		}

		while self.at(n) != b'"' {
			if self.peek_at(n, b"\\\"") {
				s.push('"');
				n += 2;
			} else if self.at(n) == 0 {
				throw_error("\" not closed", self.position_at(n));
				return None;
			} else {
				s.push(self.char_at(n));
				n += 1;
			}
		}

		self.advance(n + 1);
		Some(s)
	}

	pub fn try_symbol(&mut self) -> Option<String> {
		let mut n = 0;
		let mut s = "".to_owned();

		if ALPHA.contains(&self.at(0)) {
			s.push(self.char_at(0));
			n += 1;
		} else {
			return None;
		}

		while SYMBOL.contains(&self.at(n)) {
			s.push(self.char_at(n));
			n += 1;
		}

		self.advance(n);
		Some(s)
	}

	pub fn try_hex(&mut self) -> Option<String> {
		let mut n = 0;
		let mut s = "".to_owned();

		if self.at(0) == b'#' {
			n += 1;
		} else {
			return None;
		}

		while HEX.contains(&self.at(n)) {
			s.push(self.char_at(n));
			n += 1;
		}

		if n == 1 {
			throw_error("Expected hexadecimal", self.position_at(n));
			return None;
		}

		self.advance(n);
		Some(s)
	}

	pub fn try_bool(&mut self) -> Option<bool> {
		if self.peek(b"true") {
			self.advance(4);
			Some(true)
		} else if self.peek(b"false") {
			self.advance(5);
			Some(false)
		} else {
			None
		}
	}

	pub fn try_binary_op(&mut self) -> Option<String> {
		let mut n = 0;

		while self.is_whitespace(&self.at(n)) {
			n += 1;
		}

		for op in self.binary_ops {
			if self.peek_at(n, op) {
				self.advance(n + op.len());
				return Some(std::str::from_utf8(op).unwrap().into());
			}
		}
			
		None
	}

	pub fn try_arrow(&mut self) -> bool {
		let mut n = 0;

		while self.is_whitespace(&self.at(n)) {
			n += 1;
		}

		if self.peek_at(n, b"->") {
			self.advance(n + 2);
			true
		} else {
			false
		}
	}
}
