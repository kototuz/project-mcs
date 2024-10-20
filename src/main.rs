mod lexer;
mod parser;
mod compiler;
mod semantic;

use std::io::prelude::*;
use std::process::ExitCode;

type Result<T> = std::result::Result<T, ()>;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! exit_failure { () => { panic!(); } }

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! exit_failure { () => { std::process::exit(1); } }

#[macro_export]
macro_rules! lexical_err {
    ($loc:expr, $($arg:tt)*) => {
        eprint!("ERROR:{}: LexicalError: ", $loc);
        eprintln!($($arg)*);
        exit_failure!();
    }
}

#[macro_export]
macro_rules! syntax_err {
    ($loc:expr, $($arg:tt)*) => {
        eprint!("ERROR:{}: SyntaxError: ", $loc);
        eprintln!($($arg)*);
        exit_failure!();
    }
}

#[macro_export]
macro_rules! unexpected_token_err {
    ($loc:expr, $t:ident) => {
        syntax_err!($loc, "Unexpected {}", $t);
    }
}

#[macro_export]
macro_rules! semantic_err {
    ($loc:expr, $($arg:tt)*) => {
        eprint!("ERROR:{}: SemanticError: ", $loc);
        eprintln!($($arg)*);
        exit_failure!();
    }
}

#[macro_export]
macro_rules! compilation_err {
    ($($arg:tt)*) => {
        eprint!("ERROR: CompilationError: ");
        eprintln!($($arg)*);
        exit_failure!();
    }
}

fn main2() -> Result<()> {
    let file_path = std::env::args().nth(1).ok_or_else(|| {
        eprintln!("ERROR: source file must be provided");
    })?;

    let mut src_file = std::fs::File::open(&file_path).map_err(|err| {
        eprintln!("ERROR: could not open file `{file_path}`: {err}");
    })?;

    let mut buffer = String::new();
    let _ = src_file.read_to_string(&mut buffer).map_err(|err| {
        eprintln!("ERROR: could not read file `{file_path}`: {err}");
    })?;

    let mut lexer = lexer::Lexer::new(buffer.as_bytes()); // lexical analysis (lazy)
    let ast = parser::parse(&mut lexer);                  // syntax  analysis
    for fn_decl in &ast.fn_decls {
        println!("{fn_decl:?}");
    }
    compiler::compile(ast);                               // compilation


    //let program = parser::parse(&mut lexer);
    //semantic::analyze(program);
    //compiler::compile(program);

    Ok(())
}

fn main() -> ExitCode {
    match main2() {
        Err(_) => ExitCode::FAILURE,
        Ok(_)  => ExitCode::SUCCESS,
    }
}



// TODO: add something like that: `native!("say $var0")`
