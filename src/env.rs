use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::types::DataType;

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
