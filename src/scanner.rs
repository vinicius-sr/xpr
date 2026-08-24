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

    fn take_until<F>(&mut self, condition: F) -> (usize, usize)
    where
        F: Fn(char) -> bool,
    {
        let start = self.reader.index();
        let mut end = start;
        while let Some(c) = self.reader.peek() {
            if condition(c) {
                self.reader.advance();
                end += c.len_utf8();
                continue;
            }

            break;
        }

        (start, end)
    }

    fn identifier(&mut self) -> Result<Token<'a>, ExprError<'a>> {
        let (start, end) = self.take_until(|c| c.is_alphanumeric() || c == '_');

        let lexeme = self.reader.get_lexeme(start, end);
        Ok(match lexeme {
            "if" => If,
            "for" => For,
            _ => Identifier(lexeme),
        })
    }

    fn number(&mut self) -> Result<Token<'a>, ExprError<'a>> {
        let (start, end) = self.take_until(|c| c.is_ascii_digit() || c == '.');
        let number = self.reader.get_lexeme(start, end);
        match number.parse::<f64>() {
            Ok(e) => Ok(Number(e)),
            Err(_) => Err(InvalidNumber(number)),
        }
    }

    pub fn advance(&mut self) -> Option<Result<Token<'a>, ExprError<'a>>> {
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
                '{' => Ok(LeftBrace),
                '}' => Ok(RightBrace),
                '=' => {
                    if self.reader.peek() == Some('=') {
                        self.reader.advance();
                        Ok(EqualEqual)
                    } else {
                        Ok(Equal)
                    }
                }
                '!' => {
                    if self.reader.peek() == Some('=') {
                        self.reader.advance();
                        Ok(NotEqual)
                    } else {
                        Ok(Bang)
                    }
                }
                '>' => {
                    if self.reader.peek() == Some('=') {
                        self.reader.advance();
                        Ok(GreaterEqual)
                    } else {
                        Ok(Greater)
                    }
                }
                '<' => {
                    if self.reader.peek() == Some('=') {
                        self.reader.advance();
                        Ok(LessEqual)
                    } else {
                        Ok(Less)
                    }
                }
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

#[derive(Debug, PartialEq)]
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
    Equal,
    EqualEqual,
    NotEqual,
    Bang,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Number(f64),
    Identifier(&'a str),
    If,
    For,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, PartialEq)]
pub enum ExprError<'a> {
    UnexpectedChar(char),
    InvalidNumber(&'a str),
    UnexpectedToken(Token<'a>),
    InvalidFunction(&'a str),
    UnexpectedEnd,
    InvalidStack,
    DivisionByZero,
}

#[cfg(test)]
mod test {
    use crate::scanner::{ExprError::*, Scanner, Token::*};

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
            ("!", Bang),
            ("=", Equal),
            ("==", EqualEqual),
            (">", Greater),
            (">=", GreaterEqual),
            ("<", Less),
            ("<=", LessEqual),
            ("if", If),
            ("for", For),
            ("{", LeftBrace),
            ("}", RightBrace),
        ];

        source.into_iter().for_each(|(c, b)| {
            let mut scanner = Scanner::new(c);
            assert_eq!(scanner.advance(), Some(Ok(b)));
            assert_eq!(scanner.advance(), None);
        });

        let mut scanner = Scanner::new("= ==");
        assert_eq!(scanner.advance(), Some(Ok(Equal)));
        assert_eq!(scanner.advance(), Some(Ok(EqualEqual)));
        assert_eq!(scanner.advance(), None);

        let mut scanner = Scanner::new("== !=");
        assert_eq!(scanner.advance(), Some(Ok(EqualEqual)));
        assert_eq!(scanner.advance(), Some(Ok(NotEqual)));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_two_char_tokens_consume_both_chars() {
        let source = vec![
            ("==", EqualEqual),
            ("!=", NotEqual),
            (">=", GreaterEqual),
            ("<=", LessEqual),
        ];

        source.into_iter().for_each(|(c, expected)| {
            let mut scanner = Scanner::new(c);
            assert_eq!(scanner.advance(), Some(Ok(expected)));
            assert_eq!(scanner.advance(), None);
        });
    }

    #[test]
    fn test_comparison_tokens_in_sequence() {
        let source = "a == b != c >= d <= e";
        let mut scanner = Scanner::new(source);
        assert_eq!(scanner.advance(), Some(Ok(Identifier("a"))));
        assert_eq!(scanner.advance(), Some(Ok(EqualEqual)));
        assert_eq!(scanner.advance(), Some(Ok(Identifier("b"))));
        assert_eq!(scanner.advance(), Some(Ok(NotEqual)));
        assert_eq!(scanner.advance(), Some(Ok(Identifier("c"))));
        assert_eq!(scanner.advance(), Some(Ok(GreaterEqual)));
        assert_eq!(scanner.advance(), Some(Ok(Identifier("d"))));
        assert_eq!(scanner.advance(), Some(Ok(LessEqual)));
        assert_eq!(scanner.advance(), Some(Ok(Identifier("e"))));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_single_char_not_overconsumed() {
        let source = "= ! > < 5";
        let mut scanner = Scanner::new(source);
        assert_eq!(scanner.advance(), Some(Ok(Equal)));
        assert_eq!(scanner.advance(), Some(Ok(Bang)));
        assert_eq!(scanner.advance(), Some(Ok(Greater)));
        assert_eq!(scanner.advance(), Some(Ok(Less)));
        assert_eq!(scanner.advance(), Some(Ok(Number(5.))));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_identifier() {
        let source = "xabc";
        let mut scanner = Scanner::new(source);
        assert!(matches!(scanner.advance(), Some(Ok(Identifier("xabc")))))
    }

    #[test]
    fn test_identifier_2() {
        let source = "xabc atest 123 x_2 x2";
        let mut scanner = Scanner::new(source);
        assert!(matches!(scanner.advance(), Some(Ok(Identifier("xabc")))));
        assert!(matches!(scanner.advance(), Some(Ok(Identifier("atest")))));
        assert!(matches!(scanner.advance(), Some(Ok(Number(123.)))));
        assert!(matches!(scanner.advance(), Some(Ok(Identifier("x_2")))));
        assert!(matches!(scanner.advance(), Some(Ok(Identifier("x2")))));
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
    fn test_number_stops_at_operator_boundary() {
        let source = "123+ - 4";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.advance(), Some(Ok(Number(123.))));
        assert_eq!(scanner.advance(), Some(Ok(Plus)));
        assert_eq!(scanner.advance(), Some(Ok(Minus)));
        assert_eq!(scanner.advance(), Some(Ok(Number(4.))));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_number_stops_at_identifier_boundary() {
        let source = "123abc";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.advance(), Some(Ok(Number(123.))));
        assert_eq!(scanner.advance(), Some(Ok(Identifier("abc"))));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_invalid_number_multiple_dots() {
        let source = "1.2.3";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.advance(), Some(Err(InvalidNumber("1.2.3"))));
    }

    #[test]
    fn test_multibyte_whitespace_before_number() {
        let source = "\u{a0}123";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.advance(), Some(Ok(Number(123.))));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_multibyte_identifier() {
        let source = "café";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.advance(), Some(Ok(Identifier("café"))));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_if_and_identifier() {
        let source = "i if i";
        let mut scanner = Scanner::new(source);
        assert_eq!(scanner.advance(), Some(Ok(Identifier("i"))));
        assert_eq!(scanner.advance(), Some(Ok(If)));
        assert_eq!(scanner.advance(), Some(Ok(Identifier("i"))));
    }

    #[test]
    fn test_for_and_identifier() {
        let source = "f for e";
        let mut scanner = Scanner::new(source);
        assert_eq!(scanner.advance(), Some(Ok(Identifier("f"))));
        assert_eq!(scanner.advance(), Some(Ok(For)));
        assert_eq!(scanner.advance(), Some(Ok(Identifier("e"))));
    }
}
