use std::str::Chars;

use crate::scanner::{ExprError::UnexpectedChar, TokenType::*};

struct Reader<'a> {
    chars: Chars<'a>,
    source: &'a str,
    replay: Option<char>,
    i: isize,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            i: -1,
            replay: None,
        }
    }

    fn get_lexeme(&self, start: usize, end: usize) -> &str {
        &self.source[start..=end]
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.replay {
            self.replay = None;
            return Some(c);
        }

        while let Some(c) = self.chars.next() {
            self.i += 1;

            if c.is_whitespace() {
                continue;
            }

            return Some(c);
        }

        None
    }
}

pub struct Scanner<'a> {
    reader: Reader<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            reader: Reader::new(source),
        }
    }

    fn s_token(&self, type_: TokenType) -> Option<Result<TokenType, ExprError>> {
        Some(Ok(type_))
    }

    fn number(&mut self) -> Option<Result<TokenType, ExprError>> {
        let start = self.reader.i as usize;
        let mut end = start;
        while let Some(c) = self.reader.next() {
            if c.is_ascii_digit() {
                end += 1;
                continue;
            }

            if c == '.' {
                end += 1;
                continue;
            }

            self.reader.replay = Some(c);
            break;
        }

        match self.reader.get_lexeme(start, end).parse::<f64>() {
            Ok(e) => self.s_token(Number(e)),
            Err(_) => todo!(),
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<TokenType, ExprError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.reader.next() {
            return match c {
                '(' => self.s_token(LeftParen),
                ')' => self.s_token(RightParen),
                ',' => self.s_token(Comma),
                '.' => self.s_token(Dot),
                '-' => self.s_token(Minus),
                '+' => self.s_token(Plus),
                ';' => self.s_token(Semicolon),
                '*' => self.s_token(Star),
                '/' => self.s_token(Slash),
                c => {
                    if c.is_ascii_digit() {
                        self.number()
                    } else {
                        Some(Err(UnexpectedChar(c)))
                    }
                }
            };
        }

        None
    }
}

#[derive(Debug)]
pub enum TokenType {
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
}

#[cfg(test)]
mod test {
    use crate::scanner::{Reader, Scanner, TokenType::*};

    #[test]
    fn test_reader() {
        let mut reader = Reader::new("( / / / -- ) [ + - 123.34 ");

        assert!(matches!(reader.next(), Some('(')));
        assert_eq!(reader.i, 0);

        [2, 4, 6].into_iter().for_each(|e| {
            assert!(matches!(reader.next(), Some('/')));
            assert_eq!(reader.i, e);
        });

        [8, 9].into_iter().for_each(|e| {
            assert!(matches!(reader.next(), Some('-')));
            assert_eq!(reader.i, e);
        });

        assert!(matches!(reader.next(), Some(')')));
        assert_eq!(reader.i, 11);

        assert!(matches!(reader.next(), Some('[')));
        assert_eq!(reader.i, 13);

        assert!(matches!(reader.next(), Some('+')));
        assert_eq!(reader.i, 15);

        assert!(matches!(reader.next(), Some('-')));
        assert_eq!(reader.i, 17);

        assert!(matches!(reader.next(), Some('1')));
        assert_eq!(reader.i, 19);

        assert!(matches!(reader.next(), Some('2')));
        assert_eq!(reader.i, 20);

        assert!(matches!(reader.next(), Some('3')));
        assert_eq!(reader.i, 21);

        assert!(matches!(reader.next(), Some('.')));
        assert_eq!(reader.i, 22);

        assert!(matches!(reader.next(), Some('3')));
        assert_eq!(reader.i, 23);

        assert!(matches!(reader.next(), Some('4')));
        assert_eq!(reader.i, 24);

        assert!(matches!(reader.next(), None));
        assert_eq!(reader.i, 25);
    }

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
        println!("{}", scanner.reader.i);
        assert!(matches!(scanner.next(), Some(Ok(Plus))));
        println!("{}", scanner.reader.i);
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

    // #[test]
    // fn test_sample_formula() {
    //     let source = "1 + 2 / 3";

    //     assert!(matches!(
    //         scanner.next(),
    //         Some(Ok(Token {
    //             type_: b,
    //             start: 0,
    //             end: 1
    //         }))
    //     ))
    // }
}
