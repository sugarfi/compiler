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

use std::fs::read_dir;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    neon_build::setup();

    let destination = Path::new("./src").join("tests.rs");
    let mut test_file = File::create(&destination).unwrap();

    write_header(&mut test_file);

    read_dir("./tests").unwrap().for_each(
        |dir| write_test(&mut test_file, &dir.unwrap())
    );
}

fn write_test(test_file: &mut File, directory: &DirEntry) {
    let directory = directory.path().canonicalize().unwrap();

    write!(
        test_file,
r#"
#[test]
fn {name}() {{
	let source = fs::read_to_string("{path}/style.glz").unwrap();
	let expected_css = fs::read_to_string("{path}/style.css").unwrap();
	let expected_js = fs::read_to_string("{path}/style.js").unwrap();

	let tokens = tokenize(&source);
    let ast = parse(tokens);

    let mut generator = Generator::new();
    let (css, js) = generator.generate(ast);

    assert_eq!(css, &expected_css);
    assert_eq!(js, &expected_js);
}}
"#,
        name = directory.file_name().unwrap().to_string_lossy(),
        path = directory.display(),
    )
    .unwrap();
}

fn write_header(test_file: &mut File) {
    write!(
        test_file,
"use crate::tokenizer::tokenize;
use crate::parser::parse;
use crate::generator::Generator;
use std::fs;
",
    )
    .unwrap();
}

