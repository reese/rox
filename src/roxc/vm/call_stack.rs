use crate::roxc::vm::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct CallStack {
    stack: Rc<RefCell<Vec<Value>>>,
}

impl CallStack {
    pub(crate) fn new() -> Self {
        CallStack {
            stack: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(crate) fn get(&self, index: usize) -> Value {
        self.stack.borrow().get(index).unwrap().clone()
    }

    pub(crate) fn set(&mut self, index: usize, value: Value) {
        self.stack.borrow_mut()[index] = value;
    }

    pub(crate) fn push(&mut self, value: Value) {
        self.stack.borrow_mut().push(value)
    }

    pub(crate) fn pop(&mut self) -> Value {
        self.stack.borrow_mut().pop().unwrap()
    }

    pub(crate) fn len(&self) -> usize {
        self.stack.borrow().len()
    }
}
