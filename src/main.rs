use std::str::Chars;

use self::ExpressionError::{InvalidChar, InvalidTerm, UnexpectedEof};
macro_rules! func {
    ($self:expr, $op:expr, $chain:expr) => {{
        $self.instructions.push($op);
        $self.buffer.clear();
        $chain
    }};
}

macro_rules! expr1 {
    ($s:expr, $e:expr) => {{ $s.ensure('(').and_then(|_| $e).and_then(|_| $s.ensure(')')) }};
}
fn main() {
    let source = "pdiv(81.34,x(0)*pdiv(x(3),psqrt(np.exp(np.sin(x(0)))*x(3))))*np.sin(88.10)";

    let mut scanner = Scanner::new(source);
    println!("{:?}", scanner.expr())
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
    Read(u32),
}

struct Scanner<'a> {
    source: Chars<'a>,
    buffer: String,
    instructions: Vec<OpCode>,
    stack: Vec<f64>,
    c: char,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars(),
            buffer: String::new(),
            instructions: Vec::new(),
            stack: Vec::new(),
            c: char::MIN,
        }
    }

    pub fn expr(&mut self) -> Result<(), ExpressionError> {
        if self.next() {
            if self.is_alpha() {
                self.fill_buffer();
            }
            // println!("buffer: '{}' - current: '{}'", self.buffer, self.c);

            if let Err(e) = match self.buffer.as_str() {
                "pdiv" => func!(
                    self,
                    OpCode::Pdiv,
                    self.ensure('(')
                        .and_then(|_| self.expr())
                        .and_then(|_| self.ensure(','))
                        .and_then(|_| self.expr())
                        .and_then(|_| self.ensure(')'))
                ),
                "x" => {
                    self.buffer.clear();
                    self.ensure('(')
                        .and_then(|_| {
                            while self.next() && self.is_digits() {
                                self.buffer.push(self.c);
                            }

                            match self.buffer.parse::<u32>() {
                                Ok(e) => {
                                    self.instructions.push(OpCode::Read(e));
                                    Ok(())
                                }
                                Err(_) => Err(InvalidTerm(self.buffer.clone())),
                            }
                        })
                        .and_then(|_| self.ensure(')'))
                }
                e => match e.parse::<f64>() {
                    Ok(v) => {
                        self.stack.push(v);
                        self.buffer.clear();
                        Ok(())
                    }
                    Err(_) => Err(InvalidTerm(self.buffer.clone())),
                },
            } {
                return Err(e);
            }
            Ok(())
        } else {
            Err(UnexpectedEof)
        }

        // while let Some(c) = self.source.next() {
        //     if c.is_whitespace() {
        //         continue;
        //     }

        //     // can be func name,
        //     if c.is_ascii_lowercase()
        //         || c.is_ascii_uppercase()
        //         || c.is_ascii_digit()
        //         || c == '.'
        //         || c == '_'
        //     {
        //         self.buffer.push(c);
        //         continue;
        //     }

        //     println!("{}", self.buffer);
        //     println!("{}", c);

        //     if let Err(e) = match self.buffer.as_str() {
        //         "pdiv" => {
        //             self.instructions.push(OpCode::Pdiv);
        //             self.ensure(c, '(')
        //                 .and_then(|_| self.expr())
        //                 .and_then(|_| self.ensure(c, ','))
        //                 .and_then(|_| self.expr())
        //                 .and_then(|_| self.ensure(c, ')'))
        //         }
        //         "x" => {
        //             self.instructions.push(OpCode::Read);
        //             self.ensure(c, '(')
        //                 .and_then(|_| self.expr())
        //                 .and_then(|_| self.ensure(c, ')'))
        //         }
        //         e => match e.parse::<f64>() {
        //             Ok(v) => {
        //                 self.stack.push(v);
        //                 Ok(())
        //             }
        //             Err(_) => Err(InvalidTerm(self.buffer.clone())),
        //         },
        //     } {
        //         return Err(e);
        //     }

        //     self.buffer.clear();
        // }
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
        while let Some(c) = self.source.next() {
            if c.is_whitespace() {
                continue;
            }

            self.c = c;
            return true;
        }

        return false;
    }

    fn ensure(&mut self, expected: char) -> Result<(), ExpressionError> {
        if self.c == expected {
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
        assert!(matches!(scanner.expr(), Err(UnexpectedEof)));
    }

    #[test]
    fn test_whitespace_source_code() {
        let mut scanner = Scanner::new("   \n\n\t");
        assert!(matches!(scanner.expr(), Err(UnexpectedEof)));
    }

    #[test]
    fn test_div_simple() {
        let mut scanner = Scanner::new("pdiv(12, 3)");

        match scanner.expr() {
            Ok(()) => {
                assert_eq!(scanner.instructions, vec![OpCode::Pdiv]);
                assert_eq!(scanner.stack, vec![12., 3.]);
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_x_simple() {
        let mut scanner = Scanner::new("x(12)");

        match scanner.expr() {
            Ok(()) => {
                assert_eq!(scanner.instructions, vec![OpCode::Read(12)]);
                assert!(scanner.stack.is_empty());
                assert_eq!(scanner.c, ')');
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_pdiv_comples() {
        let mut scanner = Scanner::new("pdiv(pdiv(12,3), pdiv(3,56))");

        match scanner.expr() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![OpCode::Pdiv, OpCode::Pdiv, OpCode::Pdiv]
                );
                // assert!(scanner.stack.is_empty());
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_pdiv_x() {
        let mut scanner = Scanner::new("pdiv(x(12), x(45))");

        match scanner.expr() {
            Ok(()) => {
                assert_eq!(
                    scanner.instructions,
                    vec![OpCode::Pdiv, OpCode::Read(12), OpCode::Read(45)]
                );
                assert!(scanner.stack.is_empty());
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    // #[test]
    // fn test_invalid_char() {
    //     let mut scanner = Scanner::new("pdiv 12, 3)");
    //     assert!(matches!(scanner.expr(), Err(ExpressionError::InvalidChar { .. })));
    // }

    // #[test]
    // fn test_invalid_term() {
    //     let mut scanner = Scanner::new("abc");
    //     assert!(matches!(scanner.expr(), Err(ExpressionError::InvalidTerm(_))));
    // }
}
