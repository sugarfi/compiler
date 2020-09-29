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

use crate::nodes::*;
use cow_rc_str::CowRcStr;

pub struct Generator<'a> {
	css: String,
	js: String,
	mixins: Vec<&'a Mixin<'a>>,
	vars: Vec<Vec<Variable<'a>>>,
}

impl<'a> Generator<'a> {
	#[inline]
	pub fn new() -> Generator<'a> {
		Generator {
			css: "".into(),
			js: "".into(),
			mixins: Vec::new(),
			vars: Vec::new(),
		}
	}

	#[inline]
	pub fn generate(&mut self, nodes: &'a [Node<'a>]) -> (&str, &str) {
		for node in nodes {
			self.generate_node(node);
		}

		(&self.css, &self.js)
	}

	#[inline]
	fn generate_node(&mut self, node: &'a Node<'a>) {
		match node {
			Node::Comment(comment) => self.css += &format!("{}\n\n", comment),
			Node::Selector(selector) => self.gen_selector(selector),
			Node::Mixin(mixin) => self.mixins.push(mixin),
			Node::EOI => (),
		}
	}

	#[inline]
	fn find_mixin(&self, name: &CowRcStr<'a>) -> Option<&'a Mixin<'a>> {
		match self.mixins.iter().find(|mixin| mixin.name == name) {
			None => None,
			Some(mixin) => Some(*mixin),
		}
	}

	#[inline]
	fn get_var(&self, name: &CowRcStr<'a>) -> Value<'a> {
		match self.vars.iter().find(
			|scope| scope.iter().find(
				|var| var.name == name
			).is_some()
		) {
			None => panic!("Variable {} not defined", name),
			Some(scope) => self.get_expr(&scope.iter().find(|var| var.name == name).unwrap().expr),
		}
	}

	#[inline]
	fn get_expr(&self, expr: &Expr<'a>) -> Value<'a> {
		match expr {
			Expr::Variable(var) => self.get_var(var),
			Expr::Value(val) => *val.clone(),
		}
	}

	#[inline]
	fn get_mixin_props(&mut self, name: &CowRcStr<'a>, args: &Vec<Value<'a>>) -> Option<String> {
		println!("{:?}", args);
		match self.find_mixin(name) {
			None => None,
			Some(mixin) => {
				self.vars.push(mixin.params
					.iter()
					.enumerate()
					.map(
						|(i, param)| Variable {
							name: param.clone(),
							expr: Expr::Value(Box::new(args.get(i).expect("Not enough arguments").clone())),
						}
					).collect::<Vec<Variable<'a>>>()
				);

				let props = mixin.props.iter().map(
					|prop| {
						let args = match &prop.value {
							Value::Tuple(tup) => (*tup).to_vec(),
							_ => vec![prop.value.clone()],
						};

						match self.get_mixin_props(
							&prop.name, 
							&args,
						) {
							None => format!("\t{}: {};\n", prop.name, self.gen_value(&prop.value)),
							Some(props) => props,
						}
					}
				).collect::<String>();

				self.vars.pop();

				Some(props)
			},
		}
	}

	#[inline]
	fn gen_selector(&mut self, selector: &Selector<'a>) {
		let sels = selector.sels.join(",\n");

		let props = selector.props.iter().map(
			|prop| {
				let args = match &prop.value {
					Value::Tuple(tup) => (*tup).to_vec(),
					_ => vec![prop.value.clone()],
				};

				match self.get_mixin_props(
					&prop.name, 
					&args,
				) {
					None => format!("\t{}: {};\n", prop.name, self.gen_value(&prop.value)),
					Some(props) => props,
				}
			}
		).collect::<String>();

		let calls = selector.calls.iter().map(
			|call| self
				.get_mixin_props(&call.name, &call.args)
				.expect(&format!("Could not find mixin: {}", call.name))
		).collect::<String>();

		if props.len() > 0 || calls.len() > 0 {
			self.css += &format!("{} {{\n{}{}}}\n\n", sels, props, calls);
		}

		for child in &selector.nested {
			let mut child_sels = Vec::new();

			for child_sel in &child.sels {
				for sel in &selector.sels {
					child_sels.push(format!("{} {}", sel, child_sel).into());
				}
			}

			self.gen_selector(&Selector {
				sels: child_sels,
				props: child.props.clone(),
				calls: child.calls.clone(),
				nested: child.nested.clone(),
			});
		}
	}

	#[inline]
	fn gen_value(&self, value: &Value<'a>) -> String {
		match value {
			Value::Keyword(kw) => kw.to_string(),
			Value::Number(n) => n.to_string(),
			Value::String(s) => format!("\"{}\"", s),
			Value::Hash(h) => format!("#{}", h),
			Value::Dimension(v, u) => format!("{}{}", v, u),
			Value::Interop(expr) => self.gen_value(&self.get_expr(expr)),
			Value::Tuple(tup) => (*tup).iter().map(|v| self.gen_value(v)).collect::<Vec<String>>().join(" "),
		}
	}
}
