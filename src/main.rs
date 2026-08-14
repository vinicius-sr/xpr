use std::str::Chars;

fn main() {
    let source =
        "pdiv(81.34,x[:, 0]*pdiv(x[:, 3],psqrt(np.exp(np.sin(x[:, 0]))*x[:, 3])))*np.sin(88.10)";

    let mut scanner = Scanner::new(source);
    scanner.scan();
}

struct Scanner<'a> {
    source: Chars<'a>,
    buffer: String,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars(),
            buffer: String::new(),
        }
    }

    fn scan(&mut self) {
        while let Some(c) = self.source.next() {
            if c.is_whitespace() {
                continue;
            }

            if c.is_ascii_lowercase() || c.is_ascii_uppercase() {
                self.buffer.push(c);
                continue;
            }

            match self.buffer.as_str() {
                "pdiv" => {
                    let e = self.source.next();
                }
                _ => todo!("Unexpected function: {}", self.buffer),
            }

            todo!("Unexpected char: '{}', current buffer: {}", c, self.buffer)
        }
    }

    // fn ensure(&mut self, expected: char) -> bool {
    //     if let Some(current) = self.source.next() && current == expected{
            
    //     }
    // }
}
