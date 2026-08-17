use std::str::Chars;

pub struct Reader<'a> {
    chars: Chars<'a>,
    source: &'a str,
    current: Option<char>,
    i: isize,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            i: -1,
            current: None,
        }
    }

    pub fn index(&self) -> usize {
        self.i as usize
    }

    pub fn get_lexeme(&self, start: usize, end: usize) -> &str {
        &self.source[start..=end]
    }

    pub fn fill(&mut self) {
        if self.current.is_some() {
            return;
        }

        while let Some(c) = self.chars.next() {
            self.i += 1;

            if c.is_whitespace() {
                continue;
            }

            self.current = Some(c);
            return;
        }

        self.current = None;
    }

    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.current = None;
        Some(c)
    }

    pub fn peek(&mut self) -> Option<char> {
        self.fill();
        self.current
    }
}

#[cfg(test)]
mod test {
    use crate::reader::Reader;

    #[test]
    fn test_reader() {
        let mut reader = Reader::new("( / / / -- ) [ + - 123.34 ");

        assert!(matches!(reader.advance(), Some('(')));
        assert_eq!(reader.index(), 0);

        [2, 4, 6].into_iter().for_each(|e| {
            assert!(matches!(reader.advance(), Some('/')));
            assert_eq!(reader.index(), e);
        });

        [8, 9].into_iter().for_each(|e| {
            assert!(matches!(reader.advance(), Some('-')));
            assert_eq!(reader.index(), e);
        });

        assert!(matches!(reader.advance(), Some(')')));
        assert_eq!(reader.index(), 11);

        assert!(matches!(reader.advance(), Some('[')));
        assert_eq!(reader.index(), 13);

        assert!(matches!(reader.advance(), Some('+')));
        assert_eq!(reader.index(), 15);

        assert!(matches!(reader.advance(), Some('-')));
        assert_eq!(reader.index(), 17);

        assert!(matches!(reader.advance(), Some('1')));
        assert_eq!(reader.index(), 19);

        assert!(matches!(reader.advance(), Some('2')));
        assert_eq!(reader.index(), 20);

        assert!(matches!(reader.advance(), Some('3')));
        assert_eq!(reader.index(), 21);

        assert!(matches!(reader.advance(), Some('.')));
        assert_eq!(reader.index(), 22);

        assert!(matches!(reader.advance(), Some('3')));
        assert_eq!(reader.index(), 23);

        assert!(matches!(reader.advance(), Some('4')));
        assert_eq!(reader.index(), 24);

        assert!(matches!(reader.advance(), None));
        assert_eq!(reader.index(), 25);
    }
}
