use std::{
    error::Error,
    fmt::Display,
    iter::{Enumerate, Peekable},
    num::{ParseFloatError, ParseIntError},
    str::{Chars, Lines},
};

use super::{
    position::{Indexed, Located, Position},
    tokens::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub ln: usize,
    pub indent: usize,
    pub tokens: Vec<Indexed<Token>>,
}

#[derive(Debug)]
pub struct Lexer<'source> {
    pub lines: Enumerate<Lines<'source>>,
}
#[derive(Debug)]
pub struct LineLexer<'source> {
    pub ln: usize,
    pub chars: Peekable<Enumerate<Chars<'source>>>,
}
impl<'source> From<&'source str> for Lexer<'source> {
    fn from(value: &'source str) -> Self {
        Self {
            lines: value.lines().enumerate(),
        }
    }
}
impl<'source> From<(usize, &'source str)> for LineLexer<'source> {
    fn from(value: (usize, &'source str)) -> Self {
        Self {
            ln: value.0,
            chars: value.1.chars().enumerate().peekable(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    BadCharacter(char),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    ExpectedCharacter,
    ExpectedEscape,
    UnclosedChar,
    UnclosedString,
}
impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadCharacter(c) => write!(f, "bad character {c:?}"),
            Self::ParseIntError(err) => write!(f, "error while parsing int: {err}"),
            Self::ParseFloatError(err) => write!(f, "error while parsing float: {err}"),
            Self::ExpectedCharacter => write!(f, "expected character"),
            Self::ExpectedEscape => write!(f, "expected escape character"),
            Self::UnclosedChar => write!(f, "unclosed character"),
            Self::UnclosedString => write!(f, "unclosed string"),
        }
    }
}
impl Error for LexError {}
impl<'source> Lexer<'source> {
    pub fn lex(self) -> Result<Vec<Line>, Located<LexError>> {
        let (lines, errors): (Vec<_>, Vec<_>) = self.partition(Result::is_ok);
        let mut errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).rev().collect();
        if let Some(error) = errors.pop() {
            return Err(error);
        }
        let lines = lines.into_iter().map(Result::unwrap).collect();
        Ok(lines)
    }
}
impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Line, Located<LexError>>;
    fn next(&mut self) -> Option<Self::Item> {
        let (ln, line) = self.lines.next()?;
        let mut line_lexer = LineLexer::from((ln, line));
        let indent = {
            let mut indent = 0;
            while let Some((_, c)) = line_lexer.chars.peek() {
                if !c.is_ascii_whitespace() {
                    break;
                }
                line_lexer.chars.next();
                indent += 1;
            }
            indent
        };
        let (tokens, errors): (Vec<_>, Vec<_>) = line_lexer.partition(Result::is_ok);
        let mut errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).rev().collect();
        if let Some(error) = errors.pop() {
            return Some(Err(error));
        }
        Some(Ok(Line {
            indent,
            ln,
            tokens: tokens.into_iter().map(Result::unwrap).collect(),
        }))
    }
}
impl<'source> Iterator for LineLexer<'source> {
    type Item = Result<Indexed<Token>, Located<LexError>>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((_, c)) = self.chars.peek() {
            if !c.is_ascii_whitespace() {
                break;
            }
            self.chars.next();
        }
        let (col, c) = self.chars.next()?;
        let mut index = col..col;
        match c {
            '=' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::EqualEqual, index)))
                } else if let Some((col, '>')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::EqualArrow, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Equal, index)))
                }
            }
            ',' => Some(Ok(Indexed::new(Token::Comma, index))),
            '.' => Some(Ok(Indexed::new(Token::Dot, index))),
            ':' => Some(Ok(Indexed::new(Token::Colon, index))),
            '!' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::ExclamationEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Exclamation, index)))
                }
            }
            '(' => Some(Ok(Indexed::new(Token::ParanLeft, index))),
            ')' => Some(Ok(Indexed::new(Token::ParanRight, index))),
            '[' => Some(Ok(Indexed::new(Token::BracketLeft, index))),
            ']' => Some(Ok(Indexed::new(Token::BracketRight, index))),
            '{' => Some(Ok(Indexed::new(Token::BraceLeft, index))),
            '}' => Some(Ok(Indexed::new(Token::BraceRight, index))),
            '+' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::PlusEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Plus, index)))
                }
            }
            '-' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::MinusEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Minus, index)))
                }
            }
            '*' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::StarEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Star, index)))
                }
            }
            '/' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::SlashEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Slash, index)))
                }
            }
            '%' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::PercentEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Percent, index)))
                }
            }
            '^' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::ExponentEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Exponent, index)))
                }
            }
            '<' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::LessEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Less, index)))
                }
            }
            '>' => {
                if let Some((col, '=')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    Some(Ok(Indexed::new(Token::GreaterEqual, index)))
                } else {
                    Some(Ok(Indexed::new(Token::Greater, index)))
                }
            }
            '&' => Some(Ok(Indexed::new(Token::Ampersand, index))),
            '|' => Some(Ok(Indexed::new(Token::Pipe, index))),
            '\'' => {
                let c = match self
                    .chars
                    .next()
                    .ok_or(LexError::ExpectedCharacter)
                    .map_err(|err| {
                        Located::new(err, Position::new(self.ln..self.ln, index.clone()))
                    }) {
                    Ok((col, c)) => match c {
                        '\\' => {
                            let c = match self.chars.peek().cloned() {
                                Some((_, 'n')) => '\n',
                                Some((_, 't')) => '\t',
                                Some((_, 'r')) => '\r',
                                Some((_, '0')) => '\0',
                                Some((_, c)) => c,
                                None => {
                                    return Some(Err(Located::new(
                                        LexError::ExpectedEscape,
                                        Position::new(self.ln..self.ln, index.end..index.end),
                                    )))
                                }
                            };
                            self.chars.next();
                            index.end = col;
                            c
                        }
                        c => c,
                    },
                    Err(err) => return Some(Err(err)),
                };
                if let Some((col, '\'')) = dbg!(self.chars.next()) {
                    index.end = col;
                    Some(Ok(Indexed::new(Token::Char(c), index)))
                } else {
                    Some(Err(Located::new(
                        LexError::UnclosedChar,
                        Position::new(self.ln..self.ln, index),
                    )))
                }
            }
            '"' => {
                let mut string = String::new();
                while let Some((col, c)) = self.chars.peek().cloned() {
                    if c == '"' {
                        break;
                    }
                    string.push(match c {
                        '\\' => {
                            self.chars.next()?;
                            match self.chars.peek().cloned().map(|p| p.1) {
                                Some('n') => '\n',
                                Some('t') => '\t',
                                Some('r') => '\r',
                                Some('0') => '\0',
                                Some(c) => c,
                                None => {
                                    return Some(Err(Located::new(
                                        LexError::ExpectedEscape,
                                        Position::new(self.ln..self.ln, index),
                                    )))
                                }
                            }
                        }
                        c => c,
                    });
                    index.end = col;
                    self.chars.next();
                }
                if let Some((col, '"')) = self.chars.next() {
                    index.end = col;
                    Some(Ok(Indexed::new(Token::String(string), index)))
                } else {
                    Some(Err(Located::new(
                        LexError::UnclosedString,
                        Position::new(self.ln..self.ln, index),
                    )))
                }
            }
            c if c.is_ascii_digit() => {
                let mut number = String::from(c);
                while let Some((col, c)) = self.chars.peek().cloned() {
                    if !c.is_ascii_alphanumeric() && c != '_' {
                        break;
                    }
                    self.chars.next();
                    index.end = col;
                    if c != '_' {
                        number.push(c);
                    }
                }
                if let Some((col, '.')) = self.chars.peek().cloned() {
                    self.chars.next();
                    index.end = col;
                    number.push('.');
                    while let Some((col, c)) = self.chars.peek().cloned() {
                        if !c.is_ascii_alphanumeric() && c != '_' {
                            break;
                        }
                        self.chars.next();
                        index.end = col;
                        if c != '_' {
                            number.push(c);
                        }
                    }
                    match number
                        .parse()
                        .map_err(LexError::ParseFloatError)
                        .map_err(|err| {
                            Located::new(err, Position::new(self.ln..self.ln, index.clone()))
                        }) {
                        Ok(number) => Some(Ok(Indexed::new(Token::Float(number), index))),
                        Err(err) => Some(Err(err)),
                    }
                } else {
                    match number
                        .parse()
                        .map_err(LexError::ParseIntError)
                        .map_err(|err| {
                            Located::new(err, Position::new(self.ln..self.ln, index.clone()))
                        }) {
                        Ok(number) => Some(Ok(Indexed::new(Token::Int(number), index))),
                        Err(err) => Some(Err(err)),
                    }
                }
            }
            c if c.is_ascii_alphanumeric() || c == '_' => {
                let mut ident = String::from(c);
                while let Some((col, c)) = self.chars.peek().cloned() {
                    if !c.is_ascii_alphanumeric() && c != '_' {
                        break;
                    }
                    self.chars.next();
                    index.end = col;
                    ident.push(c);
                }
                Some(Ok(Indexed::new(Token::ident(ident), index)))
            }
            c => Some(Err(Located::new(
                LexError::BadCharacter(c),
                Position::new(self.ln..self.ln, index),
            ))),
        }
    }
}
