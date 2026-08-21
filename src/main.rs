use std::fs;

use crate::{eval::Eval, parser::Parse, token::Lexer};

mod error;
mod token;
mod parser;
mod eval;

fn main() {
    let src = fs::read_to_string("test.ca").unwrap();
    let tokens = Lexer::new(&src).tokenize().unwrap();
    let parse = Parse::new(tokens);
    let ast = match parse.parse_program() {
        Ok(ast) => ast,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    println!("{:?}", ast);

    let mut eval = Eval::new(ast);
    eval.eval();
    /*for t in lexer.tokenize().unwrap() {
        print!("{:?}({}, {}, {}) ", t.token_type, t.lexeme, t.line, t.column);
    }*/
}
