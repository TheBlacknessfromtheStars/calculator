use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CalcError {
    Lex(String),
    Parse(String),
    Eval(String)
}

pub type CalcResult<T> = Result<T, CalcError>;

impl Display for CalcError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::Lex(msg) => { write!(f, "Lex error: {}", msg) }
            CalcError::Parse(msg) => { write!(f, "Parse error: {}", msg) }
            CalcError::Eval(msg) => { write!(f, "Evaluation error: {}", msg) }
        }
    }
}

impl Error for CalcError {}