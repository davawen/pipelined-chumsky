use chumsky::prelude::*;
use super::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(String),
    Number(f64),
    String(String),
    Tuple(Vec<Expr>),
    Pipeline(Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, String),
    Mutate(Box<Expr>, String),
    Lambda(Vec<String>, Box<Expr>)
}

use chumsky::{prelude::*, input::{StrInput, SpannedInput}, pratt::{self, left, infix, postfix, prefix}};

pub fn parser<'a>() -> impl Parser<'a, SpannedInput<Token<'a>, SimpleSpan, &'a [(Token<'a>, SimpleSpan)]>, Expr, chumsky::extra::Err<Rich<'a, Token<'a>>>> {
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
