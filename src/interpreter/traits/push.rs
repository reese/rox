pub trait Push<T> {
    fn push(&mut self, t: T) -> u8;
}
