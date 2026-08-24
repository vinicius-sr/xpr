pub trait Blueprint {
    fn find(name: &str) -> Option<Vec<ArgType>>;
}

pub trait Callable<T> {
    fn call(op: T) -> f64;
}

pub enum ArgType {
    U8,
    F64,
}
