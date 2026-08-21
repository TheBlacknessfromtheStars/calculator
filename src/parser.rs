use crate::error::{CalcError, CalcResult};
use crate::token::{Token, TokenType};
use std::cell::Cell;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Lit {
    Integer(i64),
    Decimal(f64),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    FloorDiv,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Lit),
    Var(String),
    Call(String, Vec<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    FuncDef(String, Vec<String>, Expr),
    VarDef(String, Expr),
}

pub struct Parse {
    tokens: Vec<Token>,
    pos: Cell<usize>,
}

impl Parse {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: Cell::new(0),
        }
    }

    fn pos(&self) -> usize {
        self.pos.get()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos())
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos() + 1)
    }

    fn peek_two(&self, first: TokenType, second: TokenType) -> bool {
        matches!(
            (self.peek(), self.peek_next()),
            (Some(c), Some(n)) if c.token_type == first && n.token_type == second
        )
    }

    fn bump(&self) {
        self.pos.set(self.pos() + 1);
    }

    fn eat(&self, tt: TokenType) -> Option<&Token> {
        match self.peek() {
            Some(t) if t.token_type == tt => {
                self.bump();
                Some(t)
            },
            _ => None,
        }
    }

    fn expect(&self, tt: TokenType) -> CalcResult<&Token> {
        if let Some(t) = self.eat(tt) {
            Ok(t)
        } else {
            if self.peek().is_some() {
                Err(CalcError::Parse(format!(
                    "预期为{:?},但遇到{:?}",
                    tt,
                    self.peek().unwrap().token_type
                )))
            } else { 
                Err(CalcError::Parse("意料之外的结束".to_string()))
            }
        }
    }

    pub fn parse_program(&self) -> CalcResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while self.peek().is_some() {
            statements.push(self.parse_statement()?);
            while self.eat(TokenType::NewLine).is_some() {}
        }

        Ok(statements)
    }   

    fn parse_statement(&self) -> CalcResult<Stmt> {
        if let Some(fd) = self.try_parse_func_def()? {
            return Ok(fd);
        }

        if let Some(vd) = self.try_parse_var_def()? {
            return Ok(vd);
        }

        Ok(Stmt::Expr(self.parse_expression()?))
    }

    fn try_parse_func_def(&self) -> CalcResult<Option<Stmt>> {
        let save = self.pos();

        if !self.peek_two(TokenType::Ident, TokenType::LParen) {
            self.pos.set(save);
            return Ok(None);
        }

        let func_name = self.eat(TokenType::Ident).unwrap().lexeme.clone();
        self.bump();

        let Some(params) = self.parse_func_param_list()? else {
            self.pos.set(save);
            return Ok(None);
        };

        match self.peek() {
            Some(t) if t.token_type == TokenType::Assign => self.bump(),

            _ => {
                self.pos.set(save);
                return Ok(None);
            }
        }

        let boy = self.parse_expression()?;

        Ok(Some(Stmt::FuncDef(func_name, params, boy)))
    }

    fn parse_func_param_list(&self) -> CalcResult<Option<Vec<String>>> {
        let mut params = Vec::new();

        if self.eat(TokenType::RParen).is_some() {
            return Ok(Some(params));
        }

        loop {
            let param_name = match self.eat(TokenType::Ident) {
                Some(t) => t.lexeme.clone(),

                None => return Ok(None),
            };

            params.push(param_name);

            if let Some(t) = self.peek() {
                match t.token_type {
                    TokenType::RParen => {
                        self.bump();
                        break;
                    }

                    TokenType::Comma => self.bump(),

                    _ => return Ok(None), // 可能是函数调用，回退
                }
            } else {
                return Err(CalcError::Parse("意料之外的结束".to_string()));
            }
        }

        Ok(Some(params))
    }

    fn try_parse_var_def(&self) -> CalcResult<Option<Stmt>> {
        let save = self.pos();

        if !self.peek_two(TokenType::Ident, TokenType::Assign) {
            self.pos.set(save);
            return Ok(None);
        }

        let var_name = self.eat(TokenType::Ident).unwrap().lexeme.clone();
        self.bump();

        let boy = self.parse_expression()?;

        Ok(Some(Stmt::VarDef(var_name, boy)))
    }

    fn parse_expression(&self) -> CalcResult<Expr> {
        self.parse_add_sub()
    }
    fn parse_binary<F>(&self, next: F, ops: &[(TokenType, BinaryOp)]) -> CalcResult<Expr>
    where
        F: Fn() -> CalcResult<Expr>,
    {
        let mut left = next()?;

        loop {
            let Some(t) = self.peek() else { break };
            let Some((_, bo)) = ops.iter().find(|(tt, _)| t.token_type == *tt) else {
                break;
            };

            self.bump();
            let right = next()?;
            left = Expr::Binary(*bo, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_add_sub(&self) -> CalcResult<Expr> {
        let ops = [
            (TokenType::Plus, BinaryOp::Add),
            (TokenType::Minus, BinaryOp::Sub),
            (TokenType::Percent, BinaryOp::Mod),
        ];

        self.parse_binary(|| self.parse_mul_div_mod(), &ops)
    }

    fn parse_mul_div_mod(&self) -> CalcResult<Expr> {
        let ops = [
            (TokenType::Star, BinaryOp::Mul),
            (TokenType::Slash, BinaryOp::Div),
            (TokenType::DoubleSlash, BinaryOp::FloorDiv),
            (TokenType::Percent, BinaryOp::Mod),
        ];

        self.parse_binary(|| self.parse_unary(), &ops)
    }

    fn parse_unary(&self) -> CalcResult<Expr> {
        if let Some(_) = self.eat(TokenType::Minus) {
            let operand = self.parse_power()?;
            return Ok(Expr::Unary(UnaryOp::Neg, Box::new(operand)));
        }

        self.parse_power()
    }

    fn parse_power(&self) -> CalcResult<Expr> {
        let left = self.parse_primary()?;

        if self.eat(TokenType::DoubleStar).is_some() {
            let right = self.parse_unary()?;
            Ok(Expr::Binary(BinaryOp::Pow, Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_primary(&self) -> CalcResult<Expr> {
        if self.eat(TokenType::LParen).is_some() {
            let boy = self.parse_expression()?;
            self.expect(TokenType::RParen)?;
            return Ok(boy);
        }

        match self.peek() {
            Some(t) => match t.token_type {
                TokenType::Integer => self.parse_integer_literal(),
                TokenType::Decimal => self.parse_decimal_literal(),
                TokenType::Ident => self.parse_identifier(),

                _ => Err(CalcError::Parse(format!("意外的token: {:?}", t))),
            },

            None => Err(CalcError::Parse("意外的输入结束".to_string())),
        }
    }

    fn parse_integer_literal(&self) -> CalcResult<Expr> {
        let t = self.peek().unwrap();
        let i: i64 = t
            .lexeme
            .parse()
            .map_err(|_| CalcError::Parse(format!("超出范围{}", t.lexeme)))?;
        self.bump();
        Ok(Expr::Literal(Lit::Integer(i)))
    }

    fn parse_decimal_literal(&self) -> CalcResult<Expr> {
        let t = self.peek().unwrap();
        let f: f64 = t
            .lexeme
            .parse()
            .map_err(|_| CalcError::Parse(format!("超出范围{}", t.lexeme)))?;
        self.bump();
        Ok(Expr::Literal(Lit::Decimal(f)))
    }

    fn parse_identifier(&self) -> CalcResult<Expr> {
        let t = self.peek().unwrap();
        let name = t.lexeme.clone();

        self.bump();
        if self.eat(TokenType::LParen).is_some() {
            if self.eat(TokenType::RParen).is_none() {
                let mut args = Vec::new();
                loop {
                    let arg = self.parse_expression()?;
                    args.push(arg);
                    if self.eat(TokenType::Comma).is_none() {
                        break;
                    }
                }
                self.expect(TokenType::RParen)?;
                return Ok(Expr::Call(name, args));
            }
        }

        Ok(Expr::Var(name))
    }
}
