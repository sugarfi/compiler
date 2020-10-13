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

mod lexer;

use crate::error::throw_error;
use crate::ast::*;
use lexer::Lexer;
use std::process::exit;
use fnv::FnvHashMap;

fn unexpected(lexer: &Lexer) {
	throw_error(&format!("Unexpected symbol: {:?}", lexer.char_at(0)), lexer.position());
}

// TODO: Function types such as (Number -> Number) -> Number
pub fn parse_type(lexer: &mut Lexer) -> Option<Type> {
	if lexer.try_peek(b"Number") {
		Some(Type::Number)
	} else if lexer.try_peek(b"String") {
		Some(Type::String)
	} else if lexer.try_peek(b"Hex") {
		Some(Type::Hex)
	} else if lexer.try_peek(b"Dimension") {
		Some(Type::Dimension)
	} else if lexer.try_peek(b"Bool") {
		Some(Type::Bool)
	} else if lexer.try_char('(') {
		lexer.skip_whitespace();

		if lexer.try_char(')') {
			Some(Type::Tuple(Vec::new()))
		} else {
			let mut types = Vec::new();

			loop {
				if let Some(t) = parse_type(lexer) {
					types.push(t);
					lexer.skip_whitespace();

					if lexer.try_char(')') {
						break;
					} else if !lexer.try_char(',') {
						unexpected(lexer);
						exit(0);
					}

					lexer.skip_whitespace();
				} else if lexer.try_char(')') {
					break;
				} else {
					unexpected(lexer);
					exit(0);
				}
			}

			Some(Type::Tuple(types))
		}
	} else if lexer.try_char('[') {
		lexer.skip_whitespace();

		if let Some(t) = parse_type(lexer) {
			lexer.skip_whitespace();

			if lexer.try_char(']') {
				Some(Type::List(Box::new(t)))
			} else {
				unexpected(lexer);
				exit(0);
			}
		} else {
			unexpected(lexer);
			exit(0);
		}
	} else if lexer.try_char('{') {
		lexer.skip_whitespace();

		if lexer.try_char('}') {
			Some(Type::Record(FnvHashMap::default()))
		} else {
			let mut types = FnvHashMap::default();

			loop {
				if let Some(s) = lexer.try_symbol() {
					lexer.skip_whitespace();

					if lexer.try_peek(b"::") {
						lexer.skip_whitespace();

						if let Some(t) = parse_type(lexer) {
							types.insert(s, t);
							lexer.skip_whitespace();

							if lexer.try_char('}') {
								break;
							} else if !lexer.try_char(',') {
								unexpected(lexer);
								exit(0);
							}

							lexer.skip_whitespace();
						}
					} else {
						unexpected(lexer);
						exit(0);
					}
				} else if lexer.try_char('}') {
					break;
				} else {
					unexpected(lexer);
					exit(0);
				}
			}

			Some(Type::Record(types))
		}
	} else if let Some(s) = lexer.try_symbol() {
		Some(Type::Alias(s))
	} else {
		None
	}
}

fn parse_expr(lexer: &mut Lexer) -> Option<Expr> {
	let a =
		if lexer.try_char('(') {
			lexer.skip_whitespace();

			if lexer.try_char(')') {
				Expr::Tuple(Vec::new())
			} else {
				let mut contents = Vec::new();
				let mut trailing = false;

				loop {
					if let Some(e) = parse_expr(lexer) {
						contents.push(e);
						lexer.skip_whitespace();

						if lexer.try_char(')') {
							break;
						} else if !lexer.try_char(',') {
							unexpected(lexer);
							exit(0);
						}

						lexer.skip_whitespace();
					} else if lexer.try_char(')') {
						trailing = true;
						break;
					} else {
						unexpected(lexer);
						exit(0);
					}
				}

				if contents.len() == 1 && !trailing {
					contents.get(0).unwrap().clone()
				} else {
					Expr::Tuple(contents)
				}
			}
		} else if lexer.try_char('[') {
			lexer.skip_whitespace();

			if lexer.try_char(']') {
				Expr::List(Vec::new())
			} else {
				let mut contents = Vec::new();

				loop {
					if let Some(e) = parse_expr(lexer) {
						contents.push(e);
						lexer.skip_whitespace();

						if lexer.try_char(']') {
							break;
						} else if !lexer.try_char(',') {
							unexpected(lexer);
							exit(0);
						}

						lexer.skip_whitespace();
					} else if lexer.try_char(']') {
						break;
					} else {
						unexpected(lexer);
						exit(0);
					}
				}

				Expr::List(contents)
			}
		} else if lexer.try_char('{') {
			lexer.skip_whitespace();

			if lexer.try_char('}') {
				Expr::Record(FnvHashMap::default())
			} else {
				let mut contents = FnvHashMap::default();

				loop {
					if let Some(s) = lexer.try_symbol() {
						lexer.skip_whitespace();

						if lexer.try_char(':') {
							lexer.skip_whitespace();

							if let Some(e) = parse_expr(lexer) {
								contents.insert(s, e);
								lexer.skip_whitespace();

								if lexer.try_char('}') {
									break;
								} else if !lexer.try_char(',') {
									unexpected(lexer);
									exit(0);
								}

								lexer.skip_whitespace();
							}
						} else {
							unexpected(lexer);
							exit(0);
						}
					} else if lexer.try_char('}') {
						break;
					} else {
						unexpected(lexer);
						exit(0);
					}
				}

				Expr::Record(contents)
			}
		} else if lexer.try_char('$') {
			if let Some(s) = lexer.try_symbol() {
				Expr::Symbol(s)
			} else {
				unexpected(lexer);
				exit(0);
			}
		} else if let Some(n) = lexer.try_number() {
			if let Some(u) = lexer.try_symbol() {
				Expr::Dimension(n, u)
			} else {
				Expr::Number(n)
			}
		} else if let Some(s) = lexer.try_string() {
			Expr::String(s)
		} else if let Some(h) = lexer.try_hex() {
			Expr::Hex(h)
		} else if let Some(b) = lexer.try_bool() {
			Expr::Bool(b)
		} else if let Some(s) = lexer.try_symbol() {
			if lexer.try_char('(') {
				lexer.skip_whitespace();

				if lexer.try_char(')') {
					Expr::Call(s, Vec::new())
				} else {
					let mut contents = Vec::new();

					loop {
						if let Some(e) = parse_expr(lexer) {
							contents.push(e);
							lexer.skip_whitespace();

							if lexer.try_char(')') {
								break;
							} else if !lexer.try_char(',') {
								unexpected(lexer);
								exit(0);
							}

							lexer.skip_whitespace();
						} else if lexer.try_char(')') {
							break;
						} else {
							unexpected(lexer);
							exit(0);
						}
					}

					Expr::Call(s, contents)
				}
			} else {
				Expr::Symbol(s)
			}
		} else {
			return None;
		};

	Some(
		if lexer.try_char('[') {
			lexer.skip_whitespace();

			if let Some(e) = parse_expr(lexer) {
				lexer.skip_whitespace();

				if lexer.try_char(']') {
					Expr::Index(Box::new(a), Box::new(e))
				} else {
					unexpected(lexer);
					exit(0);
				}
			} else {
				unexpected(lexer);
				exit(0);
			}
		} else if let Some(op) = lexer.try_binary_op() {
			lexer.skip_whitespace();

			if let Some(b) = parse_expr(lexer) {
				Expr::BinaryOp(op, Box::new(a), Box::new(b))
			} else {
				unexpected(lexer);
				exit(0);
			}
		} else {
			a
		}
	)
}

// fn parse_selector(lexer: &mut Lexer) -> Option<Node> {

// }

fn parse_function(lexer: &mut Lexer) -> Option<Node> {
	if let Some(s) = lexer.try_symbol() {
		if lexer.try_char('(') {
			lexer.skip_whitespace();

			let params = if lexer.try_char(')') {
				Vec::new()
			} else {
				let mut params = Vec::new();

				loop {
					if let Some(s) = lexer.try_symbol() {
						params.push(s);
						lexer.skip_whitespace();

						if lexer.try_char(')') {
							break;
						} else if !lexer.try_char(',') {
							unexpected(lexer);
							exit(0);
						}

						lexer.skip_whitespace();
					} else if lexer.try_char(')') {
						break;
					} else {
						unexpected(lexer);
						exit(0);
					}
				}

				params
			};

			lexer.skip_whitespace();

			let types = if lexer.try_peek(b"::") {
				lexer.skip_whitespace();

				if let Some(t) = parse_type(lexer) {
					let mut types = vec![t];

					while lexer.try_arrow() {
						lexer.skip_whitespace();

						if let Some(t) = parse_type(lexer) {
							types.push(t);
						} else {
							unexpected(lexer);
							exit(0);
						}
					}

					if lexer.try_newline() {
						types
					} else {
						unexpected(lexer);
						exit(0);
					}
				} else {
					unexpected(lexer);
					exit(0);
				}
			} else {
				unexpected(lexer);
				exit(0);
			};

			let mut nodes = Vec::new();

			loop {
				while lexer.try_newline() {}

				if lexer.try_indent(1) {
					if let Some(e) = parse_expr(lexer) {
						nodes.push(Node::Expr(e));
					}
				} else {
					break;
				}
			}

			Some(Node::Function(s, params, types, nodes))
		} else {
			unexpected(lexer);
			exit(0);
		}
	} else {
		None
	}
}

// fn parse_definition(lexer: &mut Lexer) -> Option<Node> {

// }

// fn parse_enum(lexer: &mut Lexer) -> Option<Node> {

// }

fn parse_root_node(lexer: &mut Lexer) -> Option<Node> {
	// if let Some(n) = parse_selector(lexer) {
	// 	n
	// } else 
	if let Some(n) = parse_function(lexer) {
		Some(n)
	// } else if let Some(n) = parse_definition(lexer) {
	// 	n
	// } else if let Some(n) = parse_enum(lexer) {
	// 	n
	} else if lexer.try_newline() {
		None
	} else {
		unexpected(lexer);
		exit(0);
	}
}

pub fn parse(input: &[u8]) -> Vec<Node> {
	let mut lexer = Lexer::new(&input);
	let mut ast = Vec::new();

	while lexer.has_left() {
		match parse_root_node(&mut lexer) {
			Some(n) => ast.push(n),
			None => (),
		}
	}

	ast
}
