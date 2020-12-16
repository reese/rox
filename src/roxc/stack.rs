#[derive(Debug)]
pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { stack: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.stack.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    #[allow(dead_code)]
    pub fn top(&self) -> &T {
        self.stack.last().expect("No items on stack")
    }

    pub fn get_unchecked(&self, index: usize) -> &T {
        &self.stack[index]
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.stack[index] = value;
    }
}
