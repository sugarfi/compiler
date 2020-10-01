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
 * Parses token into Value enum
 */
fn parse_value(token: Pair<Rule>) -> Value {
	match token.as_rule() {
		Rule::symbol => Value::Keyword(token.as_str().into()),
		Rule::hash => Value::Hash(token.as_str().into()),
		Rule::number => Value::Number(token.as_str().parse().unwrap()),
		Rule::single_string |
		Rule::double_string => Value::String(token.as_str().into()),
		Rule::dimension => {
			/*
		     * Dimension: <number> <symbol>
			 */
			let mut inner = token.into_inner();
			Value::Dimension(
				inner.next().unwrap().as_str().parse().unwrap(),
				inner.next().unwrap().as_str().into(),
			)
		},
		Rule::var => Value::Variable(token.as_str().into()),
		Rule::interpolation => {
			/*
			 * Interop: <not_ws>? (<expr> <not_ws>?)+
			 */
			Value::Interpolation(
				token.into_inner()
					.map(
						|t| match t.as_rule() {
							Rule::surround => Expr::Value(Box::new(Value::Keyword(t.as_str().into()))),
							_ => parse_expr(t)
						}
					)
					.collect()
			)
		},
		Rule::tuple => Value::Tuple(token.into_inner().map(parse_value).collect()),
		_ => unreachable!(),
	}
}

/*
 * Parses token into Expr enum
 */
fn parse_expr(token: Pair<Rule>) -> Expr {
	match token.as_rule() {
		Rule::accessor => {
			/*
			 * Accessor: <var> <symbol>+
			 */
			let mut inner = token.into_inner();
			Expr::Accessor(
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
		_ => Expr::Value(Box::new(parse_value(token))),
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
		_ => unreachable!(),
	}
}

/*
 * Selector: <sel>+ <line>* <property|mixin_call|subsel>+
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

	// <property|mixin_call|subsel>+

	let mut props = Vec::<Property>::new();
	let mut nested = Vec::<Selector>::new();

	inner.for_each(
		|pair| {
			match pair.as_rule() {
				Rule::property => {
					/*
				     * Property: <symbol> <value>
					 */
					let mut inner = pair.into_inner();
					props.push(Property {
						name: inner.next().unwrap().as_str().into(),
						value: parse_value(inner.next().unwrap()),
					});
				},
				Rule::mixin_call => {
					/*
				     * Mixin-call: <function> <value>*
					 */
					let mut inner = pair.into_inner();
					props.push(Property {
						name: inner.next().unwrap().as_str().into(),
						value: match inner.clone().count() {
							1 => parse_value(inner.next().unwrap()),
							_ => Value::Tuple(inner.map(parse_value).collect()),
						},
					});
				},
				Rule::subsel => {
					/*
					 * Sub-selector: <sel>+ <property|mixin_call|subsel>*
					 */
					nested.push(parse_selector(pair));
				},
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
 * Mixin: <function> <symbol>* <line>* <property>+
 */
fn parse_mixin(token: Pair<Rule>) -> Mixin {
	let mut inner = token.into_inner().peekable();

	// <function>
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
				     * Property: <symbol> <value>
					 */
					let mut inner = pair.into_inner();
					Property {
						name: inner.next().unwrap().as_str().into(),
						value: parse_value(inner.next().unwrap()),
					}
				},
				Rule::mixin_call => {
					/*
				     * Mixin-call: <function> <value>*
					 */
					let mut inner = pair.into_inner();
					Property {
						name: inner.next().unwrap().as_str().into(),
						value: match inner.clone().count() {
							1 => parse_value(inner.next().unwrap()),
							_ => Value::Tuple(inner.map(parse_value).collect()),
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
 * Generates AST from parsed tokens
 */
pub fn parse(tokens: Pairs<Rule>) -> Vec<Node> {
	tokens.map(
		|token| {
			match token.as_rule() {
				Rule::multi_line_comment => Node::Comment(token.as_str().into()),
				Rule::selector => Node::Selector(parse_selector(token)),
				Rule::mixin => Node::Mixin(parse_mixin(token)),
				Rule::EOI => Node::EOI,
				_ => unreachable!(),
			}
		}
	).collect()
}
