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
use std::fs;

macro_rules! test {
	($name:ident) => (
		#[test]
		fn $name() {
			let name = format!("src/tests/{}", stringify!($name));
			let source = fs::read_to_string(format!("{}/style.glz", name)).unwrap();
			let expected_css = fs::read_to_string(format!("{}/style.css", name)).unwrap();
			let expected_js = fs::read_to_string(format!("{}/style.js", name)).unwrap();

			let tokens = tokenize(&source);
		    let ast = parse(tokens);

		    let mut generator = Generator::new();
		    let (css, js) = generator.generate(ast);

		    assert_eq!(css, &expected_css);
		    assert_eq!(js, &expected_js);
		}
	)
}

test!(mixins);
test!(nesting);
test!(object);
test!(value);
test!(variables);
