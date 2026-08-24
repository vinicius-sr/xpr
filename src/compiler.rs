use crate::scanner::{
    ExprError::{self, *},
    Scanner,
    Token::{self, *},
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

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
    output: Vec<OpCode>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            current: None,
            output: Vec::new(),
        }
    }

    fn advance(&mut self) -> Option<Result<Token<'a>, ExprError<'a>>> {
        match self.current.take() {
            Some(c) => Some(Ok(c)),
            None => self.scanner.advance(),
        }
    }

    pub fn compile(&mut self) -> Result<Vec<OpCode>, ExprError<'a>> {
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
                Identifier(name) => todo!(),
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
    Negate,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Call,
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

    macro_rules! assert_ok {
        ($method:ident | $source:expr => $expected:expr) => {{
            let mut compiler = Compiler::new($source);
            match compiler.$method() {
                Ok(v) => assert_eq!(v, $expected),
                Err(e) => panic!("{:?}", e),
            }
        }};
    }

    macro_rules! assert_err {
        ($method:ident | $source:expr => $expected:expr) => {{
            let mut compiler = Compiler::new($source);
            match compiler.$method() {
                Ok(v) => panic!("Expected err, found: {:?}", v),
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
}
