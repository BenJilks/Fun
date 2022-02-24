mod tokenizer;
mod data_type;
mod ast;
mod parser;
mod intermediate;
mod compiler;
mod optimizer;
mod code_generator;
use parser::parse;
use compiler::compile;
use optimizer::optimize;
use code_generator::x86;
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

    match compile(ast)
    {
        Ok(program) =>
        {
            if false
            {
                for function in &program.functions
                {
                    println!("{}:", function.name);
                    for instruction in &function.code {
                        println!("    {}", instruction);
                    }
                    println!();
                }
            }

            x86::generate(program, &mut std::io::stdout())?;
        },

        Err(err) =>
        {
            eprintln!("Error: {}", err);
            exit(1);
        },
    }

    Ok(())
}

