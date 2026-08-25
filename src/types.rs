use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use std::f64;

use crate::SchemeError;

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Integer(i64),
    Float(f64),
    Symbol(String),
    Children(Vec<AST>)
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct ReadFromTokenResult {
    pub remain: Vec<String>,
    pub result: AST
}

#[derive(Clone)]
#[derive(PartialEq)]
pub struct Procedure {
    pub body: AST,
    pub params: Vec<DataType>,
    pub env: Rc<RefCell<Env>>
}

impl fmt::Debug for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let env_raw = &self.env as *const _;

        f.debug_struct("Procedure")
            .field("body", &self.body)
            .field("params", &self.params)
            .field("env", &env_raw)
            .finish()
    }
}

pub struct Function(pub Rc<dyn Fn(Vec<DataType>, Rc<RefCell<Env>>) -> Result<Option<DataType>, SchemeError>>);

impl Function {
    pub fn call(&self, arguments: Vec<DataType>, env: Rc<RefCell<Env>>) -> Result<Option<DataType>, SchemeError> {
        (self.0)(arguments, env)
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function(self.0.clone())
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let raw = &self.0 as *const _;
        f.debug_tuple("Function").field(&raw).finish()
    }
}

impl std::cmp::PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        let self_raw = &self.0 as *const _;
        let other_raw = &other.0 as *const _;
        self_raw == other_raw
    }
}

pub trait FloatIterExt {
    fn float_min(&mut self) -> f64;
    fn float_max(&mut self) -> f64;
}

impl<T> FloatIterExt for T where T: Iterator<Item=f64> {
    fn float_max(&mut self) -> f64 {
        self.fold(f64::NAN, f64::max)
    }

    fn float_min(&mut self) -> f64 {
        self.fold(f64::NAN, f64::min)
    }
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub enum DataType {
    Bool(bool),
    Pair((Box<DataType>, Box<DataType>)),
    Number(f64),
    Symbol(String),
    String(String),
    Proc(Function),
    List(Vec<DataType>),
    Lambda(Procedure)
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct Env {
    pub local: Box<RefCell<HashMap<String, DataType>>>,
    pub parent: Option<Box<Rc<RefCell<Env>>>>
}

impl Env {
    pub fn get(&self, key: &String) -> Option<DataType> {
        match self.local.borrow().get::<str>(key) {
            Some(&DataType::Bool(b)) => Some(DataType::Bool(b)),
            Some(&DataType::Pair(ref p)) => Some(DataType::Pair(p.clone())),
            Some(&DataType::Number(f)) => Some(DataType::Number(f)),
            Some(&DataType::Symbol(ref ss)) => Some(DataType::Symbol(ss.clone())),
            Some(&DataType::String(ref ss)) => Some(DataType::String(ss.clone())),
            Some(&DataType::Proc(ref p)) => Some(DataType::Proc(p.clone())),
            Some(&DataType::List(ref l)) => Some(DataType::List(l.clone())),
            Some(&DataType::Lambda(ref p)) => Some(DataType::Lambda(p.clone())),
            None => {
                match self.parent {
                    Some(ref some_parent) => {
                        let parent_borrow = some_parent.borrow();
                        parent_borrow.get(key)
                    }
                    None => None
                }
            }
        }
    }
}
