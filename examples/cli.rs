use std::env;

use xpr::{Blueprint, Callable, Interpreter, MethodInfo};

#[derive(Debug)]
enum MethodId {
    Sum,
    Sub,
    Foo,
}

struct Math;

impl Blueprint<MethodId> for Math {
    fn find(&self, name: &str) -> Option<MethodInfo<MethodId>> {
        match name {
            "sum" => Some(MethodInfo::new(MethodId::Sum, 2)),
            "sub" => Some(MethodInfo::new(MethodId::Sub, 2)),
            "foo" => Some(MethodInfo::new(MethodId::Foo, 1)),
            _ => None,
        }
    }
}

impl Callable<MethodId> for Math {
    fn call(&self, op: &MethodId, args: &[f64]) -> f64 {
        match (op, args) {
            (MethodId::Sum, &[l, r]) => l + r,
            (MethodId::Sub, &[l, r]) => l - r,
            (MethodId::Foo, &[l]) => l,
            _ => todo!(),
        }
    }
}

fn main() {
    let math = Math;
    match env::args().nth(1) {
        Some(source) => {
            println!("Received: {}", source);

            match Interpreter::compile_and_run(source.as_str(), math) {
                Ok(v) => println!("Response: {}", v),
                Err(e) => println!("{:?}", e),
            }
        }
        None => println!("No expression or function provided"),
    }
}
