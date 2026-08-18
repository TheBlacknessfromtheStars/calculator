use crate::token::Lexer;

mod error;
mod token;
mod parser;

fn main() {
    let src = String::from("f(x, a)=  +1 ** -9 + +2.1*-3.2\n 3 + 1");

    let lexer = Lexer::new(&src);
    for t in lexer.tokenize().unwrap() {
        print!("{:?}({}, {}, {}) ", t.token_type, t.lexeme, t.line, t.column);
    }
}
