use crate::{
    parse,
    scan::{
        ast::{Atom, BinaryOperator, Chunk, Expression, Parameter, Path, Statement, UnaryOperator},
        lexer::{Lexer, Line},
        parser::ParseError,
        position::{Indexed, Located},
        tokens::Token,
    },
};

#[test]
pub fn lexer_ident() {
    let text = "hello what the sigma";
    let lines = Lexer::from(text).lex().unwrap();
    dbg!(&lines);
    assert_eq!(
        lines,
        vec![Line {
            indent: 0,
            ln: 0,
            tokens: vec![
                Indexed::new(Token::Ident("hello".to_string()), 0..0),
                Indexed::new(Token::Ident("what".to_string()), 0..0),
                Indexed::new(Token::Ident("the".to_string()), 0..0),
                Indexed::new(Token::Ident("sigma".to_string()), 0..0),
            ]
        },]
    );
}
#[test]
pub fn lexer_ident_indent() {
    let text = "hello what\n    the sigma";
    let lines = Lexer::from(text).lex().unwrap();
    dbg!(&lines);
    assert_eq!(
        lines,
        vec![
            Line {
                indent: 0,
                ln: 0,
                tokens: vec![
                    Indexed::new(Token::Ident("hello".to_string()), 0..0),
                    Indexed::new(Token::Ident("what".to_string()), 0..0),
                ]
            },
            Line {
                indent: 4,
                ln: 1,
                tokens: vec![
                    Indexed::new(Token::Ident("the".to_string()), 0..0),
                    Indexed::new(Token::Ident("sigma".to_string()), 0..0),
                ]
            },
        ]
    );
}
#[test]
pub fn lexer_number() {
    let text = "1234 1_200 1.5 1. 139847651.12934781923 139_847_651.129_347_83";
    let lines = Lexer::from(text).lex().unwrap();
    dbg!(&lines);
    assert_eq!(
        lines,
        vec![Line {
            indent: 0,
            ln: 0,
            tokens: vec![
                Indexed::new(Token::Int(1234), 0..0),
                Indexed::new(Token::Int(1_200), 0..0),
                Indexed::new(Token::Float(1.5), 0..0),
                Indexed::new(Token::Float(1.), 0..0),
                Indexed::new(Token::Float(139_847_651.129_347_83), 0..0),
                Indexed::new(Token::Float(139_847_651.129_347_83), 0..0),
            ]
        },]
    );
}
#[test]
pub fn lexer_string() {
    let text = r#""hello world" "new\nline" """#;
    let lines = Lexer::from(text).lex().unwrap();
    dbg!(&lines);
    assert_eq!(
        lines,
        vec![Line {
            indent: 0,
            ln: 0,
            tokens: vec![
                Indexed::new(Token::String("hello world".to_string()), 0..0),
                Indexed::new(Token::String("new\nline".to_string()), 0..0),
                Indexed::new(Token::String("".to_string()), 0..0),
            ]
        },]
    );
}
#[test]
pub fn lexer_char() {
    let text = r#"'a' 'b' 'c' '\n' '\t' '\0'"#;
    let lines = Lexer::from(text).lex().unwrap();
    dbg!(&lines);
    assert_eq!(
        lines,
        vec![Line {
            indent: 0,
            ln: 0,
            tokens: vec![
                Indexed::new(Token::Char('a'), 0..0),
                Indexed::new(Token::Char('b'), 0..0),
                Indexed::new(Token::Char('c'), 0..0),
                Indexed::new(Token::Char('\n'), 0..0),
                Indexed::new(Token::Char('\t'), 0..0),
                Indexed::new(Token::Char('\0'), 0..0),
            ]
        },]
    );
}
#[test]
pub fn parser_stat_let() {
    let text = "let a = 1\nlet b = 2";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![
                    Located::new(
                        Statement::LetBinding {
                            param: Located::new(
                                Parameter::Ident("a".to_string()),
                                Default::default()
                            ),
                            expr: Located::new(Expression::Atom(Atom::Int(1)), Default::default()),
                        },
                        Default::default()
                    ),
                    Located::new(
                        Statement::LetBinding {
                            param: Located::new(
                                Parameter::Ident("b".to_string()),
                                Default::default()
                            ),
                            expr: Located::new(Expression::Atom(Atom::Int(2)), Default::default()),
                        },
                        Default::default()
                    )
                ]
            },
            Default::default()
        )
    )
}
#[test]
pub fn parser_stat_assign() {
    let text = "a = 1\nb = 2\na.b = 3";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![
                    Located::new(
                        Statement::Assign {
                            op: Default::default(),
                            path: Located::new(Path::Ident("a".to_string()), Default::default()),
                            expr: Located::new(Expression::Atom(Atom::Int(1)), Default::default()),
                        },
                        Default::default()
                    ),
                    Located::new(
                        Statement::Assign {
                            op: Default::default(),
                            path: Located::new(Path::Ident("b".to_string()), Default::default()),
                            expr: Located::new(Expression::Atom(Atom::Int(2)), Default::default()),
                        },
                        Default::default()
                    ),
                    Located::new(
                        Statement::Assign {
                            op: Default::default(),
                            path: Located::new(
                                Path::Field {
                                    head: Box::new(Located::new(
                                        Path::Ident("a".to_string()),
                                        Default::default()
                                    )),
                                    field: Located::new("b".to_string(), Default::default()),
                                },
                                Default::default()
                            ),
                            expr: Located::new(Expression::Atom(Atom::Int(3)), Default::default()),
                        },
                        Default::default()
                    )
                ]
            },
            Default::default()
        )
    )
}
#[test]
pub fn parser_stat_return() {
    let text = "return \"what\"\nreturn";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![
                    Located::new(
                        Statement::Return(Some(Located::new(
                            Expression::Atom(Atom::String("what".to_string())),
                            Default::default()
                        ))),
                        Default::default()
                    ),
                    Located::new(Statement::Return(None), Default::default()),
                ]
            },
            Default::default()
        )
    )
}
#[test]
pub fn parser_stat_call() {
    let text = "print(a)";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![Located::new(
                    Statement::Call {
                        head: Located::new(Path::Ident("print".to_string()), Default::default()),
                        args: vec![Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                            Default::default()
                        )],
                    },
                    Default::default()
                ),]
            },
            Default::default()
        )
    );
    let text = "print(a, b)";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![Located::new(
                    Statement::Call {
                        head: Located::new(Path::Ident("print".to_string()), Default::default()),
                        args: vec![
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                                Default::default()
                            ),
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                                Default::default()
                            )
                        ],
                    },
                    Default::default()
                ),]
            },
            Default::default()
        )
    );
    let text = "print(a, b,)";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![Located::new(
                    Statement::Call {
                        head: Located::new(Path::Ident("print".to_string()), Default::default()),
                        args: vec![
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                                Default::default()
                            ),
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                                Default::default()
                            )
                        ],
                    },
                    Default::default()
                ),]
            },
            Default::default()
        )
    );
    let text = "player:update(a)";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![Located::new(
                    Statement::SelfCall {
                        head: Located::new(Path::Ident("player".to_string()), Default::default()),
                        field: Located::new("update".to_string(), Default::default()),
                        args: vec![Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                            Default::default()
                        )],
                    },
                    Default::default()
                ),]
            },
            Default::default()
        )
    );
    let text = "player:update(a, b)";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![Located::new(
                    Statement::SelfCall {
                        head: Located::new(Path::Ident("player".to_string()), Default::default()),
                        field: Located::new("update".to_string(), Default::default()),
                        args: vec![
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                                Default::default()
                            ),
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                                Default::default()
                            )
                        ],
                    },
                    Default::default()
                ),]
            },
            Default::default()
        )
    );
    let text = "player:update(a, b,)";
    let chunk = parse(text).unwrap();
    dbg!(&chunk);
    assert_eq!(
        chunk,
        Located::new(
            Chunk {
                stats: vec![Located::new(
                    Statement::SelfCall {
                        head: Located::new(Path::Ident("player".to_string()), Default::default()),
                        field: Located::new("update".to_string(), Default::default()),
                        args: vec![
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                                Default::default()
                            ),
                            Located::new(
                                Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                                Default::default()
                            )
                        ],
                    },
                    Default::default()
                ),]
            },
            Default::default()
        )
    );
}
#[test]
pub fn parser_atom_expr() {
    let text = "(hello)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Expression(Box::new(Located::new(
                Expression::Atom(Atom::Path(Path::Ident("hello".to_string()))),
                Default::default()
            ))),
            Default::default()
        )
    );
    let text = "(\"fuck no\")";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Expression(Box::new(Located::new(
                Expression::Atom(Atom::String("fuck no".to_string())),
                Default::default()
            ))),
            Default::default()
        )
    );
}
#[test]
pub fn parser_atom_vector() {
    let text = "[1, 2, 3]";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Vector(vec![
                Located::new(Expression::Atom(Atom::Int(1)), Default::default()),
                Located::new(Expression::Atom(Atom::Int(2)), Default::default()),
                Located::new(Expression::Atom(Atom::Int(3)), Default::default()),
            ]),
            Default::default()
        )
    );
    let text = "[]";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(expr, Located::new(Atom::Vector(vec![]), Default::default()));
    let text = "[1]";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Vector(vec![Located::new(
                Expression::Atom(Atom::Int(1)),
                Default::default()
            ),]),
            Default::default()
        )
    );
    let text = "[1,]";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Vector(vec![Located::new(
                Expression::Atom(Atom::Int(1)),
                Default::default()
            ),]),
            Default::default()
        )
    );
    let text = "[1 2]";
    let err = parse::<Atom>(text).unwrap_err();
    dbg!(&err);
    assert_eq!(
        err.to_string(),
        Located::new(
            ParseError::Expected {
                expected: Token::Comma,
                got: Token::Int(2)
            },
            Default::default()
        )
        .to_string()
    );
}
#[test]
pub fn parser_atom_tuple() {
    let text = "(1, 2, 3)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Tuple(vec![
                Located::new(Expression::Atom(Atom::Int(1)), Default::default()),
                Located::new(Expression::Atom(Atom::Int(2)), Default::default()),
                Located::new(Expression::Atom(Atom::Int(3)), Default::default()),
            ]),
            Default::default()
        )
    );
    let text = "(1,)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Tuple(vec![Located::new(
                Expression::Atom(Atom::Int(1)),
                Default::default()
            ),]),
            Default::default()
        )
    );
    let text = "(1 2)";
    let err = parse::<Atom>(text).unwrap_err();
    dbg!(&err);
    assert_eq!(
        err.to_string(),
        Located::new(
            ParseError::Expected {
                expected: Token::ParanRight,
                got: Token::Int(2)
            },
            Default::default()
        )
        .to_string()
    );
}
#[test]
pub fn parser_atom_map() {
    let text = "{ a = 1, b = 2, c = 3 }";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Map(vec![
                (
                    Located::new("a".to_string(), Default::default()),
                    Located::new(Expression::Atom(Atom::Int(1)), Default::default())
                ),
                (
                    Located::new("b".to_string(), Default::default()),
                    Located::new(Expression::Atom(Atom::Int(2)), Default::default())
                ),
                (
                    Located::new("c".to_string(), Default::default()),
                    Located::new(Expression::Atom(Atom::Int(3)), Default::default())
                ),
            ]),
            Default::default()
        )
    );
    let text = "{}";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(expr, Located::new(Atom::Map(vec![]), Default::default()));
    let text = "{a=1}";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Map(vec![(
                Located::new("a".to_string(), Default::default()),
                Located::new(Expression::Atom(Atom::Int(1)), Default::default())
            )]),
            Default::default()
        )
    );
    let text = "{a=1,}";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Atom::Map(vec![(
                Located::new("a".to_string(), Default::default()),
                Located::new(Expression::Atom(Atom::Int(1)), Default::default())
            )]),
            Default::default()
        )
    );
    let text = "{a 1}";
    let err = parse::<Atom>(text).unwrap_err();
    dbg!(&err);
    assert_eq!(
        err.to_string(),
        Located::new(
            ParseError::Expected {
                expected: Token::Equal,
                got: Token::Int(1)
            },
            Default::default()
        )
        .to_string()
    );
}
#[test]
pub fn parser_expr_binary() {
    let text = "a + b * c";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Binary {
                op: BinaryOperator::Plus,
                left: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                    Default::default()
                )),
                right: Box::new(Located::new(
                    Expression::Binary {
                        op: BinaryOperator::Star,
                        left: Box::new(Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                            Default::default()
                        )),
                        right: Box::new(Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("c".to_string()))),
                            Default::default()
                        )),
                    },
                    Default::default()
                )),
            },
            Default::default()
        )
    );
    let text = "a * b + c";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Binary {
                op: BinaryOperator::Plus,
                left: Box::new(Located::new(
                    Expression::Binary {
                        op: BinaryOperator::Star,
                        left: Box::new(Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                            Default::default()
                        )),
                        right: Box::new(Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                            Default::default()
                        )),
                    },
                    Default::default()
                )),
                right: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("c".to_string()))),
                    Default::default()
                )),
            },
            Default::default()
        )
    );
}
#[test]
pub fn parser_expr_unary() {
    let text = "-a";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Unary {
                op: UnaryOperator::Minus,
                right: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                    Default::default()
                )),
            },
            Default::default()
        )
    );
    let text = "not a";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Unary {
                op: UnaryOperator::Not,
                right: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                    Default::default()
                )),
            },
            Default::default()
        )
    );
    let text = "--a";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Unary {
                op: UnaryOperator::Minus,
                right: Box::new(Located::new(
                    Expression::Unary {
                        op: UnaryOperator::Minus,
                        right: Box::new(Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                            Default::default()
                        )),
                    },
                    Default::default()
                )),
            },
            Default::default()
        )
    );
    let text = "not not a";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Unary {
                op: UnaryOperator::Not,
                right: Box::new(Located::new(
                    Expression::Unary {
                        op: UnaryOperator::Not,
                        right: Box::new(Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                            Default::default()
                        )),
                    },
                    Default::default()
                )),
            },
            Default::default()
        )
    );
    let text = "not -a";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Unary {
                op: UnaryOperator::Not,
                right: Box::new(Located::new(
                    Expression::Unary {
                        op: UnaryOperator::Minus,
                        right: Box::new(Located::new(
                            Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                            Default::default()
                        )),
                    },
                    Default::default()
                )),
            },
            Default::default()
        )
    );
}
#[test]
pub fn parser_expr_call() {
    let text = "print(a)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Call {
                head: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("print".to_string()))),
                    Default::default()
                )),
                args: vec![Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                    Default::default()
                )],
            },
            Default::default()
        )
    );
    let text = "print(a, b)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Call {
                head: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("print".to_string()))),
                    Default::default()
                )),
                args: vec![
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                        Default::default()
                    ),
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                        Default::default()
                    )
                ],
            },
            Default::default()
        )
    );
    let text = "print(a, b,)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::Call {
                head: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("print".to_string()))),
                    Default::default()
                )),
                args: vec![
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                        Default::default()
                    ),
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                        Default::default()
                    )
                ],
            },
            Default::default()
        )
    );
    let text = "player:update(a)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::SelfCall {
                head: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("player".to_string()))),
                    Default::default()
                )),
                field: Located::new("update".to_string(), Default::default()),
                args: vec![Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                    Default::default()
                )],
            },
            Default::default()
        )
    );
    let text = "player:update(a, b)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::SelfCall {
                head: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("player".to_string()))),
                    Default::default()
                )),
                field: Located::new("update".to_string(), Default::default()),
                args: vec![
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                        Default::default()
                    ),
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                        Default::default()
                    )
                ],
            },
            Default::default()
        )
    );
    let text = "player:update(a, b,)";
    let expr = parse(text).unwrap();
    dbg!(&expr);
    assert_eq!(
        expr,
        Located::new(
            Expression::SelfCall {
                head: Box::new(Located::new(
                    Expression::Atom(Atom::Path(Path::Ident("player".to_string()))),
                    Default::default()
                )),
                field: Located::new("update".to_string(), Default::default()),
                args: vec![
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("a".to_string()))),
                        Default::default()
                    ),
                    Located::new(
                        Expression::Atom(Atom::Path(Path::Ident("b".to_string()))),
                        Default::default()
                    )
                ],
            },
            Default::default()
        )
    );
}
