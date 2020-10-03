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

pub struct Generator {
	css: String,
	js: String,
	mixins: Vec<Mixin>,
	functions: Vec<Function>,
	stack: Vec<Vec<Variable>>, // Pushes to and pops from a scoped stack
	ret: Option<Expr>, // Currently returned expression
}

impl<'a> Generator {
	pub fn new() -> Self {
		Self {
			css: "".into(),
			js: "".into(),
			mixins: Vec::new(),
			functions: Vec::new(),
			stack: Vec::new(),
			ret: None,
		}
	}

	/*
	 * Generates CSS & JS from AST
	 */
	pub fn generate(&mut self, nodes: Vec<Node>) -> (&str, &str) {
		self.stack.push(Vec::new());

		nodes.iter().for_each(|node| self.generate_node(node));
		
		self.stack.pop();

		(&self.css, &self.js)
	}

	/*
	 * Generates CSS & JS from a single node
	 */
	fn generate_node(&mut self, node: &Node) {
		match node {
			Node::Comment(comment) => self.css += &format!("{}\n\n", comment),
			Node::Line(line) => self.eval_line(line),
			Node::Selector(selector) => self.gen_selector(selector),
			Node::Mixin(mixin) => self.mixins.push(mixin.clone()),
			Node::Function(function) => self.functions.push(function.clone()),
			Node::EOI => (),
		}
	}

	/*
	 * Finds a mixin in memory
	 */
	fn find_mixin(&self, name: &str) -> Option<Mixin> {
		match self.mixins.iter().find(|m| m.name == name) {
			None => None,
			Some(mixin) => Some(mixin.clone()),
		}
	}

	/*
	 * Finds a function in memory
	 */
	fn find_function(&self, name: &str) -> Option<Function> {
		match self.functions.iter().find(|f| f.name == name) {
			None => None,
			Some(function) => Some(function.clone()),
		}
	}

	/*
	 * Finds a variable in memory
	 */
	fn find_var(&self, name: &str) -> Expr {
		let var = self.stack.iter().rev().find_map(
			|scope| scope.iter().find(|v| v.name == name)
		);

		match var {
			None => panic!("Could not find variable: {}", name),
			Some(v) => v.expr.clone(),
		}
	}

	/*
	 * Finds a field of an object
	 */
	fn find_field(&mut self, props: &Vec<Variable>, fields: &Vec<String>, i: usize) -> Expr {
		match props.iter().find(|p| &p.name == fields.get(i).unwrap()) {
			None => panic!("Could not find field {}", fields.get(i).unwrap()),
			Some(var) => match &var.expr {
				Expr::Object(props) => self.find_field(props, fields, i + 1),
				_ => self.eval_expr(&var.expr),
			},
		}
	}

	/*
	 * Calls a function
	 */
	fn call_function(&mut self, name: &str, args: &Vec<Expr>) -> Expr {
		match self.find_function(name) {
			None => panic!("Could not find function {}", name),
			Some(function) => {
				// Adds arguments to scope
				self.stack.push(
					function.params
						.iter()
						.enumerate()
						.map(
							|(i, param)| Variable {
								name: param.to_owned(),
								expr: args
									.get(i)
									.expect("Not enough arguments")
									.clone(),
							}
						)
						.collect()
				);

				// Executes lines of code
				for line in &function.lines {
					match &self.ret {
						None => self.eval_line(line),
						Some(_) => break,
					}
				}

				self.stack.pop();

				// Return expression
				let ret = self.ret.clone();
				self.ret = None;

				match ret {
					None => panic!("Function {} missing return statement", name),
					Some(e) => e.clone(),
				}
			},
		}
	}

	/*
	 * Evaluates a composite expression into a base expression
	 */
	fn eval_expr(&mut self, expr: &Expr) -> Expr {
		match expr {
			Expr::ObjectAccessor(obj, fields) => {
				let props = match &**obj {
					Expr::Variable(var) => match self.find_var(&var) {
						Expr::Object(props) => props,
						_ => panic!("{} is not an object", var),
					},
					_ => unreachable!(),
				};

				let expr = self.find_field(&props, fields, 0);
				self.eval_expr(&expr)
			},
			Expr::ArrayAccessor(arr, index) => {
				let arr = match &**arr {
					Expr::Variable(var) => match self.find_var(&var) {
						Expr::Array(arr) => arr,
						_ => panic!("{} is not an array", var),
					},
					_ => unreachable!(),
				};

				let index = match self.eval_expr(index) {
					Expr::Number(n) => n as usize,
					_ => panic!("Only numbers can be used to index arrays"),
				};

				let expr = match arr.get(index) {
					None => panic!("Index {} is invalid", index),
					Some(expr) => expr,
				};

				self.eval_expr(&expr)
			},
			Expr::Variable(var) => self.find_var(var),
			Expr::FunctionCall(name, args) => self.call_function(name, args),
			Expr::Operation(op, a, b) => self.eval_operation(op, a, b),
			_ => expr.clone(),
		}
	}

	/*
	 * Evaluates a line of code
	 */
	fn eval_line(&mut self, line: &Line) {
		match line {
			Line::VarDef(name, expr) => {
				let name = name.to_string();
				let expr = self.eval_expr(expr);

				self.push_var(Variable {
					name,
					expr,
				})
			},
			Line::ForLoop(var, iter, lines) => {
				let iter = match self.eval_expr(iter) {
					Expr::Tuple(tup) => tup,
					Expr::Array(arr) => arr,
					_ => panic!("Not iterable"),
				};

				iter.iter().for_each(
					|item| {
						self.push_var(Variable {
							name: var.clone(),
							expr: item.clone(),
						});

						lines.iter().for_each(|l| self.eval_line(l));
					}
				);
			},
			Line::Return(expr) => {
				self.ret = Some(self.eval_expr(expr));
			},
		}
	}

	/*
	 * Evaluates a binary operation
	 */
	fn eval_operation(&mut self, op: &str, a: &Expr, b: &Expr) -> Expr {
		let a = self.eval_expr(a);
		let b = self.eval_expr(b);

		match op {
			"+" => match (a, b) {
				(Expr::Number(a), Expr::Number(b)) => Expr::Number(a + b),
				_ => panic!("Cannot use +"),
			},
			"-" => match (a, b) {
				(Expr::Number(a), Expr::Number(b)) => Expr::Number(a - b),
				_ => panic!("Cannot use -"),
			},
			"*" => match (a, b) {
				(Expr::Number(a), Expr::Number(b)) => Expr::Number(a * b),
				_ => panic!("Cannot use *"),
			},
			"/" => match (a, b) {
				(Expr::Number(a), Expr::Number(b)) => Expr::Number(a / b),
				_ => panic!("Cannot use /"),
			},
			".." => match (a, b) {
				(Expr::Number(a), Expr::Number(b)) => Expr::Array(
					((a as u32)..(b as u32))
						.map(|n| Expr::Number(n as f32))
						.collect()
				),
				_ => panic!("Cannot use .."),
			},
			"..=" => match (a, b) {
				(Expr::Number(a), Expr::Number(b)) => Expr::Array(
					((a as u32)..=(b as u32))
						.map(|n| Expr::Number(n as f32))
						.collect()
				),
				_ => panic!("Cannot use ..="),
			},
			"++" => match (a, b) {
				(Expr::Array(a), Expr::Array(b)) => {
					let mut a = a;
					let mut b = b;
					a.append(&mut b);
					Expr::Array(a)
				},
				(Expr::Array(a), b @ _) => {
					let mut a = a;
					a.push(b);
					Expr::Array(a)
				},
				(a @ _, Expr::Array(b)) => {
					let mut b = b;
					b.insert(0, a);
					Expr::Array(b)
				},
				(Expr::Tuple(a), Expr::Tuple(b)) => {
					let mut a = a;
					let mut b = b;
					a.append(&mut b);
					Expr::Tuple(a)
				},
				(Expr::Tuple(a), b @ _) => {
					let mut a = a;
					a.push(b);
					Expr::Tuple(a)
				},
				(a @ _, Expr::Tuple(b)) => {
					let mut b = b;
					b.insert(0, a);
					Expr::Tuple(b)
				},
				(Expr::String(a), Expr::String(b)) => {
					let mut a = a;
					a += &b;
					Expr::String(a)
				},
				_ => panic!("Cannot use ++"),
			},
			_ => unreachable!(),
		}
	}

	/*
	 * Pushes a variable into the current scope
	 */
	fn push_var(&mut self, var: Variable) {
		let scope = self.stack
			.last_mut()
			.unwrap();

		match scope.iter().find(|v| v.name == var.name) {
			None => scope.push(var),
			Some(_) => {
				scope.clear();
				scope.extend(
					scope.iter()
						.filter(|v| v.name != var.name)
						.chain(vec![var.clone()].iter())
						.map(|v| v.clone())
						.collect::<Vec<Variable>>()
				);
			},
		}
	}

	/*
	 * Generates properties from a mixin call
	 */
	fn get_mixin_props(&mut self, name: &str, args: &[Expr]) -> Option<String> {
		match self.find_mixin(name) {
			None => None,
			Some(mixin) => {
				// Adds arguments to scope
				self.stack.push(
					mixin.params
						.iter()
						.enumerate()
						.map(
							|(i, param)| Variable {
								name: param.to_owned(),
								expr: args
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
		let args = match &prop.expr {
			Expr::Tuple(tup) => (*tup).to_vec(),
			_ => vec![prop.expr.clone()],
		};

		// Checks if property name is an existing mixin
		match self.get_mixin_props(
			&prop.name, 
			&args,
		) {
			None => format!("\t{}: {};\n", prop.name, self.gen_expr(&prop.expr)),
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
	 * Generates CSS from a base expression
	 */
	fn gen_expr(&mut self, expr: &Expr) -> String {
		match expr {
			Expr::Keyword(kw) => kw.to_string(),
			Expr::Number(n) => n.to_string(),
			Expr::String(s) => format!("\"{}\"", s),
			Expr::Hash(h) => format!("#{}", h),
			Expr::Dimension(v, u) => format!("{}{}", v, u),
			Expr::Interpolation(exprs) => {
				exprs.iter()
					.map(|e| {
						let expr = self.eval_expr(e);
						self.gen_expr(&expr)
					})
					.collect()
			},
			Expr::Tuple(tup) => {
				tup
					.iter()
					.map(|e| self.gen_expr(e))
					.collect::<Vec<String>>()
					.join(" ")
			},
			Expr::Array(arr) => {
				arr
					.iter()
					.map(|e| self.gen_expr(e))
					.collect::<Vec<String>>()
					.join(", ")
			},
			Expr::Object(_) => panic!("Object types cannot be resolved to CSS"), // temp
			_ => {
				let expr = self.eval_expr(expr);
				self.gen_expr(&expr)
			},
		}
	}
}
