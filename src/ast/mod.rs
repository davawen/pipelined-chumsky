use std::{fmt::Display, path::Path};

use ariadne::{Report, ReportKind, Label, Color, sources, FileCache};
use chumsky::{prelude::{Rich, Input}, Parser};

mod lexer;
mod parser;

pub use parser::Expr;

use self::{lexer::lexer, parser::parser};

fn show_errs<T: Clone + Display>(src: &str, filename: &str, errs: Vec<Rich<T>>) {
    let mut cache = (filename, src.into());

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
                .eprint(&mut cache)
                .unwrap()
        });
}

pub fn parse(src: &str, filename: &str) -> Option<Expr> {
    let (tokens, errs) = lexer().parse(src).into_output_errors();

    show_errs(src, filename, errs);

    let tokens = tokens?;

    let (ast, errs) = parser().parse(Input::spanned(&tokens, (src.len()..src.len()).into())).into_output_errors();

    show_errs(src, filename, errs);

    ast
}
