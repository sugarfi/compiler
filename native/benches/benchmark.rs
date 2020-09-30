use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{env, fs};
use glaze::tokenizer::tokenize;
use glaze::parser::parse;
use glaze::generator::Generator;

fn criterion_benchmark(c: &mut Criterion) {
    let args: Vec<String> = 
        if env::args().len() > 2 {
            env::args().skip(1).collect()
        } else {
            vec!["tokenize", "parse", "generate"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        };

	let mut g = c.benchmark_group("sample-size-example");
    g.sample_size(320);

	let source = fs::read_to_string("benches/test.glz").unwrap();

    if args.iter().any(|a| a == "tokenize") {
        g.bench_function("tokenize", |b| b.iter(|| tokenize(black_box(&source))));
    }

    let tokens = tokenize(&source);

    if args.iter().any(|a| a == "parse") {
        g.bench_function("parse", |b| b.iter(|| parse(black_box(tokens.clone()))));
    }

    let ast = parse(tokens);

    if args.iter().any(|a| a == "generate") {
         g.bench_function("generate", |b| b.iter(|| {
            let mut generator = Generator::new();
            generator.generate(black_box(ast.clone()));
        }));
    }

    g.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
