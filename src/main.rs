mod scanner;

use std::env;

use crate::scanner::Scanner;

fn main() {
    match env::args().nth(1) {
        Some(source) => {
            println!("Received: {}", source);
            let mut scanner = Scanner::new(source.as_str());
        },
        None => println!("No expression or function provided"),
    }
}