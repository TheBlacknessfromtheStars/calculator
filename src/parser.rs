use std::cell::Cell;
use crate::error::{CalcError, CalcResult};
use crate::token::{Token, TokenType};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Lit {
    Integer(i64),
    Decimal(f64)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Pow, Mod, FloorDiv
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryOp {
    Neg
}

#[derive(Debug)]
pub enum Expr {
    Literal(Lit),
    Var(String),
    Call(String, Vec<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    FuncDef(String, Vec<String>, Expr),
    VarDef(String, Expr)
}

pub struct Parse {
    tokens: Vec<Token>,
    pos: Cell<usize>
}

impl Parse {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: Cell::new(0)
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
        if let Some(t) = self.peek() && t.token_type == tt {
            self.bump();
            Some(t)
        } else {
            None
        }
    }

    fn expect(&self, tt: TokenType) -> CalcResult<&Token> {
        if let Some(t) = self.eat(tt) {
            Ok(t)
        } else {
            Err(CalcError::Parse(String::from(format!("预期为{:?},但遇到{:?}", tt, self.peek().unwrap().token_type))))
        }
    }

    pub fn parse_program(&self) -> CalcResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while self.peek().is_some() {
            statements.push(self.parse_statement()?);
            self.eat(TokenType::NewLine);
        }

        Ok(statements)
    }

    fn parse_statement(&self) -> CalcResult<Stmt> {
        if let Some(fd) = self.try_parse_funcdef()? {
            return Ok(fd)
        }

        if let Some(vd) = self.try_parse_vardef()? {
            return Ok(vd)
        }

        Ok(Stmt::Expr(self.parse_expression()?))
    }

    fn try_parse_funcdef(&self) -> CalcResult<Option<Stmt>> {
        let save = self.pos();

        if !self.peek_two(TokenType::Ident, TokenType::LParen) {
            self.pos.set(save);
            return Ok(None)
        }

        let func_name = self.eat(TokenType::Ident).unwrap().lexeme.clone();
        self.bump();

        let mut params = Vec::new();

        if let Some(t) = self.peek() {
            if t.token_type == TokenType::RParen { // 匹配到')',无参数函数
                self.bump();
            } else {
                loop {
                    let param_name = match self.peek() {
                        Some(t) if t.token_type == TokenType::Ident => {
                            self.bump();
                            t.lexeme.clone()
                        }

                        _ => {
                            self.pos.set(save);
                            return Ok(None)
                        }
                    };

                    params.push(param_name);

                    if let Some(t) =  self.peek() {
                        match t.token_type {
                            TokenType::RParen => {
                                self.bump();
                                break;
                            }

                            TokenType::Comma => {
                                self.bump();
                            }

                            _ => {
                                self.pos.set(save);
                                return Ok(None)
                            }
                        }
                    }
                }
            }
        } else {
            return Err(CalcError::Parse(String::from(format!("非预期的{:?}", self.peek()))))
        }

        match self.peek() {
            Some(t) if t.token_type == TokenType::Assign => self.bump(),

            _ => {
                self.pos.set(save);
                return Ok(None)
            }
        }

        let boy = self.parse_expression()?;

        Ok(Some(Stmt::FuncDef(func_name, params, boy)))
    }

    fn try_parse_vardef(&self) -> CalcResult<Option<Stmt>> {
        let save = self.pos();

        if !self.peek_two(TokenType::Ident, TokenType::Assign) {
            self.pos.set(save);
            return Ok(None)
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
    where F: Fn() -> CalcResult<Expr> {
        let mut left = next()?;

        loop {
            let Some(t) = self.peek() else { break };
            let Some((_, bo)) = ops.iter().find(|(tt, _)| t.token_type == *tt) else { break };

            self.bump();
            let right = next()?;
            left = Expr::Binary(*bo, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_add_sub(&self) -> CalcResult<Expr> {
        let ops = [
            (TokenType::Plus, BinaryOp::Add),
            (TokenType::Minus, BinaryOp::Sub)
        ];

        self.parse_binary(|| self.parse_mod(), &ops)
    }

    fn parse_mod(&self) -> CalcResult<Expr> {
        let ops = [
            (TokenType::Percent, BinaryOp::Mod)
        ];

        self.parse_binary(|| self.parse_mul_div(), &ops)
    }

    fn parse_mul_div(&self) -> CalcResult<Expr> {
        let ops = [
            (TokenType::Star, BinaryOp::Mul),
            (TokenType::Slash, BinaryOp::Div),
            (TokenType::DoubleSlash, BinaryOp::FloorDiv)
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
        let ops = [
            (TokenType::DoubleStar, BinaryOp::Pow)
        ];

        self.parse_binary(|| self.parse_primary(), &ops)
    }

    fn parse_primary(&self) -> CalcResult<Expr> {
        if self.eat(TokenType::LParen).is_some() {
            let boy = self.parse_expression()?;
            self.expect(TokenType::RParen);
            return Ok(boy)
        }

        match self.peek() {
            Some(t) => {
                match t.token_type {
                    TokenType::Integer => {
                        let i: i64 = t.lexeme.parse().map_err(|_| CalcError::Parse(
                                String::from(format!("超出范围{}", t.lexeme))
                        ))?;
                        self.bump();
                        return Ok(Expr::Literal(Lit::Integer(i)))
                    }

                    TokenType::Decimal => {
                        let f: f64 = t.lexeme.parse().map_err(|_| CalcError::Parse(
                                String::from(format!("超出范围{}", t.lexeme))
                        ))?;
                        self.bump();
                        return Ok(Expr::Literal(Lit::Decimal(f)))
                    }

                    TokenType::Ident => {
                        let name = t.lexeme.clone();
                        
                        self.bump();
                        if self.eat(TokenType::LParen).is_some() {
                            if self.eat(TokenType::RParen).is_some() {
                                let mut args = Vec::new();
                                loop {
                                    let arg = self.parse_expression()?;
                                    args.push(arg);
                                    if self.eat(TokenType::Comma).is_none() {
                                        break;
                                    }
                                }
                                self.expect(TokenType::RParen)?;
                                return Ok(Expr::Call(
                                    name, 
                                    args
                                ))
                            }
                        }

                        return Ok(Expr::Var(name))
                    }

                    _ => return Err(CalcError::Parse(format!("意外的token: {:?}", t)))
                }
            }

            Some(t) => return Err(CalcError::Parse(String::from(format!("意外的token: {:?}", t)))),
            None => Err(CalcError::Parse(String::from("意外的输入结束"))),
        }
    }
}
