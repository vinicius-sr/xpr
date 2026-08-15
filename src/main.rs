use std::str::Chars;

use self::ExpressionError::{InvalidChar, InvalidTerm, UnexpectedEof};
macro_rules! func {
    ($self:expr, $op:expr, $chain:expr) => {{
        $self.instructions.push($op);
        $self.buffer.clear();
        $chain
    }};
}

fn main() {
    let source = "pdiv(81.34,x(0)*pdiv(x(3),psqrt(np.exp(np.sin(x(0)))*x(3))))*np.sin(88.10)";

    let mut scanner = Scanner::new(source);
    println!("{:?}", scanner.scan());
    println!("{:?}", scanner.instructions)
}

#[derive(Debug)]
enum ExpressionError {
    InvalidChar {
        current: char,
        expected: char,
        buffer: String,
    },
    UnexpectedEof,
    InvalidTerm(String),
}

#[derive(Debug, PartialEq)]
enum OpCode {
    Pdiv,
    Mult,
    Add,
    Sub,
    Read(u32),
    Const(f64),
}

struct Scanner<'a> {
    source: Chars<'a>,
    buffer: String,
    instructions: Vec<OpCode>,
    c: char,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars(),
            buffer: String::new(),
            instructions: Vec::new(),
            c: char::MIN,
        }
    }

    pub fn scan(&mut self) -> Result<(), ExpressionError> {
        if self.c == char::MIN {
            if !self.next() {
                return Err(UnexpectedEof);
            }
        }

        self.expr().and_then(|_| {
            if self.c == char::MIN {
                Ok(())
            } else {
                self.expr()
            }
        })
    }

    fn expr(&mut self) -> Result<(), ExpressionError> {
        println!("expr:{}", self.c);
        if self.is_alpha() {
            self.fill_buffer();
        }
        println!("buffer: '{}' - current: '{}'", self.buffer, self.c);

        match self.c {
            '*' => self.operator(OpCode::Mult),
            '+' => self.operator(OpCode::Add),
            '-' => self.operator(OpCode::Sub),
            _ => self.factor(),
        }
    }

    fn operator(&mut self, op_code: OpCode) -> Result<(), ExpressionError> {
        println!("operator {:?} - {}", op_code, self.buffer);
        self.instructions.push(op_code);
        self.factor().and_then(|_| self.expr())
    }

    fn move_or_eof(&mut self) -> Result<(), ExpressionError> {
        println!("moeof: {}", self.c);
        if self.next() {
            self.expr()
        } else {
            Err(UnexpectedEof)
        }
    }

    fn factor(&mut self) -> Result<(), ExpressionError> {
        println!("sf: {} - buf:{}", self.c, self.buffer);
        if let Err(e) = match self.buffer.as_str() {
            "pdiv" => func!(
                self,
                OpCode::Pdiv,
                self.consume('(')
                    .and_then(|_| self.expr())
                    .and_then(|_| self.consume(','))
                    .and_then(|_| self.expr())
                    .and_then(|_| self.consume(')'))
            ),
            "x" => {
                self.buffer.clear();
                self.consume('(')
                    .and_then(|_| {
                        self.buffer.push(self.c);
                        while self.next() && self.is_digits() {
                            self.buffer.push(self.c);
                        }

                        match self.buffer.parse::<u32>() {
                            Ok(e) => {
                                self.instructions.push(OpCode::Read(e));
                                self.buffer.clear();
                                Ok(())
                            }
                            Err(_) => Err(InvalidTerm(self.buffer.clone())),
                        }
                    })
                    .and_then(|_| self.consume(')'))
            }
            e => match e.parse::<f64>() {
                Ok(v) => {
                    self.instructions.push(OpCode::Const(v));
                    self.buffer.clear();
                    Ok(())
                }
                Err(_) => Err(InvalidTerm(self.buffer.clone())),
            },
        } {
            return Err(e);
        }

        println!("ef: {} - buf:{}", self.c, self.buffer);
        Ok(())
    }

    fn fill_buffer(&mut self) {
        self.buffer.push(self.c);
        while self.next() && self.is_alpha() {
            self.buffer.push(self.c);
        }
    }

    fn is_alpha(&self) -> bool {
        self.c.is_ascii_lowercase()
            || self.c.is_ascii_uppercase()
            || self.c.is_ascii_digit()
            || self.c == '.'
            || self.c == '_'
    }

    fn is_digits(&self) -> bool {
        self.c.is_ascii_digit()
    }

    fn next(&mut self) -> bool {
        println!("moving...");
        while let Some(c) = self.source.next() {
            if c.is_whitespace() {
                continue;
            }

            self.c = c;
            return true;
        }

        self.c = char::MIN;
        return false;
    }

    fn consume(&mut self, expected: char) -> Result<(), ExpressionError> {
        if self.c == expected {
            self.next();
            // println!("Ensure '{}' - GOOD", expected);
            Ok(())
        } else {
            Err(InvalidChar {
                current: self.c,
                expected,
                buffer: self.buffer.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;
    use crate::{ExpressionError::UnexpectedEof, OpCode};

    #[test]
    fn test_empty_source_code() {
        let mut scanner = Scanner::new("");
        assert!(matches!(scanner.scan(), Err(UnexpectedEof)));
    }

    #[test]
    fn test_whitespace_source_code() {
        let mut scanner = Scanner::new("   \n\n\t");
        assert!(matches!(scanner.scan(), Err(UnexpectedEof)));
    }

    #[test]
    fn test_div_simple() {
        let mut scanner = Scanner::new("pdiv(12, 3)");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![OpCode::Pdiv, OpCode::Const(12.), OpCode::Const(3.)]
                );
                assert_eq!(scanner.c, ')')
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_x_simple() {
        let mut scanner = Scanner::new("x(12)");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(scanner.instructions, vec![OpCode::Read(12)]);
                assert_eq!(scanner.c, ')');
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_pdiv_complex() {
        let mut scanner = Scanner::new("pdiv(pdiv(12,3), pdiv(3,56))");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![
                        OpCode::Pdiv,
                        OpCode::Pdiv,
                        OpCode::Const(12.),
                        OpCode::Const(3.),
                        OpCode::Pdiv,
                        OpCode::Const(3.),
                        OpCode::Const(56.)
                    ]
                );
                // assert!(scanner.stack.is_empty());
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_pdiv_x() {
        let mut scanner = Scanner::new("pdiv(x(12), x(45))");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![OpCode::Pdiv, OpCode::Read(12), OpCode::Read(45)]
                );
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_binary() {
        let mut scanner = Scanner::new("10 * 12");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![OpCode::Mult, OpCode::Const(10.), OpCode::Const(12.)]
                );
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_binary_2() {
        let mut scanner = Scanner::new("10 * 12 + 15");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![
                        OpCode::Mult,
                        OpCode::Const(10.),
                        OpCode::Add,
                        OpCode::Const(12.),
                        OpCode::Const(15.)
                    ]
                );
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_binary_with_func() {
        let mut scanner = Scanner::new("x(3) - 2.4");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![OpCode::Sub, OpCode::Read(3), OpCode::Const(2.4)]
                );
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_binary_with_func_complex() {
        let mut scanner = Scanner::new("10 * 12 + x(3) - pdiv(x(3), 90.45)");

        match scanner.scan() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![
                        OpCode::Mult,
                        OpCode::Const(10.),
                        OpCode::Add,
                        OpCode::Const(12.),
                        OpCode::Sub,
                        OpCode::Read(3),
                        OpCode::Pdiv,
                        OpCode::Read(3),
                        OpCode::Const(90.45)
                    ]
                );
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}
