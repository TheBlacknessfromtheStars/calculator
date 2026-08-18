use crate::error::{CalcError, CalcResult};
use crate::token::Class::Digit;
use crate::token::State::GotMinus;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash, // + - * /
    LParen,
    RParen, // ( )
    Percent,
    Comma,
    Assign, // % , =
    DoubleStar,
    DoubleSlash, // **(乘方) //(整除)
    Ident,
    Integer, 
    Decimal,
    NewLine,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    S,
    InInteger,
    InFraction,
    InIdentifier,

    GotDot,
    GotPlus,
    GotMinus,
    GotDoubleStar,
    GotDoubleSlash,
    GotLParen,
    GotRParent,
    GotPercent,
    GotComma,
    GotAssign,
    GotNewLine,

    SeenStar,
    SeenSlash,
}

enum Class {
    Digit,
    Letter,
    Char(char),
}

fn classify(c: char) -> Class {
    if c.is_ascii_digit() {
        Digit
    } else if c.is_ascii_alphabetic() {
        Class::Letter
    } else {
        Class::Char(c)
    }
}

fn transition(state: State, cls: Class) -> Option<State> {
    use Class::*;
    use State::*;

    match (state, cls) {
        (S, Digit) => Some(InInteger),
        (S, Letter) => Some(InIdentifier),
        (S, Char('+')) => Some(GotPlus),
        (S, Char('-')) => Some(GotMinus),
        (S, Char('*')) => Some(SeenStar),
        (S, Char('/')) => Some(SeenSlash),
        (S, Char('(')) => Some(GotLParen),
        (S, Char(')')) => Some(GotRParent),
        (S, Char('%')) => Some(GotPercent),
        (S, Char(',')) => Some(GotComma),
        (S, Char('=')) => Some(GotAssign),
        (S, Char('\n')) => Some(GotNewLine),
        (S, Char(c)) if c.is_whitespace() => Some(S),

        (InInteger, Digit) => Some(InInteger),
        (InInteger, Char('.')) => Some(GotDot),
        (GotDot, Digit) => Some(InFraction),
        (InFraction, Digit) => Some(InFraction),

        (SeenStar, Char('*')) => Some(GotDoubleStar),
        (SeenSlash, Char('/')) => Some(GotDoubleSlash),

        _ => None,
    }
}

fn is_accepting(state: State) -> bool {
    use State::*;

    !matches!(state, S | GotDot)
}

fn token_type_from_state(state: State) -> TokenType {
    match state {
        State::InInteger => TokenType::Integer,
        State::InFraction => TokenType::Decimal,
        State::InIdentifier => TokenType::Ident,
        State::GotPlus => TokenType::Plus,
        GotMinus => TokenType::Minus,
        State::SeenStar => TokenType::Star,
        State::SeenSlash => TokenType::Slash,
        State::GotDoubleStar => TokenType::DoubleStar,
        State::GotDoubleSlash => TokenType::DoubleSlash,
        State::GotLParen => TokenType::LParen,
        State::GotRParent => TokenType::RParen,
        State::GotPercent => TokenType::Percent,
        State::GotComma => TokenType::Comma,
        State::GotAssign => TokenType::Assign,
        State::GotNewLine => TokenType::NewLine,

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


                match transition(state, classify(c)) {
                    Some(next) => {
                        state = next;
                        self.bump();

                        if c == '\n' {
                            number_line += 1;
                            column = 0;
                        } else {
                            column += 1;
                        }

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
                    let lexeme = lexeme.as_str().trim().to_string();
                    let lexeme_len = lexeme.len();
                    tokens.push(Token {
                        token_type: token_type_from_state(state),
                        lexeme,
                        line: number_line,
                        column: column - lexeme_len,
                    });
                    self.pos = end;
                }

                None => return Err(CalcError::Lex(String::from("unexpected EOF"))),
            }
        }

        Ok(tokens)
    }
}
