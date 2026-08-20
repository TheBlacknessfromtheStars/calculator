use std::fs;

use crate::{eval::Eval, parser::Parse, token::Lexer};

mod error;
mod token;
mod parser;
mod eval;

fn main() {
    let src = fs::read_to_string("/data/data/com.termux/files/home/calculator/test.ca").unwrap();

    let tokens = Lexer::new(&src).tokenize().unwrap();
    let parse = Parse::new(tokens);
    let ast = parse.parse_program().unwrap();

    let mut eval = Eval::new(ast);
    eval.eval();
    /*for t in lexer.tokenize().unwrap() {
        print!("{:?}({}, {}, {}) ", t.token_type, t.lexeme, t.line, t.column);
    }*/
}
