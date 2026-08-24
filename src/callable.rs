pub trait Blueprint<T> {
    fn find(name: &str) -> Option<MethodInfo<T>>;
}

pub trait Callable<T> {
    fn call(op: T, args: &[f64]) -> f64;
}

pub struct MethodInfo<T> {
    id: T,
    arity: usize,
}

impl<T> MethodInfo<T> {
    pub fn new(id: T, arity: usize) -> Self {
        Self { id, arity }
    }
}
