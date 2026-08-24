trait Callable<T> {
    fn find(&self) -> Option<Vec<ArgType>>;
}

enum ArgType {
    U8,
    F64
}
