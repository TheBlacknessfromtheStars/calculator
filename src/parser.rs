use crate::error::CalcResult;
use crate::token::{Token, TokenType};

#[derive(Copy, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Decimal(f64)
}

pub enum BinaryOp {
    Add, Sub, Mul, Div, Pow
}

pub enum Expr {
    Value(Value),
    Variable(String),
    Call(String, Vec<Expr>),
    Neg(Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

pub enum Statement {
    Expr(Expr),
    FuncDef(String, Vec<String>, Box<Expr>),
    VariableDef(String, Box<Expr>)
}

pub struct Parse {
    tokens: Vec<Token>,
    pos: usize
}

impl Parse {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn bump(&mut self) {
        self.pos += 1;
    }

    fn eat(&mut self, tt: TokenType) -> bool {
        if let Some(t) = self.peek() && t.token_type == tt {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn parse_program(&mut self) -> CalcResult<Vec<Statement>> {
        let mut statements = Vec::new();

        while self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> CalcResult<Statement> {
        if let Some(fd) = self.try_parse_funcdef()? {
            return Ok(fd)
        }

        Ok(Statement::Expr(self.parse_assign()?))
    }

    fn try_parse_funcdef(&mut self) -> CalcResult<Option<Statement>> {
        todo!()
    }

    fn parse_assign(&mut self) -> CalcResult<Expr> {
        todo!()
    }
}