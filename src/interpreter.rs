use std::fmt::Debug;
use std::marker::PhantomData;

use crate::{
    callable::{Blueprint, Callable},
    compiler::{
        Compiler,
        OpCode::{self, *},
    },
    scanner::ExprError::{self, *},
};

pub struct Interpreter<'a, T, U>
where
    T: Callable<U> + Blueprint<U>,
    U: Debug,
{
    op_code: Vec<OpCode<U>>,
    stack: Vec<f64>,
    ip: usize,
    callable: T,
    _phantom: PhantomData<&'a str>,
    _phantom_ops: PhantomData<U>,
}

impl<'a, T, U> Interpreter<'a, T, U>
where
    T: Callable<U> + Blueprint<U>,
    U: Debug,
{
    fn new(op_code: Vec<OpCode<U>>, callable: T) -> Self {
        Self {
            op_code,
            stack: Vec::new(),
            ip: 0,
            _phantom: PhantomData,
            _phantom_ops: PhantomData,
            callable,
        }
    }

    fn compile(source: &'a str, callable: T) -> Result<Interpreter<'a, T, U>, ExprError<'a>> {
        let mut compiler = Compiler::new(source, &callable);
        let op_code = compiler.compile()?;
        Ok(Self::new(op_code, callable))
    }

    pub fn compile_and_run(source: &'a str, callable: T) -> Result<Option<f64>, ExprError<'a>> {
        let mut interpreter = Interpreter::compile(source, callable)?;
        interpreter.run()
    }

    pub fn run(&mut self) -> Result<Option<f64>, ExprError<'a>> {
        // println!("{:?}", self.op_code);
        loop {
            match self.op_code.get(self.ip) {
                Some(op_code) => match op_code {
                    Const(value) => self.stack.push(*value),
                    Add => self.binary(|l, r| Ok(l + r))?,
                    Sub => self.binary(|l, r| Ok(l - r))?,
                    Mult => self.binary(|l, r| Ok(l * r))?,
                    Div => self.binary(|l, r| {
                        if r == 0f64 {
                            Err(DivisionByZero)
                        } else {
                            Ok(l / r)
                        }
                    })?,
                    Negate => todo!(),
                    Equal => self.binary(|l, r| Ok(f64::from(l == r)))?,
                    NotEqual => self.binary(|l, r| Ok(f64::from(l != r)))?,
                    Greater => self.binary(|l, r| Ok(f64::from(l > r)))?,
                    GreaterEqual => self.binary(|l, r| Ok(f64::from(l >= r)))?,
                    Less => self.binary(|l, r| Ok(f64::from(l < r)))?,
                    LessEqual => self.binary(|l, r| Ok(f64::from(l <= r)))?,
                    Call(_, _) => todo!(),
                },
                None => return Ok(self.stack.pop()),
            }

            self.ip += 1;
        }
    }

    fn binary<F>(&mut self, action: F) -> Result<(), ExprError<'a>>
    where
        F: FnOnce(f64, f64) -> Result<f64, ExprError<'a>>,
    {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(r), Some(l)) => {
                let result = action(l, r)?;
                self.stack.push(result);
                Ok(())
            }
            _ => Err(InvalidStack),
        }
    }
}
