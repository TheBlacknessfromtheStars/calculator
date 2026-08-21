use std::collections::HashMap;

use crate::parser::{BinaryOp, Expr, Lit, Stmt, UnaryOp};

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Integer(i64),
    Decimal(f64),
}

trait Calc: Sized {
    fn calc(op: BinaryOp, lhs: Self, rhs: Self) -> Self;
}

impl Calc for i64 {
    fn calc(op: BinaryOp, lhs: Self, rhs: Self) -> Self {
        match op {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div | BinaryOp::FloorDiv => lhs / rhs,
            BinaryOp::Pow => {
                if rhs <= u32::MAX as i64 {
                    lhs.pow(rhs as u32)
                } else {
                    println!("指数过大");
                    0
                }
            }
            BinaryOp::Mod => lhs % rhs,
        }
    }
}

impl Calc for f64 {
    fn calc(op: BinaryOp, lhs: Self, rhs: Self) -> Self {
        match op {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
            BinaryOp::Pow => lhs.powf(rhs),
            BinaryOp::Mod => lhs % rhs,
            BinaryOp::FloorDiv => (lhs / rhs).trunc(),
        }
    }
}

impl From<Lit> for Value {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Integer(i) => Value::Integer(i),
            Lit::Decimal(f) => Value::Decimal(f),
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    sym_table: HashMap<String, Value>,
    func_table: HashMap<String, (Vec<String>, Expr)>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            sym_table: HashMap::new(),
            func_table: HashMap::new(),
        }
    }

    pub fn get_sym(&self, sym_name: &String) -> Option<Value> {
        self.sym_table.get(sym_name).copied()
    }

    pub fn insert_sym(&mut self, sym_name: String, value: Value) {
        self.sym_table.insert(sym_name, value);
    }

    pub fn get_func(&self, sym_name: &String) -> Option<&(Vec<String>, Expr)> {
        self.func_table.get(sym_name)
    }

    pub fn insert_func(&mut self, sym_name: String, value: (Vec<String>, Expr)) {
        self.func_table.insert(sym_name, value);
    }
}

#[derive(Debug)]
pub struct Eval {
    stmts: Vec<Stmt>,
    scope: Scope,
}

impl Eval {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self {
            stmts,
            scope: Scope::new(),
        }
    }

    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    pub fn eval(&mut self) {
        let stmts = std::mem::take(&mut self.stmts);

        for s in stmts {
            self.eval_stmt(s);
        }
    }

    fn eval_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                let value = eval_expr(&expr, &self.scope);

                match value {
                    Value::Integer(i) => println!("{i}"),
                    Value::Decimal(f) => println!("{f}"),
                }
            }

            Stmt::VarDef(name, expr) => {
                let value = eval_expr(&expr, &self.scope);
                self.scope.insert_sym(name, value);
            }

            Stmt::FuncDef(name, params, expr) => self.scope.insert_func(name, (params, expr)),
        }
    }
}

fn promote(lv: Value, rv: Value) -> (Value, Value) {
    match (lv, rv) {
        (Value::Integer(i), Value::Decimal(f)) => (Value::Decimal(i as f64), Value::Decimal(f)),

        (Value::Decimal(f), Value::Integer(i)) => (Value::Decimal(f), Value::Decimal(i as f64)),

        other => other,
    }
}

fn arith(op: BinaryOp, lv: Value, rv: Value) -> Value {
    match promote(lv, rv) {
        (Value::Integer(l), Value::Integer(r)) => Value::Integer(Calc::calc(op, l, r)),

        (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(Calc::calc(op, l, r)),

        _ => panic!(),
    }
}

fn eval_expr(expr: &Expr, scope: &Scope) -> Value {
    match expr {
        Expr::Var(name) => {
            let v = scope.get_sym(&name);

            if v.is_none() {
                panic!("{}", format!("未找到变量{}的值", name));
            } else {
                v.unwrap()
            }
        }

        Expr::Binary(op, left, right) => {
            let left_value = eval_expr(left, scope);
            let right_value = eval_expr(right, scope);

            match op {
                _ => arith(*op, left_value, right_value),
            }
        }

        Expr::Unary(op, expr) => {
            let value = eval_expr(expr, scope);

            match op {
                UnaryOp::Neg => match value {
                    Value::Integer(i) => Value::Integer(-i),
                    Value::Decimal(f) => Value::Decimal(-f),
                },
            }
        }

        Expr::Call(name, args) => {
            let (params, func_expr) = match scope.get_func(&name) {
                Some((p, e)) => (p.clone(), e.clone()),
                None => panic!("未定义的函数 {}", name),
            };

            if args.len() != params.len() {
                panic!(
                    "函数{}需要{}个参数但传入了{}",
                    name,
                    params.len(),
                    args.len()
                );
            }

            let mut args_value = Vec::new();
            let mut func_scope = Scope::new();

            for arg in args {
                args_value.push(eval_expr(arg, scope));
            }

            for (k, v) in params.iter().zip(args_value) {
                func_scope.insert_sym(k.clone(), v);
            }

            eval_expr(&func_expr, &func_scope)
        }

        Expr::Literal(lit) => Value::from(*lit),
    }
}
