use std::str::CharIndices;

pub struct Reader<'a> {
    chars: CharIndices<'a>,
    source: &'a str,
    current: Option<char>,
    i: isize,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices(),
            i: -1,
            current: None,
        }
    }

    pub fn index(&self) -> usize {
        self.i as usize
    }

    pub fn get_lexeme(&self, start: usize, end: usize) -> &'a str {
        &self.source[start..=end]
    }

    fn fill(&mut self) {
        if self.current.is_some() {
            return;
        }

        if let Some((i, c)) = self.chars.next() {
            self.i = i as isize;
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
            reader.advance();
            assert!(matches!(reader.advance(), Some('/')));
            assert_eq!(reader.index(), e);
        });

        reader.advance();
        [8, 9].into_iter().for_each(|e| {
            assert!(matches!(reader.advance(), Some('-')));
            assert_eq!(reader.index(), e);
        });

        reader.advance();
        assert!(matches!(reader.advance(), Some(')')));
        assert_eq!(reader.index(), 11);

        reader.advance();
        assert!(matches!(reader.advance(), Some('[')));
        assert_eq!(reader.index(), 13);

        reader.advance();
        assert!(matches!(reader.advance(), Some('+')));
        assert_eq!(reader.index(), 15);

        reader.advance();
        assert!(matches!(reader.advance(), Some('-')));
        assert_eq!(reader.index(), 17);

        reader.advance();
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

        reader.advance();
        assert!(reader.advance().is_none());
        assert_eq!(reader.index(), 25);
    }
}
