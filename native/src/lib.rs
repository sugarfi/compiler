use neon::prelude::*;
use std::{
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
};

/* Recursively copies an entire directory */
pub fn copy(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let from_str = cx.argument::<JsString>(0)?.value();
    let to_str = cx.argument::<JsString>(1)?.value();
    let from = Path::new(&from_str);
    let to = Path::new(&to_str);

    let mut stack = Vec::new();
    stack.push(PathBuf::from(&from));

    let output_root = PathBuf::from(&to);
    let input_root = PathBuf::from(&from).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest).unwrap();
        }

        for entry in fs::read_dir(working_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path).unwrap();
                    }
                    None => panic!("failed: {:?}", path),
                }
            }
        }
    }

    Ok(cx.undefined())
}

/* Compiles source code */
fn compile(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value();

    let file = File::open(&input).expect(&format!("Could not open file at {}", input));

    let mut buf_reader = BufReader::new(file);
    let mut source = String::new();
    buf_reader.read_to_string(&mut source).expect("Could not read the file.");

    Ok(cx.string(source))
}

register_module!(mut cx, {
    cx.export_function("copy", copy)?;
    cx.export_function("compile", compile)?;
    Ok(())
});
