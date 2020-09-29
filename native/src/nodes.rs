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

use cow_rc_str::CowRcStr;

#[derive(Debug, Clone)]
pub enum Value<'a> {
	Keyword(CowRcStr<'a>),
	Hash(CowRcStr<'a>),
	Number(f32),
	String(CowRcStr<'a>),
	Dimension(f32, CowRcStr<'a>),
	Interop(Expr<'a>),
	Tuple(Vec<Value<'a>>),
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
	Variable(CowRcStr<'a>),
	Value(Box<Value<'a>>),
}

#[derive(Debug, Clone)]
pub struct Property<'a> {
	pub name: CowRcStr<'a>,
	pub value: Value<'a>,
}

#[derive(Debug, Clone)]
pub struct MixinCall<'a> {
	pub name: CowRcStr<'a>,
	pub args: Vec<Value<'a>>,
}

#[derive(Debug, Clone)]
pub struct Selector<'a> {
	pub sels: Vec<CowRcStr<'a>>,
	pub props: Vec<Property<'a>>,
	pub calls: Vec<MixinCall<'a>>,
	pub nested: Vec<Selector<'a>>,
}

#[derive(Debug)]
pub struct Mixin<'a> {
	pub name: CowRcStr<'a>,
	pub params: Vec<CowRcStr<'a>>,
	pub props: Vec<Property<'a>>,
}

#[derive(Debug)]
pub struct Variable<'a> {
	pub name: CowRcStr<'a>,
	pub expr: Expr<'a>,
}

#[derive(Debug)]
pub enum Node<'a> {
	Comment(CowRcStr<'a>),
	Selector(Selector<'a>),
	Mixin(Mixin<'a>),
	EOI,
}
