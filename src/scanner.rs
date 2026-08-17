use crate::reader::Reader;
use crate::scanner::ExprError::InvalidNumber;
use crate::scanner::{ExprError::UnexpectedChar, Token::*};

pub struct Scanner<'a> {
    reader: Reader<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            reader: Reader::new(source),
        }
    }

    fn number(&mut self) -> Result<Token, ExprError> {
        let start = self.reader.index();
        let mut end = start;
        while let Some(c) = self.reader.peek() {
            if c.is_ascii_digit() || c == '.' {
                self.reader.advance();
                end += 1;
                continue;
            }

            break;
        }

        let number = self.reader.get_lexeme(start, end);
        match number.parse::<f64>() {
            Ok(e) => Ok(Number(e)),
            Err(_) => Err(InvalidNumber(number.to_string())),
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token, ExprError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.reader.advance() {
            return Some(match c {
                '(' => Ok(LeftParen),
                ')' => Ok(RightParen),
                ',' => Ok(Comma),
                '.' => Ok(Dot),
                '-' => Ok(Minus),
                '+' => Ok(Plus),
                ';' => Ok(Semicolon),
                '*' => Ok(Star),
                '/' => Ok(Slash),
                c => {
                    if c.is_ascii_digit() {
                        self.number()
                    } else {
                        Err(UnexpectedChar(c))
                    }
                }
            });
        }

        None
    }
}

#[derive(Debug)]
pub enum Token {
    LeftParen,
    RightParen,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,
    Slash,
    Number(f64),
    Identifier { start: usize, end: usize },
}

#[derive(Debug)]
pub enum ExprError {
    UnexpectedChar(char),
    InvalidNumber(String),
}

#[cfg(test)]
mod test {
    use crate::scanner::{Scanner, Token::*};

    // #[test]
    // fn test_reader_lexeme() {
    //     let mut reader = Reader::new("123.34 xabd");
    //     assert!(matches!(reader.next(), Some('1')));
    //     reader.set_start();

    //     for _ in 0..5 {
    //         reader.next();
    //     }

    //     assert_eq!(reader.get_lexeme(), "123.34");

    //     assert!(matches!(reader.next(), Some('x')));
    //     reader.set_start();
    //     for _ in 0..4 {
    //         reader.next();
    //     }

    //     assert_eq!(reader.get_lexeme(), "xabd");
    //     assert!(matches!(reader.next(), None));
    // }

    #[test]
    fn test_simple_token() {
        let source = vec![
            ("(", LeftParen),
            (")", RightParen),
            (",", Comma),
            (".", Dot),
            ("-", Minus),
            ("+", Plus),
            (";", Semicolon),
            ("*", Star),
            ("/", Slash),
        ];

        source.into_iter().for_each(|(c, b)| {
            let mut scanner = Scanner::new(c);
            assert!(matches!(scanner.next(), Some(Ok(b))))
        });
    }

    #[test]
    fn test_number() {
        let source = "123";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.next(), Some(Ok(Number(123.)))))
    }

    #[test]
    fn test_float_number() {
        let source = "123.23";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.next(), Some(Ok(Number(123.23)))))
    }

    #[test]
    fn test_number_rewind_1() {
        let source = "123 + ";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.next(), Some(Ok(Number(123.)))));
        assert!(matches!(scanner.next(), Some(Ok(Plus))));
    }

    #[test]
    fn test_number_rewind_2() {
        let source = "123+";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.next(), Some(Ok(Number(123.)))));
        assert!(matches!(scanner.next(), Some(Ok(Plus))))
    }

    #[test]
    fn test_number_rewind_3() {
        let source = "123+ - 4";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.next(), Some(Ok(Number(123.)))));
        assert!(matches!(scanner.next(), Some(Ok(Plus))));
        assert!(matches!(scanner.next(), Some(Ok(Minus))));
        assert!(matches!(scanner.next(), Some(Ok(Number(4.)))));
    }

    // // #[test]
    // // fn test_sample_formula() {
    // //     let source = "1 + 2 / 3";

    // //     assert!(matches!(
    // //         scanner.next(),
    // //         Some(Ok(Token {
    // //             type_: b,
    // //             start: 0,
    // //             end: 1
    // //         }))
    // //     ))
    // // }
}
