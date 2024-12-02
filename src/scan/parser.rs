use super::{
    ast::*,
    lexer::Line,
    position::{Indexed, Located, Position},
    tokens::Token,
};
use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub trait Parsable: Debug + Clone + PartialEq {
    type Error: Error;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>>;
}
#[derive(Debug, Clone)]
pub struct Parser {
    pub lines: Vec<Line>,
}
impl Parser {
    pub fn new(lines: Vec<Line>) -> Self {
        Self { lines }
    }
    #[inline(always)]
    pub fn get(&mut self) -> Option<Indexed<Token>> {
        let line = self.lines.first_mut()?;
        if !line.tokens.is_empty() {
            Some(line.tokens.remove(0))
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn advance_line(&mut self) -> Option<Line> {
        if !self.eof() {
            Some(self.lines.remove(0))
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn eol(&self) -> bool {
        self.lines
            .first()
            .map(|line| line.tokens.is_empty())
            .unwrap_or_default()
    }
    #[inline(always)]
    pub fn eof(&self) -> bool {
        self.lines.is_empty()
    }
    #[inline(always)]
    pub fn expect_eol(&mut self) -> Result<(), Located<ParseError>> {
        if let Some(Indexed { value: _, index }) = self.get() {
            Err(Located::new(
                ParseError::ExpectedNewLine,
                Position::new(self.ln()..self.ln(), index),
            ))
        } else {
            Ok(())
        }
    }
    #[inline(always)]
    pub fn expect_any(&mut self) -> Result<Indexed<Token>, Located<ParseError>> {
        if let Some(token) = self.get() {
            Ok(token)
        } else {
            Err(Located::new(
                ParseError::UnexpectedEOL,
                Position::new(self.ln()..self.ln(), 0..0),
            ))
        }
    }
    #[inline(always)]
    pub fn skip(&mut self, token: Token) {
        let Some(Indexed {
            value: current,
            index: _,
        }) = self.peek()
        else {
            return;
        };
        if current == &token {
            self.get();
        }
    }
    #[inline(always)]
    pub fn expect(&mut self, token: Token) -> Result<Indexed<Token>, Located<ParseError>> {
        let Indexed {
            value: current,
            index,
        } = self
            .get()
            .ok_or(Located::new(ParseError::UnexpectedEOL, Position::default()))?;
        if current != token {
            return Err(Located::new(
                ParseError::Expected {
                    expected: token,
                    got: current,
                },
                Position::new(self.ln()..self.ln(), index),
            ));
        }
        Ok(Indexed {
            value: token,
            index,
        })
    }
    #[inline(always)]
    pub fn peek(&self) -> Option<&Indexed<Token>> {
        self.lines.first()?.tokens.first()
    }
    #[inline(always)]
    pub fn ln(&self) -> usize {
        self.lines.first().map(|line| line.ln).unwrap_or_default()
    }
    #[inline(always)]
    pub fn indent(&self) -> usize {
        self.lines
            .first()
            .map(|line| line.indent)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedEOL,
    ExpectedNewLine,
    ExpectedIndentedBlock,
    UnexpectedToken(Token),
    Expected { expected: Token, got: Token },
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEOF => write!(f, "unexpected end of file"),
            ParseError::UnexpectedEOL => write!(f, "unexpected end of line"),
            ParseError::ExpectedNewLine => write!(f, "expected new line"),
            ParseError::ExpectedIndentedBlock => write!(f, "expected indented block"),
            ParseError::UnexpectedToken(token) => write!(f, "unexpected {}", token.name()),
            ParseError::Expected { expected, got } => {
                write!(f, "expected {}, got {}", expected.name(), got.name())
            }
        }
    }
}
impl Error for ParseError {}
impl Parsable for Chunk {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let mut stats = vec![];
        let mut pos = Position::default();
        while !parser.lines.is_empty() {
            let stat = Statement::parse(parser)?;
            pos.extend(&stat.pos);
            stats.push(stat);
        }
        Ok(Located::new(Self { stats }, pos))
    }
}
impl Parsable for Block {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let parent_indent = parser.indent();
        parser.expect_eol()?;
        parser.advance_line();
        let base_indent = parser.indent();
        if parent_indent >= base_indent {
            return Err(Located::new(
                ParseError::ExpectedIndentedBlock,
                Position::new(parser.ln()..parser.ln(), 0..0),
            ));
        }
        let mut stats = vec![];
        let mut pos = Position::default();
        while parser.indent() >= base_indent {
            let stat = Statement::parse(parser)?;
            pos.extend(&stat.pos);
            stats.push(stat);
        }
        Ok(Located::new(Self { stats }, pos))
    }
}
impl Parsable for Statement {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        if let Some(Indexed {
            value: Token::Ident(_),
            index: _,
        }) = parser.peek()
        {
            let path = Path::parse(parser)?;
            let Indexed {
                value: token,
                index,
            } = parser.expect_any()?;
            return match token {
                token if let Some(op) = AssignOperator::token(&token) => {
                    let expr = Expression::parse(parser)?;
                    let mut pos = path.pos.clone();
                    pos.extend(&expr.pos);
                    parser.expect_eol()?;
                    parser.advance_line();
                    Ok(Located::new(
                        Self::Assign {
                            op,
                            path,
                            expr,
                        },
                        pos,
                    ))
                }
                Token::ParanLeft => {
                    let mut pos = path.pos.clone();
                    let mut args = vec![];
                    while let Some(Indexed { value: token, .. }) = parser.peek() {
                        if token == &Token::ParanRight {
                            break;
                        }
                        let expr = Expression::parse(parser)?;
                        args.push(expr);
                        if let Some(Indexed {
                            value: Token::ParanRight,
                            index: _,
                        }) = parser.peek()
                        {
                            break;
                        }
                        parser.expect(Token::Comma)?;
                    }
                    let Indexed {
                        value: _,
                        index: end,
                    } = parser.expect(Token::ParanRight)?;
                    pos.ln.end = parser.ln();
                    pos.col.end = end.end;
                    parser.expect_eol()?;
                    parser.advance_line();
                    Ok(Located::new(Self::Call { head: path, args }, pos))
                }
                Token::Colon => {
                    let mut pos = path.pos.clone();
                    let field: Located<String> = Parameter::parse_ident(parser)?;
                    parser.expect(Token::ParanLeft)?;
                    let mut args = vec![];
                    while let Some(Indexed { value: token, .. }) = parser.peek() {
                        if token == &Token::ParanRight {
                            break;
                        }
                        let expr = Expression::parse(parser)?;
                        args.push(expr);
                        if let Some(Indexed {
                            value: Token::ParanRight,
                            index: _,
                        }) = parser.peek()
                        {
                            break;
                        }
                        parser.expect(Token::Comma)?;
                    }
                    let Indexed {
                        value: _,
                        index: end,
                    } = parser.expect(Token::ParanRight)?;
                    pos.ln.end = parser.ln();
                    pos.col.end = end.end;
                    parser.expect_eol()?;
                    parser.advance_line();
                    Ok(Located::new(
                        Self::SelfCall {
                            head: path,
                            field,
                            args,
                        },
                        pos,
                    ))
                }
                token => Err(Located::new(
                    ParseError::UnexpectedToken(token),
                    Position::new(parser.ln()..parser.ln(), index),
                )),
            };
        }
        let Indexed {
            value: token,
            mut index,
        } = parser.expect_any()?;
        match token {
            Token::Let => {
                let param = Parameter::parse(parser)?;
                parser.expect(Token::Equal)?;
                let expr = Expression::parse(parser)?;
                index.end = expr.pos.col.end;
                parser.expect_eol()?;
                parser.advance_line();
                Ok(Located::new(
                    Self::LetBinding { param, expr },
                    Position::new(parser.ln()..parser.ln(), index),
                ))
            }
            Token::Return => {
                if parser.eol() {
                    parser.expect_eol()?;
                    parser.advance_line();
                    return Ok(Located::new(
                        Self::Return(None),
                        Position::new(parser.ln()..parser.ln(), index),
                    ));
                }
                let expr = Expression::parse(parser)?;
                index.end = expr.pos.col.end;
                parser.expect_eol()?;
                parser.advance_line();
                Ok(Located::new(
                    Self::Return(Some(expr)),
                    Position::new(parser.ln()..parser.ln(), index),
                ))
            }
            Token::Fn => {
                let mut pos = Position::new(parser.ln()..parser.ln(), index);
                let name = Parameter::parse_ident(parser)?;
                parser.expect(Token::ParanLeft)?;
                let mut params = vec![];
                let mut varargs = None;
                while let Some(Indexed { value: token, .. }) = parser.peek() {
                    if token == &Token::ParanRight {
                        break;
                    }
                    if token == &Token::DotDotDot {
                        parser.expect_any()?;
                        varargs = Some(Parameter::parse_ident(parser)?);
                        break;
                    }
                    let param = Parameter::parse(parser)?;
                    params.push(param);
                    if let Some(Indexed {
                        value: Token::ParanRight,
                        index: _,
                    }) = parser.peek()
                    {
                        break;
                    }
                    parser.expect(Token::Comma)?;
                }
                parser.expect(Token::ParanRight)?;
                let body = Block::parse(parser)?;
                pos.extend(&body.pos);
                parser.expect_eol()?;
                parser.advance_line();
                Ok(Located::new(
                    Self::Fn {
                        name,
                        params,
                        varargs,
                        body,
                    },
                    pos,
                ))
            }
            token => Err(Located::new(
                ParseError::UnexpectedToken(token),
                Position::new(parser.ln()..parser.ln(), index),
            )),
        }
    }
}
impl AssignOperator {
    pub fn token(token: &Token) -> Option<Self> {
        match token {
            Token::Equal => Some(Self::None),
            Token::PlusEqual => Some(Self::Plus),
            Token::MinusEqual => Some(Self::Minus),
            Token::StarEqual => Some(Self::Star),
            Token::SlashEqual => Some(Self::Slash),
            Token::PercentEqual => Some(Self::Percent),
            Token::ExponentEqual => Some(Self::Exponent),
            _ => None,
        }
    }
}
impl Parameter {
    fn parse_ident(parser: &mut Parser) -> Result<Located<String>, Located<ParseError>> {
        let Indexed {
            value: current,
            index,
        } = parser
            .get()
            .ok_or(Located::new(ParseError::UnexpectedEOL, Position::default()))?;
        if let Token::Ident(ident) = current {
            return Ok(Located::new(
                ident,
                Position::new(parser.ln()..parser.ln(), index),
            ));
        }
        Err(Located::new(
            ParseError::Expected {
                expected: Token::Ident(Default::default()),
                got: current,
            },
            Position::new(parser.ln()..parser.ln(), index),
        ))
    }
}
impl Parsable for Parameter {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        if let Some(Indexed {
            value: Token::ParanLeft,
            index: _,
        }) = parser.peek()
        {
            let Indexed { value: _, index } = parser.expect(Token::ParanLeft)?;
            let mut pos = Position::new(parser.ln()..parser.ln(), index);
            let mut params = vec![];
            let param = Parameter::parse_ident(parser)?;
            params.push(param);
            while let Some(Indexed { value: token, .. }) = parser.peek() {
                if token == &Token::ParanRight {
                    break;
                }
                parser.expect(Token::Comma)?;
                if let Some(Indexed {
                    value: Token::ParanRight,
                    index: _,
                }) = parser.peek()
                {
                    break;
                }
                let param = Parameter::parse_ident(parser)?;
                params.push(param);
            }
            pos.col.end = parser.expect(Token::ParanRight)?.index.end;
            return Ok(Located::new(Self::Tuple(params), pos));
        }
        if let Some(Indexed {
            value: Token::BracketLeft,
            index: _,
        }) = parser.peek()
        {
            let Indexed { value: _, index } = parser.expect(Token::BracketLeft)?;
            let mut pos = Position::new(parser.ln()..parser.ln(), index);
            let mut params = vec![];
            let param = Parameter::parse_ident(parser)?;
            params.push(param);
            while let Some(Indexed { value: token, .. }) = parser.peek() {
                if token == &Token::BracketRight {
                    break;
                }
                parser.expect(Token::Comma)?;
                if let Some(Indexed {
                    value: Token::BracketRight,
                    index: _,
                }) = parser.peek()
                {
                    break;
                }
                let param = Parameter::parse_ident(parser)?;
                params.push(param);
            }
            pos.col.end = parser.expect(Token::BracketRight)?.index.end;
            return Ok(Located::new(Self::Vector(params), pos));
        }
        if let Some(Indexed {
            value: Token::BraceLeft,
            index: _,
        }) = parser.peek()
        {
            let Indexed { value: _, index } = parser.expect(Token::BraceLeft)?;
            let mut pos = Position::new(parser.ln()..parser.ln(), index);
            let mut params = vec![];
            let field = Parameter::parse_ident(parser)?;
            params.push(field);
            while let Some(Indexed { value: token, .. }) = parser.peek() {
                if token == &Token::BraceRight {
                    break;
                }
                parser.expect(Token::Comma)?;
                if let Some(Indexed {
                    value: Token::BraceRight,
                    index: _,
                }) = parser.peek()
                {
                    break;
                }
                let field = Parameter::parse_ident(parser)?;
                params.push(field);
            }
            pos.col.end = parser.expect(Token::BraceRight)?.index.end;
            return Ok(Located::new(Self::Map(params), pos));
        }
        Ok(Self::parse_ident(parser)?.map(Self::Ident))
    }
}
impl Parsable for Expression {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        Self::binary(parser, 0)
    }
}
impl BinaryOperator {
    const LAYERS: &'static [&'static [Self]] = &[
        &[Self::And, Self::Or],
        &[
            Self::EqualEqual,
            Self::ExclamationEqual,
            Self::Greater,
            Self::Less,
            Self::GreaterEqual,
            Self::LessEqual,
            Self::Is,
            Self::In,
        ],
        &[Self::Plus, Self::Minus],
        &[Self::Star, Self::Slash, Self::Percent],
        &[Self::Exponent],
        &[Self::As],
    ];
    pub fn layer(layer: usize) -> Option<&'static [Self]> {
        Self::LAYERS.get(layer).copied()
    }
    pub fn token(token: &Token) -> Option<Self> {
        match token {
            Token::Plus => Some(Self::Plus),
            Token::Minus => Some(Self::Minus),
            Token::Star => Some(Self::Star),
            Token::Slash => Some(Self::Slash),
            Token::Percent => Some(Self::Percent),
            Token::Exponent => Some(Self::Exponent),
            Token::EqualEqual => Some(Self::EqualEqual),
            Token::ExclamationEqual => Some(Self::ExclamationEqual),
            Token::Less => Some(Self::Less),
            Token::Greater => Some(Self::Greater),
            Token::LessEqual => Some(Self::LessEqual),
            Token::GreaterEqual => Some(Self::GreaterEqual),
            Token::And => Some(Self::And),
            Token::Or => Some(Self::Or),
            Token::Is => Some(Self::Is),
            Token::In => Some(Self::In),
            Token::As => Some(Self::As),
            _ => None,
        }
    }
}
impl UnaryOperator {
    const LAYERS: &'static [&'static [Self]] = &[&[Self::Not], &[Self::Minus]];
    pub fn layer(layer: usize) -> Option<&'static [Self]> {
        Self::LAYERS.get(layer).copied()
    }
    pub fn token(token: &Token) -> Option<Self> {
        match token {
            Token::Minus => Some(Self::Minus),
            Token::Not => Some(Self::Not),
            _ => None,
        }
    }
}
impl Expression {
    fn binary(parser: &mut Parser, layer: usize) -> Result<Located<Self>, Located<ParseError>> {
        let Some(ops) = BinaryOperator::layer(layer) else {
            return Self::unary(parser, 0);
        };
        let mut left = Self::binary(parser, layer + 1)?;
        while let Some(Indexed {
            value: token,
            index: _,
        }) = parser.peek()
        {
            let Some(op) = BinaryOperator::token(token) else {
                break;
            };
            if !ops.contains(&op) {
                break;
            }
            parser.expect_any()?;
            let right = Self::binary(parser, layer + 1)?;
            let mut pos = left.pos.clone();
            pos.extend(&right.pos);
            left = Located::new(
                Self::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                pos,
            )
        }
        Ok(left)
    }
    fn unary(parser: &mut Parser, layer: usize) -> Result<Located<Self>, Located<ParseError>> {
        let Some(ops) = UnaryOperator::layer(layer) else {
            return Self::call(parser);
        };
        if let Some(Indexed {
            value: token,
            index: _,
        }) = parser.peek()
        {
            if let Some(op) = UnaryOperator::token(token) {
                if ops.contains(&op) {
                    let Indexed { value: _, index } = parser.expect_any()?;
                    let mut pos = Position::new(parser.ln()..parser.ln(), index);
                    let right = Self::unary(parser, layer)?;
                    pos.extend(&right.pos);
                    return Ok(Located::new(
                        Self::Unary {
                            op,
                            right: Box::new(right),
                        },
                        pos,
                    ));
                }
            }
        }
        Self::unary(parser, layer + 1)
    }
    fn call(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        let mut head = Atom::parse(parser)?.map(Self::Atom);
        while let Some(Indexed {
            value: token,
            index: _,
        }) = parser.peek()
        {
            head = match token {
                Token::ParanLeft => {
                    parser.expect_any()?;
                    let mut pos = head.pos.clone();
                    let mut args = vec![];
                    while let Some(Indexed { value: token, .. }) = parser.peek() {
                        if token == &Token::ParanRight {
                            break;
                        }
                        let expr = Expression::parse(parser)?;
                        args.push(expr);
                        if let Some(Indexed {
                            value: Token::ParanRight,
                            index: _,
                        }) = parser.peek()
                        {
                            break;
                        }
                        parser.expect(Token::Comma)?;
                    }
                    let Indexed {
                        value: _,
                        index: end,
                    } = parser.expect(Token::ParanRight)?;
                    pos.ln.end = parser.ln();
                    pos.col.end = end.end;
                    Located::new(
                        Self::Call {
                            head: Box::new(head),
                            args,
                        },
                        pos,
                    )
                }
                Token::Colon => {
                    parser.expect_any()?;
                    let mut pos = head.pos.clone();
                    let field: Located<String> = Parameter::parse_ident(parser)?;
                    parser.expect(Token::ParanLeft)?;
                    let mut args = vec![];
                    while let Some(Indexed { value: token, .. }) = parser.peek() {
                        if token == &Token::ParanRight {
                            break;
                        }
                        let expr = Expression::parse(parser)?;
                        args.push(expr);
                        if let Some(Indexed {
                            value: Token::ParanRight,
                            index: _,
                        }) = parser.peek()
                        {
                            break;
                        }
                        parser.expect(Token::Comma)?;
                    }
                    let Indexed {
                        value: _,
                        index: end,
                    } = parser.expect(Token::ParanRight)?;
                    pos.ln.end = parser.ln();
                    pos.col.end = end.end;
                    Located::new(
                        Self::SelfCall {
                            head: Box::new(head),
                            field,
                            args,
                        },
                        pos,
                    )
                }
                _ => break,
            };
        }
        Ok(head)
    }
}
impl Parsable for Atom {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        if let Some(Indexed {
            value: Token::Ident(_),
            index: _,
        }) = parser.peek()
        {
            return Ok(Path::parse(parser)?.map(Self::Path));
        }
        let Indexed {
            value: token,
            index,
        } = parser
            .get()
            .ok_or(Located::new(ParseError::UnexpectedEOL, Position::default()))?;
        let mut pos = Position::new(parser.ln()..parser.ln(), index);
        match token {
            Token::Null => Ok(Located::new(Self::Null, pos)),
            Token::Int(v) => Ok(Located::new(Self::Int(v), pos)),
            Token::Float(v) => Ok(Located::new(Self::Float(v), pos)),
            Token::Bool(v) => Ok(Located::new(Self::Bool(v), pos)),
            Token::Char(v) => Ok(Located::new(Self::Char(v), pos)),
            Token::String(v) => Ok(Located::new(Self::String(v), pos)),
            Token::ParanLeft => {
                let expr = Expression::parse(parser)?;
                if let Some(Indexed {
                    value: Token::Comma,
                    index: _,
                }) = parser.peek()
                {
                    parser.expect(Token::Comma)?;
                    if let Some(Indexed {
                        value: Token::ParanRight,
                        index: _,
                    }) = parser.peek()
                    {
                        parser.expect(Token::ParanRight)?;
                        return Ok(Located::new(Self::Tuple(vec![expr]), pos));
                    }
                    let mut exprs = vec![expr];
                    let expr = Expression::parse(parser)?;
                    exprs.push(expr);
                    while let Some(Indexed { value: token, .. }) = parser.peek() {
                        if token == &Token::ParanRight {
                            break;
                        }
                        parser.expect(Token::Comma)?;
                        if let Some(Indexed {
                            value: Token::ParanRight,
                            index: _,
                        }) = parser.peek()
                        {
                            break;
                        }
                        let expr = Expression::parse(parser)?;
                        exprs.push(expr);
                    }
                    parser.expect(Token::ParanRight)?;
                    return Ok(Located::new(Self::Tuple(exprs), pos));
                }
                pos.col.end = parser.expect(Token::ParanRight)?.index.end;
                Ok(Located::new(Self::Expression(Box::new(expr)), pos))
            }
            Token::BracketLeft => {
                if let Some(Indexed {
                    value: Token::BracketRight,
                    index: _,
                }) = parser.peek()
                {
                    parser.expect(Token::BracketRight)?;
                    Ok(Located::new(Self::Vector(vec![]), pos))
                } else {
                    let mut exprs = vec![];
                    let expr = Expression::parse(parser)?;
                    exprs.push(expr);
                    while let Some(Indexed { value: token, .. }) = parser.peek() {
                        if token == &Token::BracketRight {
                            break;
                        }
                        parser.expect(Token::Comma)?;
                        if let Some(Indexed {
                            value: Token::BracketRight,
                            index: _,
                        }) = parser.peek()
                        {
                            break;
                        }
                        let expr = Expression::parse(parser)?;
                        exprs.push(expr);
                    }
                    pos.col.end = parser.expect(Token::BracketRight)?.index.end;
                    Ok(Located::new(Self::Vector(exprs), pos))
                }
            }
            Token::BraceLeft => {
                if let Some(Indexed {
                    value: Token::BraceRight,
                    index: _,
                }) = parser.peek()
                {
                    parser.expect(Token::BraceRight)?;
                    Ok(Located::new(Self::Map(vec![]), pos))
                } else {
                    let mut exprs = vec![];
                    let field = Parameter::parse_ident(parser)?;
                    parser.expect(Token::Equal)?;
                    let expr = Expression::parse(parser)?;
                    exprs.push((field, expr));
                    while let Some(Indexed { value: token, .. }) = parser.peek() {
                        if token == &Token::BraceRight {
                            break;
                        }
                        parser.expect(Token::Comma)?;
                        if let Some(Indexed {
                            value: Token::BraceRight,
                            index: _,
                        }) = parser.peek()
                        {
                            break;
                        }
                        let field = Parameter::parse_ident(parser)?;
                        parser.expect(Token::Equal)?;
                        let expr = Expression::parse(parser)?;
                        exprs.push((field, expr));
                    }
                    pos.col.end = parser.expect(Token::BraceRight)?.index.end;
                    Ok(Located::new(Self::Map(exprs), pos))
                }
            }
            token => Err(Located::new(ParseError::UnexpectedToken(token), pos)),
        }
    }
}
impl Parsable for Path {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let mut head = Parameter::parse_ident(parser)?.map(Self::Ident);
        while let Some(Indexed {
            value: token,
            index: _,
        }) = parser.peek()
        {
            match token {
                Token::Dot => {
                    parser.get().unwrap();
                    let field = Parameter::parse_ident(parser)?;
                    let mut pos = head.pos.clone();
                    pos.extend(&field.pos);
                    head = Located::new(
                        Self::Field {
                            head: Box::new(head),
                            field,
                        },
                        pos,
                    );
                }
                Token::BracketLeft => {
                    parser.get().unwrap();
                    let index = Box::new(Expression::parse(parser)?);
                    let mut pos = head.pos.clone();
                    let Indexed {
                        value: _,
                        index: end,
                    } = parser.expect(Token::BracketRight)?;
                    pos.col.end = end.end;
                    head = Located::new(
                        Self::Index {
                            head: Box::new(head),
                            index,
                        },
                        pos,
                    );
                }
                _ => break,
            }
        }
        Ok(head)
    }
}
