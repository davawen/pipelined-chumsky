use std::{fs, path::{Path, PathBuf}};

use argh::FromArgs;

mod ast;
mod interpret;

#[derive(FromArgs)]
/// Interpreter and compiler for the pipelined programming language
struct Pipelined {
    #[argh(switch)]
    /// show the parsed program ast
    show_ast: bool,

    #[argh(switch)]
    /// show the source of the program
    show_source: bool,

    #[argh(positional)]
    filename: String
}

fn main() {
    let args: Pipelined = argh::from_env();

    let src = fs::read_to_string(&args.filename).unwrap();
    if args.show_source {
        println!("{src}");
    }

    let ast = ast::parse(&src, &args.filename);

    if args.show_ast {
        println!("{ast:#?}");
    }
}
