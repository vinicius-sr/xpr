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
            match compiler.compile() {
                Ok(instr) => instr.iter().for_each(|f| println!("{:?}", f)),
                Err(e) => println!("{:?}", e),
            }
        }
        None => println!("No expression or function provided"),
    }
}
