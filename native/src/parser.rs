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

use crate::tokenizer::Rule;
use crate::nodes::*;
use pest::iterators::{Pair, Pairs};
use peeking_take_while::PeekableExt;

/*
 * Parses token into Expr enum
 */
fn parse_expr(token: Pair<Rule>) -> Expr {
	match token.as_rule() {
		Rule::symbol => Expr::Keyword(token.as_str().into()),
		Rule::hash => Expr::Hash(token.as_str().into()),
		Rule::number => Expr::Number(token.as_str().parse().unwrap()),
		Rule::single_string |
		Rule::double_string => Expr::String(token.as_str().into()),
		Rule::dimension => {
			/*
		     * Dimension: <number> <symbol>
			 */
			let mut inner = token.into_inner();
			Expr::Dimension(
				inner.next().unwrap().as_str().parse().unwrap(),
				inner.next().unwrap().as_str().into(),
			)
		},
		Rule::var => Expr::Variable(token.as_str().into()),
		Rule::interpolation => {
			/*
			 * Interop: <not_ws>? (<expr> <not_ws>?)+
			 */
			Expr::Interpolation(
				token.into_inner()
					.map(
						|t| match t.as_rule() {
							Rule::surround => Expr::Keyword(t.as_str().into()),
							_ => parse_expr(t)
						}
					)
					.collect()
			)
		},
		Rule::tuple => Expr::Tuple(token.into_inner().map(parse_expr).collect()),
		Rule::object_accessor => {
			/*
			 * Object-accessor: <var> <symbol>+
			 */
			let mut inner = token.into_inner();
			Expr::ObjectAccessor(
				Box::new(parse_expr(inner.next().unwrap())),
				inner
					.map(|t| t.as_str().to_string())
					.collect(),
			)
		},
		Rule::object => {
			/*
			 * Object: (<symbol> <expr>)*
			 */
			Expr::Object(
				token.into_inner()
					.collect::<Vec<Pair<Rule>>>()
					.chunks(2)
					.map(|c| Variable {
						name: c[0].as_str().to_string(),
						expr: parse_expr(c[1].clone()),
					})
					.collect()
			)
		},
		Rule::array_accessor => {
			/*
			 * Array-accessor: <var> <expr>
			 */
			let mut inner = token.into_inner();
			Expr::ArrayAccessor(
				Box::new(parse_expr(inner.next().unwrap())),
				Box::new(parse_expr(inner.next().unwrap())),
			)
		},
		Rule::array => {
			/*
			 * Array: <expr>+
			 */
			Expr::Array(
				token.into_inner()
					.map(parse_expr)
					.collect()
			)
		},
		Rule::function_call => {
			/*
			 * Function-call: <func> <array>
			 */
			let mut inner = token.into_inner();
			Expr::FunctionCall(
				inner.next().unwrap().as_str().into(),
				inner.next().unwrap()
					.into_inner()
					.map(parse_expr)
					.collect(),
			)
		},
		Rule::op_1 |
		Rule::op_2 |
		Rule::op_3 |
		Rule::op_4 => {
			/*
			 * Operation: <expr> <op_symbol> <expr>
			 */
			let mut inner = token.into_inner();
			let a = Box::new(parse_expr(inner.next().unwrap()));
			Expr::Operation(
				inner.next().unwrap().as_str().into(),
				a,
				Box::new(parse_expr(inner.next().unwrap())),
			)
		},
		_ => unreachable!(),
	}
}

/*
 * Parses token into Line enum
 */
fn parse_line(token: Pair<Rule>) -> Line {
	let line = token.into_inner().next().unwrap();
	match line.as_rule() {
		Rule::var_def => {
			/*
			 * Var-def: <symbol> <expr>
			 */
			let mut inner = line.into_inner();
			Line::VarDef(
				inner.next().unwrap().as_str().into(),
				parse_expr(inner.next().unwrap()),
			)
		},
		Rule::for_loop => {
			/*
			 * For-loop: <var> <expr> <line>+
			 */
			let mut inner = line.into_inner();
			Line::ForLoop(
				inner.next().unwrap().as_str().into(),
				parse_expr(inner.next().unwrap()),
				inner
					.map(parse_line)
					.collect()
			)
		},
		Rule::ret => {
			/*
			 * Return: <expr>
			 */
			Line::Return(
				parse_expr(line.into_inner().next().unwrap()),
			)
		},
		_ => unreachable!(),
	}
}

/*
 * Selector: <sel>+ <line>* <property|mixin_call|selector>+
 */
fn parse_selector(token: Pair<Rule>) -> Selector {
	let mut inner = token.into_inner().peekable();

	// <sel>+
	let sels = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::sel)
		.map(|p| p.as_str().to_owned())
		.collect();

	// <line>*

	let lines = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::line)
		.map(parse_line)
		.collect();

	// <property|mixin_call|selector>+

	let mut props = Vec::<Property>::new();
	let mut nested = Vec::<Selector>::new();

	inner.for_each(
		|pair| {
			match pair.as_rule() {
				Rule::property => {
					/*
				     * Property: <symbol> <expr>
					 */
					let mut inner = pair.into_inner();
					props.push(Property {
						name: inner.next().unwrap().as_str().into(),
						expr: parse_expr(inner.next().unwrap()),
					});
				},
				Rule::mixin_call => {
					/*
				     * Mixin-call: <func> <array>
					 */
					let mut inner = pair.into_inner();
					let name = inner.next().unwrap().as_str().to_owned();
					let mut arr = inner.next().unwrap().into_inner();
					props.push(Property {
						name,
						expr: match arr.clone().count() {
							1 => parse_expr(arr.next().unwrap()),
							_ => Expr::Tuple(arr.map(parse_expr).collect()),
						},
					});
				},
				Rule::selector => nested.push(parse_selector(pair)),
				_ => unreachable!(),
			}
		}
	);

	Selector {
		sels,
		lines,
		props,
		nested,
	}
}

/*
 * Mixin: <func> <symbol>* <line>* <property>+
 */
fn parse_mixin(token: Pair<Rule>) -> Mixin {
	let mut inner = token.into_inner().peekable();

	// <func>
	let name = inner.next().unwrap().as_str();

	// <symbol>*
	let params = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::symbol)
		.map(|p| p.as_str().to_owned())
		.collect();

	// <line>*

	let lines = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::line)
		.map(parse_line)
		.collect();
		
	// <property|mixin_call>+
	let props = inner.map(
		|pair| {
			match pair.as_rule() {
				Rule::property => {
					/*
				     * Property: <symbol> <expr>
					 */
					let mut inner = pair.into_inner();
					Property {
						name: inner.next().unwrap().as_str().into(),
						expr: parse_expr(inner.next().unwrap()),
					}
				},
				Rule::mixin_call => {
					/*
				     * Mixin-call: <func> <expr>*
					 */
					let mut inner = pair.into_inner();
					Property {
						name: inner.next().unwrap().as_str().into(),
						expr: match inner.clone().count() {
							1 => parse_expr(inner.next().unwrap()),
							_ => Expr::Tuple(inner.map(parse_expr).collect()),
						},
					}
				},
				_ => unreachable!(),
			}
		}
	).collect();

	Mixin {
		name: name.into(),
		params,
		lines,
		props,
	}
}

/*
 * Function: <func> <symbol>* <line>+
 */
fn parse_function(token: Pair<Rule>) -> Function {
	let mut inner = token.into_inner().peekable();

	// <function>
	let name = inner.next().unwrap().as_str();

	// <symbol>*
	let params = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::symbol)
		.map(|p| p.as_str().to_owned())
		.collect();

	// <line>+

	let lines = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::line)
		.map(parse_line)
		.collect();

	Function {
		name: name.into(),
		params,
		lines,
	}
}

/*
 * Generates AST from parsed tokens
 */
pub fn parse(tokens: Pairs<Rule>) -> Vec<Node> {
	tokens.map(
		|token| {
			match token.as_rule() {
				Rule::multi_line_comment => Node::Comment(token.as_str().into()),
				Rule::line => Node::Line(parse_line(token)),
				Rule::selector => Node::Selector(parse_selector(token)),
				Rule::mixin => Node::Mixin(parse_mixin(token)),
				Rule::function => Node::Function(parse_function(token)),
				Rule::EOI => Node::EOI,
				_ => unreachable!(),
			}
		}
	).collect()
}
