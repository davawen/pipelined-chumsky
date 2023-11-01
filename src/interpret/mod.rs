use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::ast::Expr;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone)]
struct Scope {
    variables: HashMap<String, Value>
}

type RCScope = Rc<RefCell<Scope>>;

impl Scope {
    fn get(&self, variable: &str) -> Result<Value> {
        self.variables.get(variable).cloned().ok_or(format!("variable '{variable}' does not exist"))
    }
}

#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    String(String),
    Tuple(Vec<Value>),
    List(Vec<Value>),
    Func(Function)
}

impl From<Value> for Vec<Value> {
    fn from(value: Value) -> Self {
        vec![value]
    }
}

#[derive(Debug, Clone)]
enum Function {
    Builtin(fn (Vec<Value>) -> Vec<Value>),
    Closure {
        args: Vec<String>,
        env: RCScope,
        body: Expr
    }
}

fn eval(e: &Expr, s: RCScope) -> Result<Vec<Value>> {
    match e {
        Expr::Ident(v) => s.borrow().get(v),
        &Expr::Number(n) => Ok(vec![Value::Number(n).rc()]),
        Expr::String(s) => Ok(vec![Value::String(s.clone()).rc()]),
        Expr::Tuple(t) => todo!(),
        Expr::Pipeline(_, _) => todo!(),
        Expr::Assign(_, _) => todo!(),
        Expr::Mutate(_, _) => todo!(),
        Expr::Lambda(_, _) => todo!(),
    }
}
