use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

use crate::types::DataType;

/// A shared, mutable environment reference.
///
/// All code that threads an environment through eval, closures, and harnesses
/// uses this type alias instead of the concrete `Rc<RefCell<Env>>`.  This is
/// the single seam that decouples the interpreter from the `Env` struct's
/// internal representation, enabling mock environments for testing and
/// alternative scope implementations (e.g. interned-symbol scopes for
/// performance — see SPEC_PERF bottleneck #3).
pub type EnvRef = Rc<RefCell<dyn Environment>>;

/// The environment abstraction.
///
/// `eval`, `Procedure`, `Function`, and all harnesses depend on this trait
/// rather than the concrete `Env` struct.  The original `Env` (HashMap-based)
/// is one implementation; future implementations can use interned symbols,
/// flat vectors, or persistent data structures without touching `eval.rs`.
pub trait Environment: fmt::Debug {
    /// Look up a symbol, searching up the parent chain.
    fn get(&self, key: &str) -> Option<DataType>;

    /// Set an existing binding, searching up the parent chain.
    /// Returns true if found and set, false if not found.
    fn set(&self, key: &str, value: DataType) -> bool;

    /// Define a new binding in the local scope (shadows parent).
    fn define(&self, key: String, value: DataType);
}

/// The original HashMap-based environment.
///
/// Uses `HashMap<String, DataType>` for local bindings and an optional parent
/// pointer for lexical scoping.  This is the default `Environment` impl; the
/// trait allows swapping in alternatives (e.g. `InternedEnv`) without changing
/// any call site.
pub struct Env {
    pub local: Box<RefCell<HashMap<String, DataType>>>,
    pub parent: Option<EnvRef>,
}

impl Env {
    /// Construct a new environment with the given local bindings and optional parent.
    pub fn new(local: HashMap<String, DataType>, parent: Option<EnvRef>) -> Self {
        Env {
            local: Box::new(RefCell::new(local)),
            parent,
        }
    }

    /// Create a root environment (no parent) from a setup HashMap.
    /// Returns an `EnvRef` ready to pass to `eval`.
    pub fn root(local: HashMap<String, DataType>) -> EnvRef {
        Rc::new(RefCell::new(Env::new(local, None)))
    }

    /// Create a child environment with an empty local scope parented to `parent`.
    pub fn child(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Env::new(HashMap::new(), Some(parent)))) as EnvRef
    }

    /// Create a child environment with the given bindings parented to `parent`.
    pub fn child_with(parent: EnvRef, bindings: HashMap<String, DataType>) -> EnvRef {
        Rc::new(RefCell::new(Env::new(bindings, Some(parent)))) as EnvRef
    }
}

impl Environment for Env {
    fn get(&self, key: &str) -> Option<DataType> {
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

    fn set(&self, key: &str, value: DataType) -> bool {
        if self.local.borrow().contains_key::<str>(key) {
            self.local.borrow_mut().insert(key.to_string(), value);
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

    fn define(&self, key: String, value: DataType) {
        self.local.borrow_mut().insert(key, value);
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Env")
            .field("local", &self.local)
            .field("parent", &self.parent.as_ref().map(|_| "..."))
            .finish()
    }
}
