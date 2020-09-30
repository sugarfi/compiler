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
		Rule::keyword => Value::Keyword(token.as_str().into()),
		Rule::hash => Value::Hash(token.as_str().into()),
		Rule::number => Value::Number(token.as_str().parse().unwrap()),
		Rule::single_string |
		Rule::double_string => Value::String(token.as_str().into()),
		Rule::dimension => {
			/*
		     * Dimension: <number> <keyword>
			 */
			let mut inner = token.into_inner();
			Value::Dimension(
				inner.next().unwrap().as_str().parse().unwrap(),
				inner.next().unwrap().as_str().into(),
			)
		},
		Rule::interpolation => Value::Interop(parse_expr(token.into_inner().next().unwrap())),
		Rule::tuple => Value::Tuple(token.into_inner().map(parse_value).collect()),
		_ => unreachable!(),
	}
}

/*
 * Parses token into Expr enum
 */
fn parse_expr(token: Pair<Rule>) -> Expr {
	match token.as_rule() {
		Rule::keyword => Expr::Variable(token.as_str().into()),
		_ => Expr::Value(Box::new(parse_value(token))),
	}
}

/*
 * Selector: <sel>+ <property|mixin_call|subsel>*
 */
fn parse_selector(token: Pair<Rule>) -> Selector {
	let mut inner = token.into_inner().peekable();

	// <sel>+
	let sels = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::sel)
		.map(|p| p.as_str().to_owned())
		.collect();

	// <property|mixin_call|subsel>*

	let mut props = Vec::<Property>::new();
	let mut calls = Vec::<MixinCall>::new();
	let mut nested = Vec::<Selector>::new();

	inner.for_each(
		|pair| {
			match pair.as_rule() {
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
				     * Mixin-call: <function> <value>*
					 */
					let mut inner = pair.into_inner();
					calls.push(MixinCall {
						name: inner.next().unwrap().as_str().into(),
						args: inner.map(parse_value).collect(),
					})
				},
				Rule::subsel => {
					/*
					 * Sub-selector: <sel>+ <property|mixin_call|subsel>*
					 */
					nested.push(parse_selector(pair))
				},
				_ => unreachable!(),
			}
		}
	);

	Selector {
		sels,
		props,
		calls,
		nested,
	}
}

/*
 * Mixin: <function> <keyword>* <property>*
 */
fn parse_mixin(token: Pair<Rule>) -> Mixin {
	let mut inner = token.into_inner().peekable();

	// <function>
	let name = inner.next().unwrap().as_str();

	// <keyword>*
	let params = inner
		.by_ref()
		.peeking_take_while(|p| p.as_rule() == Rule::keyword)
		.map(|p| p.as_str().to_owned())
		.collect();
		
	// <property>*
	let props = inner
		.map(|p| {
			let mut inner = p.into_inner();
			Property {
				name: inner.next().unwrap().as_str().into(),
				value: parse_value(inner.next().unwrap()),
			}
		})
		.collect();

	Mixin {
		name: name.into(),
		params,
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
