mod compiler;
mod interpreter;
mod reader;
mod scanner;
mod callable;

use std::env;

use crate::interpreter::Interpreter;

fn main() {
    match env::args().nth(1) {
        Some(source) => {
            println!("Received: {}", source);

            match Interpreter::compile_and_run(source.as_str()) {
                Ok(Some(v)) => println!("Response: {}", v),
                Ok(None) => println!("No result"),
                Err(e) => println!("{:?}", e),
            }
        }
        None => println!("No expression or function provided"),
    }
}
