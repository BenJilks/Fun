mod tokenizer;
mod data_type;
mod ast;
mod parser;
mod compiler;
mod optimizer;
use parser::parse;
use compiler::compile;
use optimizer::optimize;
use compiler::x86::X86CodeGenorator;
use std::env;
use std::process::exit;
use std::error::Error;

fn main()
    -> Result<(), Box<dyn Error>>
{
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1
    {
        eprintln!("No input files");
        exit(1);
    }

    let source_file_path = &args[1];
    let ast_or_error = parse(source_file_path);
    if ast_or_error.is_err() {
        eprintln!("Error: {}", ast_or_error.unwrap_err());
        exit(1);
    }

    let mut ast = ast_or_error.unwrap();
    optimize(&mut ast);

    let mut gen = X86CodeGenorator::new(std::io::stdout())?;
    match compile(&mut gen, ast)
    {
        Ok(_) => {},
        Err(err) =>
        {
            eprintln!("Error: {}", err);
            exit(1);
        },
    }

    Ok(())
}

