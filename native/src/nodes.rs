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

#[derive(Debug, Clone)]
pub enum Expr {
	Variable(String),
	Value(Box<Value>),
}

#[derive(Debug, Clone)]
pub struct Property {
	pub name: String,
	pub value: Value,
}

#[derive(Debug, Clone)]
pub struct MixinCall {
	pub name: String,
	pub args: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct Selector {
	pub sels: Vec<String>,
	pub props: Vec<Property>,
	pub calls: Vec<MixinCall>,
	pub nested: Vec<Selector>,
}

#[derive(Debug, Clone)]
pub struct Mixin {
	pub name: String,
	pub params: Vec<String>,
	pub props: Vec<Property>,
}

#[derive(Debug)]
pub struct Variable {
	pub name: String,
	pub expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Node {
	Comment(String),
	Selector(Selector),
	Mixin(Mixin),
	EOI,
}
