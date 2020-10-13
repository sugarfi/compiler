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
use std::ops::RangeInclusive;
use fnv::FnvHashMap;

fn gen_charset(ranges: &[RangeInclusive<char>], bytes: &str) -> Vec<u8> {
	ranges.iter()
	      .flat_map(|range| range.clone().collect::<Vec<char>>())
	      .map(|c| c as u8)
	      .chain(bytes.bytes())
	      .collect()
}

pub struct Lexer<'a> {
	input: &'a [u8],
	position: usize,
	line: u32,
	col: u32,
	charsets: FnvHashMap<&'a str, Vec<u8>>,
	binary_ops: &'a [&'a [u8]],
}

impl<'a> Lexer<'a> {
	pub fn new(input: &'a [u8]) -> Self {
		let mut charsets = FnvHashMap::with_capacity_and_hasher(5, Default::default());
		charsets.insert("digits", gen_charset(&['0'..='9'], ""));
		charsets.insert("hex", gen_charset(&['0'..='9', 'a'..='f', 'A'..='F'], ""));
		charsets.insert("alpha", gen_charset(&['a'..='z', 'A'..='Z'], ""));
		charsets.insert("symbol", gen_charset(&['a'..='z', 'A'..='Z', '0'..='9'], "_"));
		charsets.insert("ops", gen_charset(&[], "*/"));

		Self {
			input,
			position: 0,
			line: 1,
			col: 1,
			charsets,
			binary_ops: &[b"*", b"/", b"+", b"-", b"++", b"."],
		}
	}

	pub fn is_whitespace(&self, c: &u8) -> bool {
		[b' ', b'\t', b'\n', b'\r'].contains(c)
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

	pub fn charset(&self, name: &str) -> &[u8] {
		self.charsets.get(name).unwrap()
	}

	pub fn try_char(&mut self, c: char) -> bool {
		if self.char_at(0) == c {
			self.advance(1);
			true
		} else {
			false
		}
	}

	pub fn try_number(&mut self) -> Option<f32> {
		let mut n = 0;
		let mut s = "".to_owned();
		let digits = self.charset("digits");

		while digits.contains(&self.at(n)) {
			s.push(self.char_at(n));
			n += 1;
		}

		if n == 0 && self.at(0) != b'.' {
			return None;
		}

		if self.at(n) == b'.' {
			s.push('.');
			n += 1;

			if !digits.contains(&self.at(n)) {
				throw_error("Trailing . not allowed", self.position_at(n));
				return None;
			}

			while digits.contains(&self.at(n)) {
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
		let alpha = self.charset("alpha");
		let symbol = self.charset("symbol");

		if alpha.contains(&self.at(0)) {
			s.push(self.char_at(0));
			n += 1;
		} else {
			return None;
		}

		while symbol.contains(&self.at(n)) {
			s.push(self.char_at(n));
			n += 1;
		}

		self.advance(n);
		Some(s)
	}

	pub fn try_hex(&mut self) -> Option<String> {
		let mut n = 0;
		let mut s = "".to_owned();
		let hex = self.charset("hex");

		if self.at(0) == b'#' {
			n += 1;
		} else {
			return None;
		}

		while hex.contains(&self.at(n)) {
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
}
