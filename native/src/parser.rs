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

fn parse_value<'a>(token: Pair<'a, Rule>) -> Value<'a> {
	match token.as_rule() {
		Rule::keyword => Value::Keyword(token.as_str().into()),
		Rule::hash => Value::Hash(token.as_str().into()),
		Rule::number => Value::Number(token.as_str().parse().unwrap()),
		Rule::single_string |
		Rule::double_string => Value::String(token.as_str().into()),
		Rule::dimension => {
			let mut inner = token.into_inner();
			Value::Dimension(
				inner.next().unwrap().as_str().parse().unwrap(),
				inner.next().unwrap().as_str().into(),
			)
		},
		Rule::interpolation => Value::Interop(parse_expr(token.into_inner().next().unwrap())),
		Rule::tuple => Value::Tuple(token.into_inner().map(|v| parse_value(v)).collect()),
		_ => unreachable!(),
	}
}

fn parse_expr<'a>(token: Pair<'a, Rule>) -> Expr<'a> {
	match token.as_rule() {
		Rule::keyword => Expr::Variable(token.as_str().into()),
		_ => Expr::Value(Box::new(parse_value(token))),
	}
}

fn parse_selector<'a>(token: Pair<'a, Rule>) -> Selector<'a> {
	let mut sels = Vec::new();
	let mut props = Vec::new();
	let mut calls = Vec::new();
	let mut nested = Vec::new();

	for pair in token.into_inner() {
		match pair.as_rule() {
			Rule::sel => sels.push(pair.as_str().into()),
			Rule::property => {
				/*
			     * Property: <keyword> <value>
				 */
				let mut inner = pair.into_inner();
				props.push(Property {
					name: inner.next().unwrap().as_str().into(),
					value: parse_value(inner.next().unwrap()),
				});
			},
			Rule::mixin_call => {
				/*
			     * MixinCall: <function> <value>*
				 */
				let mut inner = pair.into_inner();
				calls.push(MixinCall {
					name: inner.next().unwrap().as_str().into(),
					args: inner.map(|arg| parse_value(arg)).collect(),
				})
			},
			Rule::subsel => nested.push(parse_selector(pair)),
			_ => unreachable!(),
		}
	}

	Selector {
		sels,
		props,
		calls,
		nested,
	}
}

fn parse_mixin<'a>(token: Pair<'a, Rule>) -> Mixin<'a> {
	let mut inner = token.into_inner();
	let name = inner.next().unwrap().as_str();
	let mut params = Vec::new();
	let mut props = Vec::new();

	for pair in inner {
		match pair.as_rule() {
			Rule::keyword => params.push(pair.as_str().into()),
			Rule::property => {
				/*
			     * Property: <keyword> <value>
				 */
				let mut inner = pair.into_inner();
				props.push(Property {
					name: inner.next().unwrap().as_str().into(),
					value: parse_value(inner.next().unwrap()),
				});
			},
			_ => unreachable!(),
		}
	}

	Mixin {
		name: name.into(),
		params,
		props,
	}
}

pub fn parse<'a>(tokens: Pairs<'a, Rule>) -> Vec<Node<'a>> {
	let mut nodes = Vec::new();

	for token in tokens {
		nodes.push(
			match token.as_rule() {
				Rule::multi_line_comment => Node::Comment(token.as_str().into()),
				Rule::selector => Node::Selector(parse_selector(token)),
				Rule::mixin => Node::Mixin(parse_mixin(token)),
				Rule::EOI => Node::EOI,
				_ => unreachable!(),
			}
		)
	}

	nodes
}
