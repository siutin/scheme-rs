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
        match self.local.borrow().get::<str>(key).cloned() {
            Some(data) => Some(data),
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

    /// Set an existing binding, searching up the parent chain.
    /// Returns true if found and set, false if not found.
    pub fn set(&self, key: &String, value: DataType) -> bool {
        if self.local.borrow().contains_key::<str>(key) {
            self.local.borrow_mut().insert(key.clone(), value);
            return true;
        }
        match self.parent {
            Some(ref some_parent) => {
                let parent_borrow = some_parent.borrow();
                parent_borrow.set(key, value)
            }
            None => false
        }
    }
}
