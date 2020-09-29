use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use glaze::tokenizer::tokenize;
use glaze::parser::parse;
use glaze::generator::Generator;

fn criterion_benchmark(c: &mut Criterion) {
	let mut g = c.benchmark_group("sample-size-example");
    g.sample_size(5000);

	let source = fs::read_to_string("benches/test.glz").unwrap();

    g.bench_function("tokenize", |b| b.iter(|| tokenize(black_box(&source))));

    let tokens = tokenize(&source);

    g.bench_function("parse", |b| b.iter(|| parse(black_box(tokens.clone()))));

    let ast = parse(tokens);

    g.bench_function("generate", |b| b.iter(|| {
    	let mut generator = Generator::new();
    	generator.generate(black_box(ast.clone()));
    }));

    g.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
