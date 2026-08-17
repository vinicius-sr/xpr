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

    fn identifier(&mut self) -> Result<Token, ExprError> {
        todo!()
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
            Err(_) => Err(InvalidNumber(number)),
        }
    }

    fn advance(&mut self) -> Option<Result<Token, ExprError>> {
        while let Some(c) = self.reader.advance() {
            if c.is_whitespace() {
                continue;
            }

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
                    } else if c.is_ascii_alphabetic() {
                        self.identifier()
                    } else {
                        Err(UnexpectedChar(c))
                    }
                }
            });
        }

        None
    }
}

/*impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token, ExprError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance()
    }
}*/

#[derive(Debug)]
pub enum Token<'a> {
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
    Identifier(&'a str) ,
}

#[derive(Debug)]
pub enum ExprError<'a> {
    UnexpectedChar(char),
    InvalidNumber(&'a str),
}

#[cfg(test)]
mod test {
    use crate::scanner::{Scanner, Token::*};

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
            assert!(matches!(scanner.advance(), Some(Ok(b))))
        });
    }

    #[test]
    fn test_identifier() {
        let source = "xabc";

        // let mut scanner = Scanner::new(source);
        // assert!(matches!(scanner.advance(), Some(Ok(Identifier(.)))))
    }

    #[test]
    fn test_number() {
        let source = "123";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.advance(), Some(Ok(Number(123.)))))
    }

    #[test]
    fn test_float_number() {
        let source = "123.23";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.advance(), Some(Ok(Number(123.23)))))
    }

    #[test]
    fn test_float_number_2() {
        let source = "123.23 45.8";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.advance(), Some(Ok(Number(123.23)))));
        assert!(matches!(scanner.advance(), Some(Ok(Number(45.8)))))
    }

    #[test]
    fn test_number_rewind_1() {
        let source = "123 + ";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.advance(), Some(Ok(Number(123.)))));
        assert!(matches!(scanner.advance(), Some(Ok(Plus))));
    }

    #[test]
    fn test_number_rewind_2() {
        let source = "123+";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.advance(), Some(Ok(Number(123.)))));
        assert!(matches!(scanner.advance(), Some(Ok(Plus))))
    }

    #[test]
    fn test_number_rewind_3() {
        let source = "123+ - 4";
        let mut scanner = Scanner::new(source);

        assert!(matches!(scanner.advance(), Some(Ok(Number(123.)))));
        assert!(matches!(scanner.advance(), Some(Ok(Plus))));
        assert!(matches!(scanner.advance(), Some(Ok(Minus))));
        assert!(matches!(scanner.advance(), Some(Ok(Number(4.)))));
    }
}
