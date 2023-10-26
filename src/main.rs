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

use ariadne::{Report, ReportKind, Label, Color, sources};
use chumsky::{prelude::*, input::StrInput, pratt::{self, left, infix, postfix, prefix}};

fn parser<'a>() -> impl Parser<'a, &'a str, Expr, chumsky::extra::Err<Rich<'a, char>>> {
    // let p = just("--")
    //     .then(none_of("\n").repeated())
    //     .then(just('\n'))
    //     .padded
    //     .or();

    let number = text::digits(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| Expr::Number(s.parse().unwrap()));

    let no = |x| just(x).not().rewind();

    let allowed_ident = no("->>").ignored()
        .then_ignore(no("->"))
        .then_ignore(no("=>"))
        .then_ignore(no("|>"))
        .then_ignore(no("--"))
        .then(none_of("(), \t\n\r"));

    let ident = text::digits(10).not().rewind()
        .then(allowed_ident)
        .then(allowed_ident.repeated())
        .to_slice()
        .map(|s: &str| s.to_owned());

    let string = just('"')
        .ignore_then(none_of("\"").repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(Expr::String);

    recursive(|expr| {
        let value = string.or(number);

        let tuple = value.or(expr.clone())
            .padded().separated_by(just(','))
            .collect::<Vec<_>>()
            .delimited_by(just('('), just(')'))
            .map(Expr::Tuple);

        let standalone = ident.map(Expr::Ident).or(tuple);

        let arg_list = ident
            .padded().separated_by(just(','))
            .collect::<Vec<_>>()
            .delimited_by(just('('), just(')'));

        let op = |c| just(c).padded();

        standalone
            .pratt((
                infix(left(2), op("|>"), |l, r| Expr::Pipeline(Box::new(l), Box::new(r))),
                postfix(1, op("->>").ignore_then(ident), |l, r| Expr::Mutate(Box::new(l), r)),
                postfix(1, op("->").ignore_then(ident), |l, r| Expr::Assign(Box::new(l), r)),
                prefix(1, arg_list.padded().then_ignore(op("=>")), |l, r| Expr::Lambda(l, Box::new(r)))
            ))
    })
        .lazy()
        .then_ignore(end())
}

fn main() {
    const FILENAME: &str = "../test.pipe";

    let src = include_str!("../test.pipe");
    println!("{src}");
    let (o, errs) = parser().parse(src).into_output_errors();

    if let Some(output) = o {
        println!("{output:#?}");
    }

    errs.into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        .for_each(|e| {
            Report::build(ReportKind::Error, FILENAME, e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((FILENAME, e.span().into_range()))
                        .with_message(format!("Expected: {:?}", e.expected().collect::<Vec<_>>()))
                        .with_color(Color::Red),
                )
                .finish()
                .print(sources([(FILENAME, src.clone())]))
                .unwrap()
        });
}
