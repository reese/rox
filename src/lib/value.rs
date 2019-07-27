use super::traits::Push;


#[derive(Debug)]
pub struct ValueArray {
  pub values: Vec<Value>
}

#[derive(Clone, Debug)]
pub struct Value {
  pub f: f64
}

impl ValueArray {
  pub fn new() -> ValueArray {
    return ValueArray {
      values: vec![]
    }
  }
}

impl Clone for ValueArray{
  fn clone(&self) -> ValueArray {
    return ValueArray {
      values: self.values.clone()
    }
  }
}

impl Push<Value> for ValueArray {
  fn push(&mut self, value: Value) {
    self.values.push(value)
  }
}
