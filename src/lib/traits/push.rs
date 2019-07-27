pub trait Push<T> {
  fn push(&mut self, t: T);
}

pub trait PushLine<T> {
  fn push_line(&mut self, t: T, line: i32) -> usize;
}
