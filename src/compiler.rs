use crate::scanner::{
    ExprError::{self, *},
    Scanner,
    Token::{self, *},
};

use crate::compiler::OpCode::*;

macro_rules! binary {
    ($self:ident, $method:ident | $($pat:pat => $op:tt),*) => {{
        let mut expr = $self.$method()?;
        while let Some(current) = $self.scanner.advance() {
            let current = current?;
            match current {
                $(
                    $pat => {
                        let right = $self.$method()?;
                        expr.extend(right);
                        expr.push($op);
                    }
                ),*
                _ => return Ok((Some(current), expr)),
            }
        }

        Ok((None, expr))
    }};
}

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            current: None,
        }
    }

    pub fn compile(&mut self) -> Result<Vec<OpCode>, ExprError<'a>> {
        // while let Some(token) = self.scanner.advance() {
        // let token = token?;
        // println!("{:?}", token);
        // }

        self.expr();
        todo!()
    }

    fn expr(&mut self) -> Result<(Option<Token<'a>>, Vec<OpCode>), ExprError<'a>> {
        self.term()
    }

    fn term(&mut self) -> Result<(Option<Token<'a>>, Vec<OpCode>), ExprError<'a>> {
        let (mut pending, mut expr) = self.factor()?;

        loop {
            let current = match pending.take() {
                Some(t) => t,
                None => match self.scanner.advance() {
                    Some(Ok(t)) => t,
                    Some(Err(e)) => return Err(e),
                    None => return Ok((None, expr)),
                },
            };

            match current {
                Plus => {
                    let (t, right) = self.factor()?;
                    pending = t;
                    expr.extend(right);
                    expr.push(Add);
                }
                Minus => {
                    let (t, right) = self.factor()?;
                    pending = t;
                    expr.extend(right);
                    expr.push(Sub);
                }
                _ => return Ok((Some(current), expr)),
            }
        }
    }

    fn factor(&mut self) -> Result<(Option<Token<'a>>, Vec<OpCode>), ExprError<'a>> {
        binary!(self, unary | Star => Mult, Slash => Div)
    }

    fn unary(&mut self) -> Result<Vec<OpCode>, ExprError<'a>> {
        self.primary()
    }

    fn primary(&mut self) -> Result<Vec<OpCode>, ExprError<'a>> {
        if let Some(current) = self.scanner.advance() {
            let current = current?;

            return match current {
                Number(number) => Ok(vec![Const(number)]),
                LeftParen => {
                    let (current, result) = self.expr()?;

                    match current {
                        Some(RightParen) => Ok(result),
                        Some(e) => Err(UnexpectedToken(e)),
                        None => Err(UnexpectedEnd),
                    }
                }
                _ => Err(UnexpectedToken(current)),
            };
        }

        Err(UnexpectedEnd)
    }
}

#[derive(Debug, PartialEq)]
pub enum OpCode {
    Const(f64),
    Add,
    Sub,
    Mult,
    Div,
}

#[cfg(test)]
mod test {
    use crate::{
        compiler::{
            Compiler,
            OpCode::{self, *},
        },
        scanner::{
            ExprError::{self, *},
            Token::*,
        },
    };

    macro_rules! assert_comp {
        ($method:ident | $source:expr => $expected:expr) => {{
            let mut compiler = Compiler::new($source);
            match compiler.$method() {
                Ok(v) => assert_eq!(v, $expected),
                Err(e) => panic!("{:?}", e),
            }
        }};
    }

    #[test]
    fn test_primary() {
        let mut compiler = Compiler::new("(123x");
        assert!(matches!(
            compiler.primary(),
            Err(UnexpectedToken(Identifier("x")))
        ));

        assert_comp!(primary | "123" => vec![Const(123.)]);
        assert_comp!(primary | "(123)" =>  vec![Const(123.)]);
    }

    #[test]
    fn test_term() {
        assert_comp!(term | "1 + 2" => (None, vec![Const(1.), Const(2.), Add]));
        assert_comp!(term | "1 * 2 + 3" => (None, vec![Const(1.), Const(2.), Mult, Const(3.), Add]));
    }
}
