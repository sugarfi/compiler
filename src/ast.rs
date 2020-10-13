/*
 *  Copyright (c) 2020, GiraffeKey
 *
 *  All rights reserved.
 *
 *  Redistribution and use in source and binary forms, with or without
 *  modification, are permitted provided that the following conditions are met:
 *
 *  Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 *
 *  Redistributions in binary form must reproduce the above
 *  copyright notice, this list of conditions and the following
 *  disclaimer in the documentation and/or other materials provided
 *  with the distribution.
 *
 *  Neither the name of GiraffeKey nor the names of other
 *  contributors may be used to endorse or promote products derived
 *  from this software without specific prior written permission.
 *
 *  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 *  "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 *  LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 *  A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 *  OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 *  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 *  LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 *  DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 *  THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 *  (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 *  OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use fnv::FnvHashMap;

#[allow(dead_code)] // Clear up a few warnings
#[derive(Debug)]
pub enum Type {
	Number,
	String,
	Hex,
	Dimension,
	Bool,
	Tuple(Vec<Type>),
	List(Box<Type>),
	Record(FnvHashMap<String, Type>),
	Function(Vec<Type>),
	Alias(String),
}

#[allow(dead_code)] // Clear up a few warnings
#[derive(Debug, Clone)]
pub enum Expr {
	Number(f32),
	String(String),
	Symbol(String),
	Hex(String),
	Dimension(f32, String),
	Bool(bool),
	Variable(String),
	Tuple(Vec<Expr>),
	List(Vec<Expr>),
	Record(FnvHashMap<String, Expr>),
	BinaryOp(String, Box<Expr>, Box<Expr>),
	UnaryOp(String, Box<Expr>),
	Call(String, Vec<Expr>),
	Index(Box<Expr>, Box<Expr>),
	If(Box<Expr>, Vec<Expr>, Vec<Expr>),
}

#[allow(dead_code)] // Clear up a few warnings
#[derive(Debug)]
pub enum Node {
	Selector(Vec<String>, Vec<Node>),
	Function(String, Vec<String>, Vec<Type>, Vec<Node>),
	Property(String, Vec<Expr>),
	Definition(String, Expr),
	Enum(String, Vec<String>),
	TypeAlias(String, Type),
	AtCSS(Expr),
	AtData(FnvHashMap<String, Expr>),
	AtEvent(Vec<Node>),
	Where(Vec<(String, Expr)>),
	Return(Expr),
	Expr(Expr),
}
