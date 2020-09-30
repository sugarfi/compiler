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

/*
 * A single CSS value
 */
#[derive(Debug, Clone)]
pub enum Value {
	Keyword(String),
	Hash(String),
	Number(f32),
	String(String),
	Dimension(f32, String),
	Interop(Expr),
	Tuple(Vec<Value>),
}

/*
 * A single glaze expression
 */
#[derive(Debug, Clone)]
pub enum Expr {
	Variable(String),
	Value(Box<Value>),
}

/*
 * A CSS property
 */
#[derive(Debug, Clone)]
pub struct Property {
	pub name: String,
	pub value: Value,
}

/*
 * A function call
 */
#[derive(Debug, Clone)]
pub struct MixinCall {
	pub name: String,
	pub args: Vec<Value>,
}

/*
 * A selector block
 */
#[derive(Debug, Clone)]
pub struct Selector {
	pub sels: Vec<String>,
	pub props: Vec<Property>,
	pub calls: Vec<MixinCall>,
	pub nested: Vec<Selector>,
}

/*
 * A mixin definition
 */
#[derive(Debug, Clone)]
pub struct Mixin {
	pub name: String,
	pub params: Vec<String>,
	pub props: Vec<Property>,
}

/*
 * A root node
 */
#[derive(Debug, Clone)]
pub enum Node {
	Comment(String),
	Selector(Selector),
	Mixin(Mixin),
	EOI,
}
