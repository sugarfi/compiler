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

#[macro_use]
extern crate pest_derive;

mod tokenizer;
mod nodes;
mod parser;
mod generator;

#[cfg(test)]
mod tests;

use tokenizer::tokenize;
use parser::parse;
use generator::Generator;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
use neon::prelude::*;

fn compile(mut cx: FunctionContext) -> JsResult<JsObject> {
    let input = cx.argument::<JsString>(0)?.value();

    let file = File::open(&input).unwrap_or_else(|_| panic!("Could not open file at {}", input));

    let mut buf_reader = BufReader::new(file);
    let mut source = String::new();
    buf_reader.read_to_string(&mut source).expect("Could not read the file.");

    let tokens = tokenize(&source);
    let ast = parse(tokens);

    let mut generator = Generator::new();
    let (css, js) = generator.generate(&ast);

    let out = JsObject::new(&mut cx);
    let css = cx.string(css);
    let js = cx.string(js);
    out.set(&mut cx, "css", css).unwrap();
    out.set(&mut cx, "js", js).unwrap();

    Ok(out)
}

register_module!(mut cx, {
    cx.export_function("compile", compile)?;
    Ok(())
});
