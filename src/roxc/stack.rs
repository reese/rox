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

    pub fn top(&self) -> &T {
        self.stack.last().expect("No items on stack")
    }

    pub fn top_mut(&mut self) -> &mut T {
        self.stack.last_mut().expect("No items on stack")
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.stack[index] = value;
    }

    pub fn get(&self, index: usize) -> &T {
        self.stack.get(index).unwrap()
    }

    pub fn get_inner_array(&self) -> &Vec<T> {
        &self.stack
    }
}
