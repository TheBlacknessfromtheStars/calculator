use std::{cell::Cell, collections::HashMap, ops::{Add, Div, Mul, Rem, Sub}};

use crate::parser::{BinaryOp, Expr, Lit, Stmt};

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Integer(i64),
    Decimal(f64)
}

impl From<Lit> for Value {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Integer(i) => Value::Integer(i),
            Lit::Decimal(f) => Value::Decimal(f)
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    sym_table: HashMap<String, Value>
}

impl Scope {
    pub fn new() -> Self {
        Self { sym_table: HashMap::new() }
    }

    pub fn get(&self, sym_name: &String) -> Option<Value> {
        self.sym_table.get(sym_name).copied()
    }

    pub fn insert(&mut self, sym_name: String, value: Value) {
        self.sym_table.insert(sym_name, value);
    }
}

#[derive(Debug)]
pub struct Eval {
    stmts: Vec<Stmt>,
    scope: Scope
}

impl Eval {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self {
            stmts, 
            scope: Scope::new() 
        }
    }

    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    pub fn eval(&mut self) {
        let stmts = std::mem::take(&mut self.stmts);

        for s in &stmts {
            self.eval_stmt(s);
        }
    }

    fn eval_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                let value = self.eval_expr(expr);

                match value {
                    Value::Integer(i) => println!("{i}"),
                    Value::Decimal(f) => println!("{f}")
                }
            }

            Stmt::VarDef(name, expr) => {
                let value = self.eval_expr(expr);
                self.scope.insert(name.clone(), value);
            }

            Stmt::FuncDef(name, params, expr) => {

            }
        }
    }

    fn promote(lv: Value, rv: Value) -> (Value, Value) {
        match (lv, rv) {
            (Value::Integer(i), Value::Decimal(f)) => (Value::Decimal(i as f64), Value::Decimal(f)),

            (Value::Decimal(f), Value::Integer(i)) => (Value::Decimal(f), Value::Decimal(i as f64)),

            other => other
        }
    }

    fn two_calculator<T>(op: BinaryOp, a: T, b: T) -> T
    where 
      T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Copy
    {
        match op {
            BinaryOp::Add => a + b,
            BinaryOp::Sub => a - b,
            BinaryOp::Mul => a * b,
            BinaryOp::Div => a / b,
            BinaryOp::Mod => a % b,
            _ => panic!()
        }
    }

    fn arith(op: BinaryOp, lv: Value, rv: Value) -> Value {
        match Self::promote(lv, rv) {
            (Value::Integer(l), Value::Integer(r)) => { 
                let v = Self::two_calculator(op, l, r);

                return Value::Integer(v);
            }

            (Value::Decimal(l), Value::Decimal(r)) => {
                let v = Self::two_calculator(op, l, r);

                return Value::Decimal(v);
            }

            _ => panic!()
        }
    }

    fn eval_expr(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Var(name) => {
                let v = self.scope.get(&name);

                if v.is_none() {
                    panic!("{}", format!("未找到变量{}的值", name));
                } else {
                    return v.unwrap();
                }
            }

            Expr::Binary(op, left, right) => {
                let left_value = self.eval_expr(left);
                let right_value = self.eval_expr(right);
                
                return match op {
                    _ => Self::arith(*op, left_value, right_value) 
                };
            }

            Expr::Literal(lit) => {
                return Value::from(*lit);
            }

            _ => {
                panic!("{:?}", expr)
            }
        }
    }

}
