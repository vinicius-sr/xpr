mod compiler;
mod reader;
mod scanner;

use std::env;

use crate::compiler::Compiler;

fn main() {
    match env::args().nth(1) {
        Some(source) => {
            println!("Received: {}", source);
            let mut compiler = Compiler::new(source.as_str());
            if let Err(e) = compiler.compile() {
                println!("{:?}", e);
            }
        }
        None => println!("No expression or function provided"),
    }
}
