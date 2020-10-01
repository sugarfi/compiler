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

use crate::tokenizer::tokenize;
use crate::parser::parse;
use crate::generator::Generator;

fn generate(source: &str) -> (String, String) {
    let tokens = tokenize(source);
    let ast = parse(tokens);

    let mut generator = Generator::new();
    let (css, js) = generator.generate(ast);

    (css.to_string(), js.to_string())
}

/* Test template

#[test]
fn test_() {
	assert_eq!(
		generate(
"

"
		),
		(
"
".to_owned(),
"
".to_owned(),
		),
	);
}
 */

#[test]
fn test_nesting() {
	assert_eq!(
		generate(
"
.class
	span
		color: lightgray
	p
		color: gray
		b
			font-weight: 500
	div
		background-color: blue
"
		),
		(
".class span {
	color: lightgray;
}

.class p {
	color: gray;
}

.class p b {
	font-weight: 500;
}

.class div {
	background-color: blue;
}

".to_owned(),
"".to_owned(),
		),
	);
}

#[test]
fn test_mixins() {
	assert_eq!(
		generate(
"
color-weight(c, w)
	color: $c
	font-weight: $w

.class
	color-weight(blue, 600)
	p
		color-weight: #222 normal
		&:hover
			color: blue
"
		),
		(
".class {
	color: blue;
	font-weight: 600;
}

.class p {
	color: #222;
	font-weight: normal;
}

.class p:hover {
	color: blue;
}

".to_owned(),
"".to_owned(),
		),
	);
}

#[test]
fn test_value() {
	assert_eq!(
		generate(
"
.class
	font-weight: bold
	color: #222
	flex-grow: 2
	border: 1px solid black
	&:after
		content: \"\"
		background: blue
	width: 50vw
	height: 100%
	padding: 5px 2px
"
		),
		(
".class {
	font-weight: bold;
	color: #222;
	flex-grow: 2;
	border: 1px solid black;
	width: 50vw;
	height: 100%;
	padding: 5px 2px;
}

.class:after {
	content: \"\";
	background: blue;
}

".to_owned(),
"".to_owned(),
		),
	);
}

#[test]
fn test_variables() {
	assert_eq!(
		generate(
"
.class
	$width = 50px
	$height = 30px
	$border-style = solid

	width: $width
	height: $height
	border: 1px $border-style black
"
		),
		(
".class {
	width: 50px;
	height: 30px;
	border: 1px solid black;
}

".to_owned(),
"".to_owned(),
		),
	);
}
