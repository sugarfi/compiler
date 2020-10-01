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

/*
 * Variable stored in memory
 */
#[derive(Debug)]
pub struct Variable {
	pub name: String,
	pub value: Value,
}

pub struct Generator {
	css: String,
	js: String,
	mixins: Vec<Mixin>,
	stack: Vec<Vec<Variable>>, // Pushes to and pops from a scoped stack
}

impl<'a> Generator {
	pub fn new() -> Self {
		Self {
			css: "".into(),
			js: "".into(),
			mixins: Vec::new(),
			stack: Vec::new(),
		}
	}

	/*
	 * Generates CSS & JS from AST
	 */
	pub fn generate(&mut self, nodes: Vec<Node>) -> (&str, &str) {
		nodes.iter().for_each(
			|node| {
				self.generate_node(&node);
			}
		);

		(&self.css, &self.js)
	}

	/*
	 * Generates CSS & JS from a single node
	 */
	fn generate_node(&mut self, node: &Node) {
		match node {
			Node::Comment(comment) => self.css += &format!("{}\n\n", comment),
			Node::Selector(selector) => self.gen_selector(selector),
			Node::Mixin(mixin) => self.mixins.push((*mixin).clone()),
			Node::EOI => (),
		}
	}

	/*
	 * Finds a mixin in memory
	 */
	fn find_mixin(&self, name: &str) -> Option<Mixin> {
		match self.mixins.iter().find(|mixin| mixin.name == name) {
			None => None,
			Some(mixin) => Some(mixin.clone()),
		}
	}

	/*
	 * Finds a variable in memory
	 */
	fn find_var(&self, name: &str) -> Value {
		let var = self.stack.iter().rev().find_map(
			|scope| scope.iter().find(|v| v.name == name)
		);

		match var {
			None => panic!("Could not find variable: {}", name),
			Some(v) => v.value.clone(),
		}
	}

	/*
	 * Evaluates an expression into a value
	 */
	fn eval_expr(&self, expr: &Expr) -> Value {
		match expr {
			Expr::Value(val) => *val.clone(),
		}
	}

	/*
	 * Evaluates a line of code
	 */
	fn eval_line(&mut self, line: &Line) {
		match line {
			Line::VarDef(name, expr) => {
				let value = self.eval_expr(expr);

				self.stack
					.last_mut()
					.unwrap()
					.push(Variable {
						name: name.to_string(),
						value,
					});
			}
		}
	}

	/*
	 * Generates properties from a mixin call
	 */
	fn get_mixin_props(&mut self, name: &str, args: &[Value]) -> Option<String> {
		match self.find_mixin(name) {
			None => None,
			Some(mixin) => {
				// Adds parameters to scope
				self.stack.push(
					mixin.params
						.iter()
						.enumerate()
						.map(
							|(i, param)| Variable {
								name: param.to_owned(),
								value: args
										.get(i)
										.expect("Not enough arguments")
										.clone(),
							}
						)
						.collect()
				);

				// Executes lines of code
				mixin.lines.iter().for_each(|l| self.eval_line(l));

				// Generates CSS from properties
				let props = mixin.props
								.iter()
								.map(|prop| self.gen_prop(prop))
								.collect::<String>();

				// Argument values no longer needed
				self.stack.pop();

				Some(props)
			},
		}
	}

	/*
	 * Generates CSS from a property node
	 */
	fn gen_prop(&mut self, prop: &Property) -> String {
		// Maps value to args vector
		let args = match &prop.value {
			Value::Tuple(tup) => (*tup).to_vec(),
			_ => vec![prop.value.clone()],
		};

		// Checks if property name is an existing mixin
		match self.get_mixin_props(
			&prop.name, 
			&args,
		) {
			None => format!("\t{}: {};\n", prop.name, self.gen_value(&prop.value)),
			Some(props) => props,
		}
	}

	/*
	 * Generates CSS from a selector node
	 */
	fn gen_selector(&mut self, selector: &Selector) {
		// Pushes a new stack and executes lines of code
		self.stack.push(Vec::new());
		selector.lines.iter().for_each(|l| self.eval_line(l));

		// Generates CSS from comma-separated selectors
		let sels = selector.sels.join(",\n");

		// Generates CSS from properties
		let props = selector.props
						.iter()
						.map(|prop| self.gen_prop(prop))
						.collect::<String>();

		// Generates CSS if the selector has properties
		if !props.is_empty() {
			self.css += &format!("{} {{\n{}}}\n\n", sels, props);
		}

		// Generates CSS for all nested selectors
		selector.nested.iter().for_each(
			|child| {
				let child_sels = child.sels.iter().flat_map(
					|child_sel| selector.sels.iter().map(
						move |sel| {
							if child_sel.contains('&') {
								child_sel.replace("&", sel)
							} else {
								format!("{} {}", sel, child_sel)
							}
						}
					)
				).collect();

				self.gen_selector(&Selector {
					sels: child_sels,
					lines: child.lines.clone(),
					props: child.props.clone(),
					nested: child.nested.clone(),
				});
			}
		);

		// Pops from stack
		self.stack.pop();
	}

	/*
	 * Generates CSS from a value node
	 */
	fn gen_value(&self, value: &Value) -> String {
		match value {
			Value::Keyword(kw) => kw.to_string(),
			Value::Number(n) => n.to_string(),
			Value::String(s) => format!("\"{}\"", s),
			Value::Hash(h) => format!("#{}", h),
			Value::Dimension(v, u) => format!("{}{}", v, u),
			Value::Variable(var) => self.gen_value(&self.find_var(var)),
			Value::Interpolation(exprs) => exprs.iter()
												.map(|e| self.gen_value(&self.eval_expr(e)))
												.collect(),
			Value::Tuple(tup) => (*tup)
									.iter()
									.map(|v| self.gen_value(v))
									.collect::<Vec<String>>()
									.join(" "),
		}
	}
}
