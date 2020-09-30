use pest::{Parser, iterators::Pairs};

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct Tokenizer;

/*
 * Parses a source pest file
 */
pub fn tokenize(source: &str) -> Pairs<Rule> {
	match Tokenizer::parse(Rule::file, source) {
		Ok(tokens) => tokens,
		Err(err) => panic!("{:?}", err),
	}
}
