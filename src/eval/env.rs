use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::value::Value;

#[derive(Clone)]
pub struct Env {
    inner: Rc<EnvInner>,
}

struct EnvInner {
    bindings: RefCell<HashMap<String, Value>>,
    parent: Option<Env>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            inner: Rc::new(EnvInner {
                bindings: RefCell::new(HashMap::new()),
                parent: None,
            }),
        }
    }

    pub fn extend(&self) -> Self {
        Env {
            inner: Rc::new(EnvInner {
                bindings: RefCell::new(HashMap::new()),
                parent: Some(self.clone()),
            }),
        }
    }

    pub fn set(&self, name: String, value: Value) {
        self.inner.bindings.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.inner.bindings.borrow().get(name) {
            Some(v.clone())
        } else if let Some(ref parent) = self.inner.parent {
            parent.get(name)
        } else {
            None
        }
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Env{{...}}")
    }
}
