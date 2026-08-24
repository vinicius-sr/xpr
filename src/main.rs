mod callable;
mod compiler;
mod interpreter;
mod reader;
mod scanner;

use std::env;

use crate::{
    MethodId::*,
    callable::{Blueprint, Callable, MethodInfo},
    interpreter::Interpreter,
};

pub struct Math;

#[derive(Debug, PartialEq)]
enum MethodId {
    Sum,
    Sub,
    Read,
}

impl Blueprint<MethodId> for Math {
    fn find(name: &str) -> Option<MethodInfo<MethodId>> {
        match name {
            "sum" => Some(MethodInfo::new(Sum, 2)),
            "sub" => Some(MethodInfo::new(Sub, 2)),
            "read" => Some(MethodInfo::new(Read, 1)),
            _ => None,
        }
    }
    // fn find(name: &str) -> Option<Vec<rgType>> {

    // }
}

impl Callable<MethodId> for Math {
    fn call(op: MethodId, args: &[f64]) -> f64 {
        match (op, args) {
            (Sum, &[l, r]) => l + r,
            (Sub, &[l, r]) => l - r,
            (Read, &[l]) => l as f64,
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
                Ok(Some(v)) => println!("Response: {}", v),
                Ok(None) => println!("No result"),
                Err(e) => println!("{:?}", e),
            }
        }
        None => println!("No expression or function provided"),
    }
}
