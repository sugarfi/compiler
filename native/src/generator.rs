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

pub struct Generator {
	css: String,
	js: String,
}

impl<'a> Generator {
	#[inline]
	pub fn new() -> Generator {
		Generator {
			css: "".into(),
			js: "".into(),
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
	fn generate_node(&mut self, node: &Node<'a>) {
		match node {
			Node::Selector(selector) => self.gen_selector(selector),
			Node::Comment(comment) => self.gen_comment(comment),
		}
	}

	#[inline]
	fn gen_selector(&mut self, selector: &Selector<'a>) {
		let sels = selector.sels.join(",\n");

		let props = selector.props.iter().map(
			|prop| format!("\t{}: {};\n", prop.name, self.gen_value(&prop.value))
		).collect::<String>();

		self.css += &format!("{} {{\n{}}}\n\n", sels, props);

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
				nested: child.nested.clone(),
			});
		}
	}

	#[inline]
	fn gen_comment(&mut self, comment: &CowRcStr<'a>) {
		self.css += &format!("/*{}*/\n\n", comment);
	}

	#[inline]
	fn gen_value(&self, value: &Value<'a>) -> String {
		match value {
			Value::Keyword(kw) => kw.to_string(),
			Value::Number(n) => n.to_string(),
			Value::String(s) => format!("\"{}\"", s),
			Value::Dimension(v, u) => format!("{}{}", v, u),
		}
	}
}
