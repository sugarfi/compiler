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
 * A single glaze expression
 */
#[derive(Debug, Clone)]
pub enum Expr {
	Keyword(String),
	Hash(String),
	Number(f32),
	String(String),
	Dimension(f32, String),
	Variable(String),
	Interpolation(Vec<Expr>),
	Tuple(Vec<Expr>),
	ObjectAccessor(Box<Expr>, Vec<String>),
	Object(Vec<Variable>),
	ArrayAccessor(Box<Expr>, Box<Expr>),
	Array(Vec<Expr>),
	Operation(String, Box<Expr>, Box<Expr>),
	FunctionCall(String, Vec<Expr>),
}

/*
 * A line of code
 */
#[derive(Debug, Clone)]
pub enum Line {
	VarDef(String, Expr),
	ForLoop(String, Expr, Vec<Line>),
	Return(Expr),
}

/*
 * A CSS property
 */
#[derive(Debug, Clone)]
pub struct Property {
	pub name: String,
	pub expr: Expr,
}

/*
 * A selector block
 */
#[derive(Debug, Clone)]
pub struct Selector {
	pub sels: Vec<String>,
	pub lines: Vec<Line>,
	pub props: Vec<Property>,
	pub nested: Vec<Selector>,
}

/*
 * A mixin definition
 */
#[derive(Debug, Clone)]
pub struct Mixin {
	pub name: String,
	pub params: Vec<String>,
	pub lines: Vec<Line>,
	pub props: Vec<Property>,
}

/*
 * A function definition
 */
#[derive(Debug, Clone)]
pub struct Function {
	pub name: String,
	pub params: Vec<String>,
	pub lines: Vec<Line>,
}

/*
 * Variable stored in memory
 */
#[derive(Debug, Clone)]
pub struct Variable {
	pub name: String,
	pub expr: Expr,
}

/*
 * A root node
 */
#[derive(Debug, Clone)]
pub enum Node {
	Comment(String),
	Line(Line),
	Selector(Selector),
	Mixin(Mixin),
	Function(Function),
	EOI,
}
