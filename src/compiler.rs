use crate::scanner::{ExprError, Scanner};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::<'a>::new(source),
        }
    }

    pub fn compile(&'a mut self) -> Result<(), ExprError<'a>> {
        while let Some(token) = self.scanner.advance() {
            let token = token?;
            println!("{:?}", token);
        }

        Ok(())
    }
}
