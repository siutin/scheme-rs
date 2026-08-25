use std::rc::Rc;
use std::fmt;
use std::f64;

use crate::SchemeError;
use crate::env::EnvRef;

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
pub struct Procedure {
    pub body: Rc<AST>,
    pub params: Vec<DataType>,
    pub env: EnvRef
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

impl PartialEq for Procedure {
    fn eq(&self, other: &Procedure) -> bool {
        self.body == other.body
            && self.params == other.params
            && Rc::ptr_eq(&self.env, &other.env)
    }
}

pub struct Function(pub Rc<dyn Fn(Vec<DataType>, EnvRef) -> Result<Option<DataType>, SchemeError>>);

impl Function {
    pub fn call(&self, arguments: Vec<DataType>, env: EnvRef) -> Result<Option<DataType>, SchemeError> {
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
pub enum DataType {
    Bool(bool),
    Pair((Box<DataType>, Box<DataType>)),
    Integer(i64),
    Float(f64),
    Symbol(String),
    String(String),
    Proc(Function),
    List(Vec<DataType>),
    Lambda(Procedure)
}

impl PartialEq for DataType {
    fn eq(&self, other: &DataType) -> bool {
        match (self, other) {
            // Cross-type numeric equality: Integer(42) == Float(42.0) is true
            (&DataType::Integer(a), &DataType::Float(b)) => a as f64 == b,
            (&DataType::Float(a), &DataType::Integer(b)) => a == b as f64,
            (&DataType::Integer(a), &DataType::Integer(b)) => a == b,
            (&DataType::Float(a), &DataType::Float(b)) => a == b,
            (&DataType::Bool(a), &DataType::Bool(b)) => a == b,
            (&DataType::Symbol(ref a), &DataType::Symbol(ref b)) => a == b,
            (&DataType::String(ref a), &DataType::String(ref b)) => a == b,
            (&DataType::Pair(ref a), &DataType::Pair(ref b)) => a == b,
            (&DataType::List(ref a), &DataType::List(ref b)) => a == b,
            (&DataType::Proc(ref a), &DataType::Proc(ref b)) => a == b,
            (&DataType::Lambda(ref a), &DataType::Lambda(ref b)) => a == b,
            _ => false,
        }
    }
}

impl DataType {
    /// Extract f64 from either Integer or Float. Returns None for non-numbers.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            &DataType::Integer(i) => Some(i as f64),
            &DataType::Float(f) => Some(f),
            _ => None,
        }
    }

    /// Check if this is a number (Integer or Float)
    pub fn is_number(&self) -> bool {
        match self {
            &DataType::Integer(_) | &DataType::Float(_) => true,
            _ => false,
        }
    }

    /// Check if this is an integer (Integer type specifically)
    pub fn is_integer(&self) -> bool {
        match self {
            &DataType::Integer(_) => true,
            _ => false,
        }
    }
}
