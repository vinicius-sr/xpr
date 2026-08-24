pub trait Blueprint<T> {
    fn find(&self, name: &str) -> Option<MethodInfo<T>>;
}

pub trait Callable<T> {
    fn call(&self, op: &T, args: &[f64]) -> f64;
}

pub struct MethodInfo<T> {
    pub id: T,
    pub arity: usize,
}

impl<T> MethodInfo<T> {
    pub fn new(id: T, arity: usize) -> Self {
        Self { id, arity }
    }
}
