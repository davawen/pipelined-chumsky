#[derive(Debug, Clone, Copy, PartialEq)]
enum Token<'a> {
    Number(f32),
    String(&'a str),
    Ident(&'a str),
    Lparen,
    Rparen,
    Comma,
    Pipeline,
    Assign,
    Mutate,
    Lambda
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token as T;
        match self {
            T::Number(x) => write!(f, "{x}"),
            T::String(x) => write!(f, "{x:?}"),
            T::Ident(x) => write!(f, "{x}"),
            T::Lparen => write!(f, "("),
            T::Rparen => write!(f, ")"),
            T::Comma => write!(f, ","),
            T::Pipeline => write!(f, "|>"),
            T::Assign => write!(f, "->"),
            T::Mutate => write!(f, "->>"),
            T::Lambda => write!(f, "=>"),
        }
    }
}

fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<(Token<'a>, SimpleSpan)>, extra::Err<Rich<'a, char>>> {
    let number = text::digits(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| Token::Number(s.parse().unwrap()));

    /// using a closure complains about lifetimes
    macro_rules! symbol {
        ($c:literal, $t:expr) => {
           just($c).map(|_| $t)
        };
    }

    let symbols = choice((
        symbol!("->>", Token::Mutate),
        symbol!("->", Token::Assign),
        symbol!("=>", Token::Lambda),
        symbol!("|>", Token::Pipeline),
        symbol!("(", Token::Lparen),
        symbol!(")", Token::Rparen),
        symbol!(",", Token::Comma)
    ));

    let comment = just("--")
        .then(any().and_is(just("\n").not()).repeated())
        .padded();

    let allowed_ident = symbols.clone().not().rewind()
        .then_ignore(comment.not().rewind())
        .then(none_of(" \t\n\r"));

    let ident = text::digits(10).not().rewind()
        .then(allowed_ident.clone())
        .then(allowed_ident.repeated())
        .to_slice()
        .map(|s: &str| Token::Ident(s));

    let string = just('"')
        .ignore_then(none_of("\"").repeated().to_slice())
        .then_ignore(just('"'))
        .map(Token::String);

    let token = choice((number, string, ident, symbols));

    token
        .map_with(|t, e| (t, e.span()))
        .padded_by(comment.repeated())
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
        .then_ignore(end())
}

#[derive(Debug)]
pub enum Expr {
    Ident(String),
    Number(f32),
    String(String),
    Tuple(Vec<Expr>),
    Pipeline(Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, String),
    Mutate(Box<Expr>, String),
    Lambda(Vec<String>, Box<Expr>)
}

use std::fmt::Display;

use chumsky::{prelude::*, input::{StrInput, SpannedInput}, pratt::{self, left, infix, postfix, prefix}};

fn parser<'a>() -> impl Parser<'a, SpannedInput<Token<'a>, SimpleSpan, &'a [(Token<'a>, SimpleSpan)]>, Expr, chumsky::extra::Err<Rich<'a, Token<'a>>>> {
    let value = select !{
        Token::Number(x) => Expr::Number(x),
        Token::String(x) => Expr::String(x.to_owned())
    };

    let ident = select! { Token::Ident(x) => x.to_owned() };
    let eident = ident.map(Expr::Ident);

    recursive(|expr| {
        let tuple = value.or(expr.clone())
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::Lparen), just(Token::Rparen))
            .map(Expr::Tuple);

        let standalone = eident.or(tuple);

        let arg_list = ident
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::Lparen), just(Token::Rparen));

        standalone
            .pratt((
                infix(left(2), just(Token::Pipeline), |l, r| Expr::Pipeline(Box::new(l), Box::new(r))),
                postfix(1, just(Token::Mutate).ignore_then(ident), |l, r| Expr::Mutate(Box::new(l), r)),
                postfix(1, just(Token::Assign).ignore_then(ident), |l, r| Expr::Assign(Box::new(l), r)),
                prefix(1, arg_list.then_ignore(just(Token::Lambda)), |l, r| Expr::Lambda(l, Box::new(r)))
            ))
    })
       .lazy()
}

use ariadne::{Report, ReportKind, Label, Color, sources};

fn show_errs<T: Clone + Display>(src: &str, filename: &'static str, errs: Vec<Rich<T>>) {
    errs.into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        .for_each(|e| {
            Report::build(ReportKind::Error, filename, e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((filename, e.span().into_range()))
                        .with_message(format!("Expected: {:?}", e.expected().collect::<Vec<_>>()))
                        .with_color(Color::Red),
                )
                .finish()
                .print(sources([(filename, src)]))
                .unwrap()
        });
}

fn main() {
    const FILENAME: &str = "../test.pipe";

    let src = include_str!("../test.pipe");
    println!("{src}");

    let (tokens, errs) = lexer().parse(src).into_output_errors();

    show_errs(src, FILENAME, errs);

    let Some(tokens) = tokens else { return };

    let (ast, errs) = parser().parse(Input::spanned(&tokens, (src.len()..src.len()).into())).into_output_errors();

    show_errs(src, FILENAME, errs);

    let Some(ast) = ast else { return };

    println!("{ast:#?}");
}
