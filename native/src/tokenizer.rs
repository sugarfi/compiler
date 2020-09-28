use pest::{Parser, iterators::Pairs};

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct Tokenizer;

pub fn tokenize(source: &str) -> Pairs<Rule> {
	Tokenizer::parse(Rule::file, source).unwrap()
}
