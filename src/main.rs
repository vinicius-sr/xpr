mod callable;
mod compiler;
mod interpreter;
mod reader;
mod scanner;

use std::env;

use crate::{
    callable::{
        ArgType::{self, *},
        Blueprint, Callable,
    },
    interpreter::Interpreter,
};

pub struct Math;

enum Method {
    Sum(f64, f64),
    Sub(f64, f64),
    Read(u8),
}

impl Blueprint for Math {
    fn find(name: &str) -> Option<Vec<ArgType>> {
        match name {
            "sum" => Some(vec![F64, F64]),
            "sub" => Some(vec![F64, F64]),
            "read" => Some(vec![U8]),
            _ => None,
        }
    }
}

impl Callable<Method> for Math {
    fn call(op: Method) -> f64 {
        match op {
            Method::Sum(l, r) => l + r,
            Method::Sub(l, r) => l - r,
            Method::Read(i) => i as f64,
        }
    }
}

fn main() {
    let math = Math;
    match env::args().nth(1) {
        Some(source) => {
            println!("Received: {}", source);

            match Interpreter::compile_and_run(source.as_str(), math) {
                Ok(Some(v)) => println!("Response: {}", v),
                Ok(None) => println!("No result"),
                Err(e) => println!("{:?}", e),
            }
        }
        None => println!("No expression or function provided"),
    }
}
