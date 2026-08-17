use crate::error::{CalcError, CalcResult};
use crate::token::Class::Digit;
use crate::token::State::Minus;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash, // + - * /
    ParenthesisOpen,
    ParenthesisClose, // ( )
    Percent,
    Comma,
    Assign, // % , =
    Pow,
    Divisible, // **(乘方) //(整除)
    Indent,
    Value,
    NewLine,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    S,
    Int,
    Frac,
    Dot,
    Id,
    Plus,
    Minus,
    Mul,
    Div,
    Pow,
    Divisible,
    LParent,
    RParent,
    Rem,
    Comma,
    Assign,
    NewLine,
}

enum Class {
    Digit,
    Letter,
    Char(char),
}

fn classify(ch: char) -> Class {
    if ch.is_ascii_digit() {
        Digit
    } else if ch.is_ascii_alphabetic() {
        Class::Letter
    } else {
        Class::Char(ch)
    }
}

fn transition(state: State, cls: Class) -> Option<State> {
    use Class::*;
    use State::*;

    match (state, cls) {
        (S, Digit) => Some(Int),
        (S, Letter) => Some(Id),
        (S, Char('+')) => Some(Plus),
        (S, Char('-')) => Some(Minus),
        (S, Char('*')) => Some(Mul),
        (S, Char('/')) => Some(Div),
        (S, Char('(')) => Some(LParent),
        (S, Char(')')) => Some(RParent),
        (S, Char('%')) => Some(Rem),
        (S, Char(',')) => Some(Comma),
        (S, Char('=')) => Some(Assign),
        (S, Char('\n')) => Some(NewLine),
        (S, Char(c)) if c.is_whitespace() => Some(S),

        (Minus, Digit) => Some(Int),
        (Plus, Digit) => Some(Int),
        (Int, Digit) => Some(Int),
        (Int, Char('.')) => Some(Dot),
        (Dot, Digit) => Some(Frac),
        (Frac, Digit) => Some(Frac),

        (Mul, Char('*')) => Some(Pow),
        (Div, Char('/')) => Some(Divisible),

        _ => None,
    }
}

fn is_accepting(state: State) -> bool {
    use State::*;

    !matches!(state, S | Dot)
}

fn token_type_from_state(state: State) -> TokenType {
    match state {
        State::Int | State::Frac => TokenType::Value,
        State::Id => TokenType::Indent,
        State::Plus => TokenType::Plus,
        Minus => TokenType::Minus,
        State::Mul => TokenType::Star,
        State::Div => TokenType::Slash,
        State::Pow => TokenType::Pow,
        State::Divisible => TokenType::Divisible,
        State::LParent => TokenType::ParenthesisOpen,
        State::RParent => TokenType::ParenthesisClose,
        State::Rem => TokenType::Percent,
        State::Comma => TokenType::Comma,
        State::Assign => TokenType::Assign,
        State::NewLine => TokenType::NewLine,

        _ => unreachable!("bad state {:?}", state),
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) {
        self.pos += 1;
    }

    pub fn tokenize(mut self) -> CalcResult<Vec<Token>> {
        let mut tokens = Vec::new();
        let n = self.chars.len();

        let mut number_line: usize = 0;
        let mut column: usize = 0;

        while self.pos < n {
            let start = self.pos;
            let mut state = State::S;
            let mut last_accept: Option<(State, usize)> = None;

            while let Some(c) = self.peek() {
                if c == '\n' {
                    number_line += 1;
                    column = 0;
                } else {
                    column += 1;
                }

                match transition(state, classify(c)) {
                    Some(next) => {
                        state = next;
                        self.bump();

                        if is_accepting(state) {
                            last_accept = Some((state, self.pos));
                        }
                    }

                    None => break,
                }
            }

            match last_accept {
                Some((state, end)) => {
                    let lexeme: String = self.chars[start..end].iter().collect();
                    let lexeme = lexeme.as_str();
                    tokens.push(Token {
                        token_type: token_type_from_state(state),
                        lexeme: lexeme.trim().to_string(),
                        line: number_line,
                        column,
                    });
                    self.pos = end;
                }

                None => return Err(CalcError::Lex(String::from("unexpected EOF"))),
            }
        }

        Ok(tokens)
    }
}
