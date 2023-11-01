use chumsky::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'a> {
    Number(f64),
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

pub fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<(Token<'a>, SimpleSpan)>, extra::Err<Rich<'a, char>>> {
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
