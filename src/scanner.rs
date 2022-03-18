pub struct Scanner<'a> {
    source: &'a [u8],
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner<'_> {
    pub fn new(source: &[u8]) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        let c = self.advance();

        match c {
            b'(' => self.make_token(TokenKind::LeftParen),
            b')' => self.make_token(TokenKind::RightParen),
            b'{' => self.make_token(TokenKind::LeftBrace),
            b'}' => self.make_token(TokenKind::LeftBrace),
            b';' => self.make_token(TokenKind::Semicolon),
            b',' => self.make_token(TokenKind::Comma),
            b'.' => self.make_token(TokenKind::Dot),
            b'-' => self.make_token(TokenKind::Minus),
            b'+' => self.make_token(TokenKind::Plus),
            b'/' => self.make_token(TokenKind::Slash),
            b'*' => self.make_token(TokenKind::Star),
            b'!' => {
                let matches = self.matches(b'=');
                self.make_token(
                    if matches {
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    })
            }
            b'=' => {
                let matches = self.matches(b'=');
                self.make_token(
                    if matches {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    })
            }
            b'<' => {
                let matches = self.matches(b'=');
                self.make_token(
                    if matches {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    })
            }
            b'>' => {
                let matches = self.matches(b'=');
                self.make_token(
                    if matches {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    })
            }
            b'"' => self.string(),
            c if self.is_alpha(c) => self.identifier(),
            c if self.is_digit(c) => self.number(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }

    fn is_alpha(&self, c: u8) -> bool {
        (b'a'..=b'z').contains(&c) ||
            (b'A'..=b'Z').contains(&c) ||
            c == b'_'
    }

    fn is_digit(&self, c: u8) -> bool {
        (b'0'..=b'9').contains(&c)
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source[self.current + 1]
    }

    fn matches(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        // let length = self.current - self.start;
        Token {
            kind,
            lexeme: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token {
        Token {
            kind: TokenKind::Error,
            lexeme: message.as_bytes(),
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                b' ' => {
                    self.advance();
                }
                b'\r' => {
                    self.advance();
                }
                b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        // A comment goes until the end of the line.
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        // The closing quote.
        self.advance();
        self.make_token(TokenKind::String)
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }
        self.make_token(self.identifier_kind())
    }

    fn identifier_kind(&self) -> TokenKind {
        match self.source[self.start] {
            b'a' => self.check_keyword(1, 2, b"nd", TokenKind::And),
            b'c' => self.check_keyword(1, 4, b"lass", TokenKind::Class),
            b'e' => self.check_keyword(1, 3, b"lse", TokenKind::Else),
            b'f' => {
                if self.current - self.start > 1 {
                    return match self.source[self.start + 1] {
                        b'a' => self.check_keyword(2, 3, b"lse", TokenKind::False),
                        b'o' => self.check_keyword(2, 1, b"r", TokenKind::For),
                        b'u' => self.check_keyword(2, 1, b"n", TokenKind::Fun),
                        _ => TokenKind::Identifier
                    };
                }
                TokenKind::Identifier
            }
            b'i' => self.check_keyword(1, 1, b"f", TokenKind::If),
            b'n' => self.check_keyword(1, 2, b"il", TokenKind::Nil),
            b'o' => self.check_keyword(1, 1, b"r", TokenKind::Or),
            b'p' => self.check_keyword(1, 4, b"rint", TokenKind::Print),
            b'r' => self.check_keyword(1, 5, b"eturn", TokenKind::Return),
            b's' => self.check_keyword(1, 4, b"uper", TokenKind::Super),
            b't' => {
                if self.current - self.start > 1 {
                    return match self.source[self.start + 1] {
                        b'h' => self.check_keyword(2, 2, b"is", TokenKind::This),
                        b'r' => self.check_keyword(2, 2, b"ue", TokenKind::True),
                        _ => TokenKind::Identifier
                    };
                }
                TokenKind::Identifier
            }
            b'v' => self.check_keyword(1, 2, b"ar", TokenKind::Var),
            b'w' => self.check_keyword(1, 4, b"hile", TokenKind::While),
            _ => TokenKind::Identifier
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &[u8], kind: TokenKind) -> TokenKind {
        if self.current - self.start == start + length
            && self.source[(self.start + start)..(self.start + start + length)] == *rest {
            return kind;
        }

        TokenKind::Identifier
    }

    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == b'.' && self.is_digit(self.peek_next()) {
            // Consume the ".".
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }
}

pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a [u8],
    pub line: u32,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // Error.
    Error,
    // End of file.
    Eof,
}
