use crate::analysis::tokenizer;

pub fn parse(query: &str) -> Vec<String> {
    tokenizer::analyze(query)    
}
