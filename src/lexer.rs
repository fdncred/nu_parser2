#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a [u8],
    span_offset: usize,

    // For times when the lexer should treat newlines
    // as spaces
    pub newline_is_space: bool,
}

#[derive(Debug)]
pub enum TokenType {
    Number,
    Space,
    Newline,
    Comma,
    Comment,
    SimpleString,
    String,
    Dot,
    DotDot,
    Dollar,
    Variable,
    Pipe,
    PipePipe,
    Colon,
    Semicolon,
    Plus,
    PlusPlus,
    Dash,
    Exclamation,
    Asterisk,
    AsteriskAsterisk,
    ForwardSlash,
    ForwardSlashForwardSlash,
    Equals,
    EqualsEquals,
    EqualsTilde,
    ExclamationTilde,
    ExclamationEquals,
    LParen,
    LSquare,
    LCurly,
    LessThan,
    LessThanEqual,
    RParen,
    RSquare,
    RCurly,
    GreaterThan,
    GreaterThanEqual,
    Ampersand,
    AmpersandAmpersand,
    Bareword,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub contents: &'a [u8],
    pub span_start: usize,
    pub span_end: usize,
}

fn is_symbol(b: u8) -> bool {
    [
        b'+', b'-', b'*', b'/', b'.', b',', b'(', b'[', b'{', b'<', b')', b']', b'}', b'>', b':',
        b';', b'=', b'$', b'|', b'!', b'~', b'&',
    ]
    .contains(&b)
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8], span_offset: usize) -> Self {
        Self {
            source,
            span_offset,
            newline_is_space: false,
        }
    }

    pub fn lex_quoted_string(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        let mut token_offset = 1;
        let mut is_escaped = false;
        while token_offset < self.source.len() {
            if is_escaped {
                is_escaped = false;
            } else if self.source[token_offset] == b'\\' {
                is_escaped = true;
            } else if self.source[token_offset] == b'"' {
                token_offset += 1;
                break;
            }
            token_offset += 1;
        }

        self.span_offset += token_offset;

        let contents = &self.source[..token_offset];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::String,
            contents,
            span_start,
            span_end: self.span_offset,
        })
    }

    pub fn lex_single_quoted_string(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        let mut token_offset = 1;
        while token_offset < self.source.len() {
            if self.source[token_offset] == b'\'' {
                token_offset += 1;
                break;
            }
            token_offset += 1;
        }

        self.span_offset += token_offset;

        let contents = &self.source[..token_offset];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::SimpleString,
            contents,
            span_start,
            span_end: self.span_offset,
        })
    }

    pub fn lex_quoted_bareword(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        let mut token_offset = 1;
        while token_offset < self.source.len() {
            if self.source[token_offset] == b'`' {
                token_offset += 1;
                break;
            }
            token_offset += 1;
        }

        self.span_offset += token_offset - 1;

        let contents = &self.source[1..(token_offset - 1)];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::Bareword,
            contents,
            span_start: span_start + 1,
            span_end: self.span_offset,
        })
    }

    pub fn lex_number(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        let mut token_offset = 0;
        while token_offset < self.source.len() {
            if !self.source[token_offset].is_ascii_digit() {
                break;
            }
            token_offset += 1;
        }

        self.span_offset += token_offset;

        let contents = &self.source[..token_offset];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::Number,
            contents,
            span_start,
            span_end: self.span_offset,
        })
    }

    pub fn lex_space(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        let mut token_offset = 0;
        let whitespace: &[u8] = if self.newline_is_space {
            &[b' ', b'\t', b'\r', b'\n']
        } else {
            &[b' ', b'\t', b'\r']
        };
        while token_offset < self.source.len() {
            if !whitespace.contains(&self.source[token_offset]) {
                break;
            }
            token_offset += 1;
        }
        self.span_offset += token_offset;

        let contents = &self.source[..token_offset];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::Space,
            contents,
            span_start,
            span_end: self.span_offset,
        })
    }

    pub fn lex_newline(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        self.span_offset += 1;

        let contents = &self.source[..1];
        self.source = &self.source[1..];

        Some(Token {
            token_type: TokenType::Newline,
            contents,
            span_start,
            span_end: span_start + 1,
        })
    }

    pub fn lex_variable(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        self.span_offset += 1;

        let mut token_offset = 1;
        while token_offset < self.source.len() {
            if self.source[token_offset].is_ascii_whitespace()
                || is_symbol(self.source[token_offset])
            {
                break;
            }
            token_offset += 1;
        }
        self.span_offset += token_offset;
        let contents = &self.source[..token_offset];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::Variable,
            contents,
            span_start,
            span_end: self.span_offset,
        })
    }

    pub fn lex_comment(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        self.span_offset += 1;

        let mut token_offset = 1;
        while token_offset < self.source.len() {
            if self.source[token_offset] == b'\n' {
                break;
            }
            token_offset += 1;
        }
        self.span_offset += token_offset;
        let contents = &self.source[..token_offset];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::Comment,
            contents,
            span_start,
            span_end: self.span_offset,
        })
    }

    pub fn lex_symbol(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;

        let result = match self.source[0] {
            b'(' => Token {
                token_type: TokenType::LParen,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'[' => Token {
                token_type: TokenType::LSquare,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'{' => Token {
                token_type: TokenType::LCurly,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'<' => {
                if self.source.len() > 1 && self.source[1] == b'=' {
                    Token {
                        token_type: TokenType::LessThanEqual,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::LessThan,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b')' => Token {
                token_type: TokenType::RParen,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b']' => Token {
                token_type: TokenType::RSquare,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'}' => Token {
                token_type: TokenType::RCurly,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'>' => {
                if self.source.len() > 1 && self.source[1] == b'=' {
                    Token {
                        token_type: TokenType::GreaterThanEqual,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::GreaterThan,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b'+' => {
                if self.source.len() > 1 && self.source[1] == b'+' {
                    Token {
                        token_type: TokenType::PlusPlus,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::Plus,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b'-' => Token {
                token_type: TokenType::Dash,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'*' => {
                if self.source.len() > 1 && self.source[1] == b'*' {
                    Token {
                        token_type: TokenType::AsteriskAsterisk,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::Asterisk,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b'/' => {
                if self.source.len() > 1 && self.source[1] == b'/' {
                    Token {
                        token_type: TokenType::ForwardSlashForwardSlash,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::ForwardSlash,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b'=' => {
                if self.source.len() > 1 && self.source[1] == b'=' {
                    Token {
                        token_type: TokenType::EqualsEquals,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else if self.source.len() > 1 && self.source[1] == b'~' {
                    Token {
                        token_type: TokenType::EqualsTilde,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::Equals,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b':' => Token {
                token_type: TokenType::Colon,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b';' => Token {
                token_type: TokenType::Semicolon,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'.' => {
                if self.source.len() > 1 && self.source[1] == b'.' {
                    Token {
                        token_type: TokenType::DotDot,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::Dot,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b'!' => {
                if self.source.len() > 1 && self.source[1] == b'=' {
                    Token {
                        token_type: TokenType::ExclamationEquals,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else if self.source.len() > 1 && self.source[1] == b'~' {
                    Token {
                        token_type: TokenType::ExclamationTilde,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::Exclamation,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b'$' => Token {
                token_type: TokenType::Dollar,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            b'|' => {
                if self.source.len() > 1 && self.source[1] == b'|' {
                    Token {
                        token_type: TokenType::PipePipe,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::Pipe,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b'&' => {
                if self.source.len() > 1 && self.source[1] == b'&' {
                    Token {
                        token_type: TokenType::AmpersandAmpersand,
                        contents: &self.source[..2],
                        span_start,
                        span_end: span_start + 2,
                    }
                } else {
                    Token {
                        token_type: TokenType::Ampersand,
                        contents: &self.source[..1],
                        span_start,
                        span_end: span_start + 1,
                    }
                }
            }
            b',' => Token {
                token_type: TokenType::Comma,
                contents: &self.source[..1],
                span_start,
                span_end: span_start + 1,
            },
            x => {
                panic!(
                    "Internal compiler error: symbol character mismatched in lexer: {}",
                    x as char
                )
            }
        };

        self.span_offset = result.span_end;
        self.source = &self.source[(result.span_end - span_start)..];

        Some(result)
    }

    pub fn lex_bareword(&mut self) -> Option<Token<'a>> {
        let span_start = self.span_offset;
        let mut token_offset = 0;
        while token_offset < self.source.len() {
            if self.source[token_offset].is_ascii_whitespace()
                || self.source[token_offset] == b'{'
                || self.source[token_offset] == b'}'
                || self.source[token_offset] == b')'
                || self.source[token_offset] == b'('
                || self.source[token_offset] == b'['
                || self.source[token_offset] == b']'
                || self.source[token_offset] == b';'
                || self.source[token_offset] == b':'
                || self.source[token_offset] == b','
            {
                break;
            }
            token_offset += 1;
        }
        self.span_offset += token_offset;
        let contents = &self.source[..token_offset];
        self.source = &self.source[token_offset..];

        Some(Token {
            token_type: TokenType::Bareword,
            contents,
            span_start,
            span_end: self.span_offset,
        })
    }
}

impl<'a> Lexer<'a> {
    pub fn peek(&mut self) -> Option<Token<'a>> {
        let prev_offset = self.span_offset;
        let prev_source = self.source;
        let output = self.next();
        self.span_offset = prev_offset;
        self.source = prev_source;

        output
    }

    pub fn next(&mut self) -> Option<Token<'a>> {
        if self.source.is_empty() {
            None
        } else if self.source[0].is_ascii_digit() {
            self.lex_number()
        } else if self.source[0] == b'"' {
            self.lex_quoted_string()
        } else if self.source[0] == b'\'' {
            self.lex_single_quoted_string()
        } else if self.source[0] == b'`' {
            self.lex_quoted_bareword()
        } else if self.source[0] == b' '
            || self.source[0] == b'\t'
            || self.source[0] == b'\r'
            || (self.newline_is_space && self.source[0] == b'\n')
        {
            self.lex_space()
        } else if self.source[0] == b'$' {
            self.lex_variable()
        } else if is_symbol(self.source[0]) {
            self.lex_symbol()
        } else if self.source[0] == b'\n' {
            self.lex_newline()
        } else if self.source[0] == b'#' {
            self.lex_comment()
        } else {
            self.lex_bareword()
        }
    }
}
