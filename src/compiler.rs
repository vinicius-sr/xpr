use crate::{
    callable::MethodInfo,
    scanner::{
        ExprError::{self, *},
        Scanner,
        Token::{self, *},
    },
};

use crate::compiler::OpCode::Equal as OpEqual;
use crate::compiler::OpCode::Greater as OpGreater;
use crate::compiler::OpCode::GreaterEqual as OpGreaterEqual;
use crate::compiler::OpCode::Less as OpLess;
use crate::compiler::OpCode::LessEqual as OpLessEqual;
use crate::compiler::OpCode::NotEqual as OpNotEqual;
use crate::compiler::OpCode::*;

macro_rules! binary {
    ($self:ident, $method:ident | $($pat:pat => $op:tt),*) => {{
        $self.$method()?;
        while let Some(current) = $self.advance() {
            let current = current?;
            match current {
                $(
                    $pat => {
                        $self.$method()?;
                        $self.output.push($op);
                    }
                ),*
                _ => {
                    $self.current = Some(current);
                    return Ok(());
                }
            }
        }

        Ok(())
    }};
}

pub struct Compiler<'a, F, U>
where
    F: Fn(&str) -> Option<MethodInfo<U>>,
{
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
    output: Vec<OpCode<U>>,
    find: F,
}

impl<'a, F, U> Compiler<'a, F, U>
where
    F: Fn(&str) -> Option<MethodInfo<U>>,
{
    pub fn new(source: &'a str, find: F) -> Self {
        Self {
            scanner: Scanner::new(source),
            current: None,
            output: Vec::new(),
            find,
        }
    }

    fn advance(&mut self) -> Option<Result<Token<'a>, ExprError<'a>>> {
        match self.current.take() {
            Some(c) => Some(Ok(c)),
            None => self.scanner.advance(),
        }
    }

    pub fn compile(&mut self) -> Result<Vec<OpCode<U>>, ExprError<'a>> {
        self.block()?;

        match self.advance() {
            Some(Ok(token)) => Err(UnexpectedToken(token)),
            Some(Err(e)) => Err(e),
            None => Ok(std::mem::take(&mut self.output)),
        }
    }

    fn expr(&mut self) -> Result<(), ExprError<'a>> {
        self.equality()
    }

    fn block(&mut self) -> Result<(), ExprError<'a>> {
        match self.advance() {
            Some(Ok(LeftBrace)) => todo!("Block instructions are not implemented yet."),
            Some(c) => {
                self.current = Some(c?);
                self.expr()
            }
            None => Err(UnexpectedEnd),
        }
    }

    fn equality(&mut self) -> Result<(), ExprError<'a>> {
        binary!(self, comparison | Token::EqualEqual => OpEqual,  Token::NotEqual => OpNotEqual)
    }

    fn comparison(&mut self) -> Result<(), ExprError<'a>> {
        binary!(self, term | Token::Greater => OpGreater, Token::GreaterEqual => OpGreaterEqual, Token::Less => OpLess, Token::LessEqual => OpLessEqual)
    }

    fn term(&mut self) -> Result<(), ExprError<'a>> {
        binary!(self, factor | Plus => Add, Minus => Sub)
    }

    fn factor(&mut self) -> Result<(), ExprError<'a>> {
        binary!(self, unary | Star => Mult, Slash => Div)
    }

    fn unary(&mut self) -> Result<(), ExprError<'a>> {
        if let Some(current) = self.advance() {
            let current = current?;
            match current {
                Minus | Bang => {
                    self.primary()?;
                    self.output.push(Negate);
                    Ok(())
                }
                _ => {
                    self.current = Some(current);
                    self.primary()
                }
            }
        } else {
            Err(UnexpectedEnd)
        }
    }

    fn primary(&mut self) -> Result<(), ExprError<'a>> {
        if let Some(current) = self.advance() {
            let current = current?;

            return match current {
                Number(number) => {
                    self.output.push(Const(number));
                    Ok(())
                }
                LeftParen => {
                    self.expr()?;

                    match self.current.take() {
                        Some(RightParen) => Ok(()),
                        Some(e) => Err(UnexpectedToken(e)),
                        None => Err(UnexpectedEnd),
                    }
                }
                Identifier(name) => match (self.find)(name) {
                    Some(info) => self.call(info),
                    None => Err(InvalidFunction(name)),
                },
                _ => Err(UnexpectedToken(current)),
            };
        }

        Err(UnexpectedEnd)
    }

    fn call(&mut self, info: MethodInfo<U>) -> Result<(), ExprError<'a>> {
        match self.advance() {
            Some(Ok(LeftParen)) => {}
            Some(Ok(token)) => return Err(UnexpectedToken(token)),
            Some(Err(e)) => return Err(e),
            None => return Err(UnexpectedEnd),
        }

        let mut count = 0;

        loop {
            self.expr()?;
            count += 1;

            match self.advance() {
                Some(Ok(Comma)) => {}
                Some(Ok(RightParen)) => break,
                Some(Ok(token)) => return Err(UnexpectedToken(token)),
                Some(Err(e)) => return Err(e),
                None => return Err(UnexpectedEnd),
            }
        }

        if count != info.arity {
            return Err(ArityMismatch(info.arity, count));
        }

        self.output.push(Call(info.id, info.arity));
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum OpCode<U> {
    Const(f64),
    Add,
    Sub,
    Mult,
    Div,
    Negate,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Call(U, usize),
}

#[cfg(test)]
mod test {
    use crate::{
        compiler::{
            Compiler,
            OpCode::{self, *},
        },
        math::{Math, MethodId},
        scanner::{ExprError::*, Token::*},
    };

    macro_rules! assert_ok {
        ($method:ident | $source:expr => $expected:expr) => {{
            let expected: Vec<OpCode<MethodId>> = $expected;
            let mut compiler = Compiler::new($source, Math::find);
            match compiler.$method() {
                Ok(v) => assert_eq!(v, expected),
                Err(e) => panic!("{:?}", e),
            }
        }};
    }

    macro_rules! assert_err {
        ($method:ident | $source:expr => $expected:expr) => {{
            let mut compiler = Compiler::new($source, Math::find);
            match compiler.$method() {
                Ok(v) => {
                    let v: Vec<OpCode<MethodId>> = v;
                    panic!("Expected err, found: {:?}", v)
                }
                Err(e) => assert_eq!(e, $expected),
            }
        }};
    }

    #[test]
    fn test_primary() {
        assert_err!(compile | "(123x" => UnexpectedToken(Identifier("x")));
        assert_ok!(compile | "123" => vec![Const(123.)]);
        assert_ok!(compile | "(123)" =>  vec![Const(123.)]);
    }

    #[test]
    fn test_term() {
        assert_ok!(compile | "1 + 2" => vec![Const(1.), Const(2.), Add]);
        assert_ok!(compile | "1 - 2 + 3" => vec![Const(1.), Const(2.), Sub, Const(3.), Add]);
        assert_ok!(compile | "1 - 2 * 3" => vec![Const(1.), Const(2.), Const(3.), Mult, Sub]);
        assert_ok!(compile | "(1 - 2) * 3" => vec![Const(1.), Const(2.), Sub, Const(3.), Mult]);
    }

    #[test]
    fn test_unary() {
        assert_ok!(compile | "-2" => vec![Const(2.), Negate]);
        assert_ok!(compile | "!2" => vec![Const(2.), Negate]);
        assert_ok!(compile | "-(2 + 3)" => vec![Const(2.), Const(3.), Add, Negate]);
    }

    #[test]
    fn test_factor() {
        assert_ok!(compile | "2 * 3" => vec![Const(2.), Const(3.), Mult]);
        assert_ok!(compile | "2 * (3 - 5)" => vec![Const(2.), Const(3.), Const(5.), Sub, Mult]);
    }

    #[test]
    fn test_comparison() {
        assert_ok!(compile | "1 > 2" => vec![Const(1.), Const(2.), OpCode::Greater]);
        assert_ok!(compile | "1 >= 2" => vec![Const(1.), Const(2.), OpCode::GreaterEqual]);
        assert_ok!(compile | "1 < 2" => vec![Const(1.), Const(2.), OpCode::Less]);
        assert_ok!(compile | "1 <= 2" => vec![Const(1.), Const(2.), OpCode::LessEqual]);
        assert_ok!(compile | "1 > 2 >= 3" => vec![Const(1.), Const(2.), OpCode::Greater, Const(3.), OpCode::GreaterEqual]);
    }

    #[test]
    fn test_equality() {
        assert_ok!(compile | "1 == 2" => vec![Const(1.), Const(2.), OpCode::Equal]);
        assert_ok!(compile | "1 != 2" => vec![Const(1.), Const(2.), OpCode::NotEqual]);
        assert_ok!(compile | "1 == 2 != 3" => vec![Const(1.), Const(2.), OpCode::Equal, Const(3.), OpCode::NotEqual]);
    }

    #[test]
    fn test_compile() {
        assert_ok!(compile | "1 + 2 * 3 > 4" => vec![Const(1.), Const(2.), Const(3.), Mult, Add, Const(4.), OpCode::Greater]);
        assert_ok!(compile | "1 + 2 == 3 * 4" => vec![Const(1.), Const(2.), Add, Const(3.), Const(4.), Mult, OpCode::Equal]);
        assert_ok!(compile | "-(1 + 2) <= 3" => vec![Const(1.), Const(2.), Add, Negate, Const(3.), OpCode::LessEqual]);
        assert_ok!(compile | "1 < 2 == 3 != 4" => vec![Const(1.), Const(2.), OpCode::Less, Const(3.), OpCode::Equal, Const(4.), OpCode::NotEqual]);
    }

    #[test]
    fn test_compile_errors() {
        assert_err!(compile | "" => UnexpectedEnd);
        assert_err!(compile | "1 +" => UnexpectedEnd);
        assert_err!(compile | "(" => UnexpectedEnd);
        assert_err!(compile | "--2" => UnexpectedToken(Minus));
    }

    #[test]
    fn test_compile_rejects_trailing_tokens() {
        assert_err!(compile | "1 2" => UnexpectedToken(Number(2.)));
        assert_err!(compile | "(1)(2)" => UnexpectedToken(LeftParen));
        assert_err!(compile | "1 + 2 )" => UnexpectedToken(RightParen));
    }

    #[test]
    fn test_callable() {
        assert_ok!(compile | "sum(1.0, 2.0)" => vec![Const(1.), Const(2.), Call(MethodId::Sum, 2)]);
        assert_ok!(compile | "foo(7)" => vec![Const(7.), Call(MethodId::Foo, 1)]);
        assert_ok!(compile | "sub(sum(1.0, 2.0), 3.0)" => vec![Const(1.), Const(2.), Call(MethodId::Sum, 2), Const(3.), Call(MethodId::Sub, 2)]);
        assert_ok!(compile | "sum(1.0, 2.0) + 3" => vec![Const(1.), Const(2.), Call(MethodId::Sum, 2), Const(3.), Add]);
        assert_ok!(compile | "sum(1 + 2, 3 * 4)" => vec![Const(1.), Const(2.), Add, Const(3.), Const(4.), Mult, Call(MethodId::Sum, 2)]);
    }

    #[test]
    fn test_callable_errors() {
        assert_err!(compile | "bar(1.0)" => InvalidFunction("bar"));
        assert_err!(compile | "sum(1.0)" => ArityMismatch(2, 1));
        assert_err!(compile | "sum(1.0, 2.0, 3.0)" => ArityMismatch(2, 3));
        assert_err!(compile | "sum" => UnexpectedEnd);
        assert_err!(compile | "sum 1.0" => UnexpectedToken(Number(1.)));
    }
}
