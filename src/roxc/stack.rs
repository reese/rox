#[derive(Debug)]
pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { stack: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn push(&mut self, item: T) {
        self.stack.push(item);
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn top(&self) -> &T {
        self.stack.last().expect("No items on stack")
    }

    pub fn top_mut(&mut self) -> &mut T {
        self.stack.last_mut().expect("No items on stack")
    }
}
