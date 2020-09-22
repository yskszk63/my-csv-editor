use std::fmt;
use std::str::CharIndices;
use std::iter::Peekable;

type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Token<S> {
    COMMA,
    CR,
    DQUOTE,
    LF,
    CRLF,
    TEXTDATA(S),
    LF_BEFORE_EOF,
    CRLF_BEFORE_EOF,
}

impl<'input> Token<&'input str> {
    pub(crate) fn to_owned(self) -> Token<String> {
        match self {
            Self::COMMA => Token::COMMA,
            Self::CR => Token::CR,
            Self::DQUOTE => Token::DQUOTE,
            Self::LF => Token::LF,
            Self::CRLF => Token::CRLF,
            Self::TEXTDATA(s) => Token::TEXTDATA(s.to_string()),
            Self::LF_BEFORE_EOF => Token::LF_BEFORE_EOF,
            Self::CRLF_BEFORE_EOF => Token::CRLF_BEFORE_EOF,
        }
    }
}

impl<S> fmt::Display for Token<S> where S: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::COMMA => write!(f, "COMMA"),
            Self::CR => write!(f, "CR"),
            Self::DQUOTE => write!(f, "DQUOTE"),
            Self::LF => write!(f, "LF"),
            Self::CRLF => write!(f, "CRLF"),
            Self::TEXTDATA(s) => write!(f, "TEXTDATA({:?})", s),
            Self::LF_BEFORE_EOF => write!(f, "LF_BEFORE_EOF"),
            Self::CRLF_BEFORE_EOF => write!(f, "CRLF_BEFORE_EOF"),
        }
    }
}

pub(crate) struct Lexer<'input> {
    cursor: Peekable<CharIndices<'input>>,
    input: &'input str,
}

impl<'input> Lexer<'input> {
    pub(crate) fn new(input: &'input str) -> Self {
        Self {
            cursor: input.char_indices().peekable(),
            input,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token<&'input str>, usize, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor.next() {
            Some((i, '\x2C')) => Some(Ok((i, Token::COMMA, i + 1))),
            Some((i, '\x0D')) => {
                if let Some((_, '\x0A')) = self.cursor.peek() {
                    self.cursor.next();
                    if self.cursor.peek().is_none() {
                        Some(Ok((i, Token::CRLF_BEFORE_EOF, i + 2)))
                    } else {
                        Some(Ok((i, Token::CRLF, i + 2)))
                    }
                } else {
                    Some(Ok((i, Token::CR, i + 1)))
                }
            }
            Some((i, '\x22')) => Some(Ok((i, Token::DQUOTE, i + 1))),
            Some((i, '\x0A')) => {
                if self.cursor.peek().is_none() {
                    Some(Ok((i, Token::LF_BEFORE_EOF, i + 1)))
                } else {
                    Some(Ok((i, Token::LF, i + 1)))
                }
            }
            Some((i, _)) => {
                while let Some((j, x)) = self.cursor.peek() {
                    match x {
                        '\x2C' | '\x0D' | '\x22' | '\x0A' => {
                            return Some(Ok((i, Token::TEXTDATA(&self.input[i..*j]), *j)))
                        }
                        _ => {
                            self.cursor.next();
                        }
                    }
                }
                Some(Ok((i, Token::TEXTDATA(&self.input[i..]), self.input.len())))
            }
            None => None,
        }
    }
}
