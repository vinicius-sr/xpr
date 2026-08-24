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
                    Call(id, arity) => {
                        let len = self.stack.len();
                        if len < *arity {
                            return Err(InvalidStack);
                        }
                        let result = self.callable.call(id, &self.stack[len - *arity..]);
                        self.stack.truncate(len - *arity);
                        self.stack.push(result);
                    }
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

#[cfg(test)]
mod test {
    use crate::{
        Math,
        interpreter::Interpreter,
        scanner::ExprError::*,
    };

    macro_rules! assert_ok {
        ($source:expr => $expected:expr) => {{
            match Interpreter::compile_and_run($source, Math) {
                Ok(v) => assert_eq!(v, Some($expected)),
                Err(e) => panic!("Expected ok, found: {:?}", e),
            }
        }};
    }

    macro_rules! assert_err {
        ($source:expr => $expected:expr) => {{
            match Interpreter::compile_and_run($source, Math) {
                Ok(v) => panic!("Expected err, found: {:?}", v),
                Err(e) => assert_eq!(e, $expected),
            }
        }};
    }

    #[test]
    fn test_arithmetic() {
        assert_ok!("42" => 42.);
        assert_ok!("1 + 2" => 3.);
        assert_ok!("1 - 2 + 3" => 2.);
        assert_ok!("2 * 3 - 1" => 5.);
        assert_ok!("(1 + 2) * 3" => 9.);
    }

    #[test]
    fn test_comparison() {
        assert_ok!("1 < 2" => 1.);
        assert_ok!("2 < 1" => 0.);
        assert_ok!("2 == 2" => 1.);
        assert_ok!("2 != 2" => 0.);
        assert_ok!("3 >= 3" => 1.);
        assert_ok!("1 <= 2" => 1.);
    }

    #[test]
    fn test_calls() {
        assert_ok!("sum(1.0, 2.0)" => 3.);
        assert_ok!("sub(5.0, 2.0)" => 3.);
        assert_ok!("read(7)" => 7.);
        assert_ok!("sub(sum(1.0, 2.0), 0.5)" => 2.5);
        assert_ok!("sum(1.0, 2.0) * 2" => 6.);
    }

    #[test]
    fn test_complex_calls() {
        assert_ok!("sum(sub(10.0, 2.5), read(3))" => 10.5);
        assert_ok!("sub(read(10), sum(1.0, 2.0))" => 7.);
        assert_ok!("sum(1.0, 2.0) * sub(4.0, 1.0) + read(1)" => 10.);
        assert_ok!("2 * sum(1.0, 2.0) - sub(5.0, 2.0)" => 3.);
        assert_ok!("sum(1.0, 2.0) / sub(4.0, 1.0)" => 1.);
        assert_ok!("(sum(1.0, 2.0)) + (read(4))" => 7.);
    }

    #[test]
    fn test_calls_in_comparisons() {
        assert_ok!("sum(1.0, 2.0) > 2.5" => 1.);
        assert_ok!("sub(read(10), sum(1.0, 2.0)) < 8." => 1.);
        assert_ok!("sub(sum(1.0, 2.0), 3.0) == 0.0" => 1.);
        assert_ok!("sum(1.0, 2.0) != read(3)" => 0.);
        assert_ok!("read(5) >= sub(7.0, 2.0)" => 1.);
    }

    #[test]
    fn test_expressions_as_arguments() {
        assert_ok!("sum(3 / 8 + 12 * 8, sub(4., 6 / 7))" => 3. / 8. + 12. * 8. + (4. - 6. / 7.));
        assert_ok!("sum(3 / 4 + 12 * 8, sub(4., 6 / 8))" => 100.);
        assert_ok!("sub(sum(1 + 2, 3 * 4), read(10))" => 5.);
        assert_ok!("sum(1 / 2 + 1 / 4, sub(3 / 4, 1 / 8))" => 1.375);
        assert_ok!("sub(10 - 2 / 4, 3 * 2 + 1)" => 2.5);
        assert_ok!("sum(sub(read(20), sum(1 + 1, 2)), 3 / 4)" => 16.75);
        assert_ok!("read(sum(1, 2))" => 3.);
    }

    #[test]
    fn test_errors() {
        assert_err!("1 / 0" => DivisionByZero);
        assert_err!("foo(1.0)" => InvalidFunction("foo"));
        assert_err!("sum(1.0)" => ArityMismatch(2, 1));
        assert_err!("sub(sum(1.0, 2.0))" => ArityMismatch(2, 1));
    }
}
