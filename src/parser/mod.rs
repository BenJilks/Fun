mod function;
mod statement;
mod expression;
mod struct_;
mod data_type;
use function::parse_function;
use struct_::parse_struct;
use crate::tokenizer::{tokenize, Token, TokenType};
use crate::ast::SourceFile;
use std::fmt;
use std::iter::Peekable;
use std::fs::File;
use std::path::Path;
use std::error::Error;

#[derive(Debug)]
struct UnexpectedError
{
    expected: TokenType,
    got: Option<Token>,
}

trait TokenStream
{
    fn expect(&mut self, token_type: TokenType) -> Result<Token, Box<dyn Error>>;
    fn is_next(&mut self, token_type: TokenType) -> bool;
}

impl fmt::Display for UnexpectedError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let (content, position) = match &self.got
        {
            Some(token) => (token.content(), token.position()),
            None => ("Nothing", String::default()),
        };

        write!(f, "{}: Expected token '{}', but got '{}' instead",
             position, self.expected, content)
    }
}

impl Error for UnexpectedError {}

impl<Iter> TokenStream for Peekable<Iter>
    where Iter: Iterator<Item = Token>
{

    fn expect(&mut self, token_type: TokenType)
        -> Result<Token, Box<dyn Error>>
    {
        let token_or_none = self.next();
        if token_or_none.is_none() 
        {
            return Err(Box::from(UnexpectedError
            {
                expected: token_type,
                got: None,
            }));
        }

        let token = token_or_none.unwrap();
        if token.token_type() != &token_type
        {
            return Err(Box::from(UnexpectedError
            {
                expected: token_type,
                got: Some(token),
            }));
        }

        Ok(token)
    }

    fn is_next(&mut self, token_type: TokenType) -> bool
    {
        match self.peek()
        {
            Some(token) if token.token_type() == &token_type => true,
            _ => false,
        }
    }

}

pub fn parse(source_file_path: impl AsRef<Path>)
    -> Result<SourceFile, Box<dyn Error>>
{
    let file_path_str = source_file_path.as_ref().to_str().unwrap().to_owned();
    let file = File::open(source_file_path)?;
    let mut output = SourceFile::default();
    let mut tokens = tokenize(&file_path_str, file)?.peekable();

    loop
    {
        let token_or_none = tokens.next();
        if token_or_none.is_none() {
            break;
        }

        let token = token_or_none.unwrap();
        match token.token_type()
        {
            TokenType::Func => output.functions.push(parse_function(&mut tokens)?),
            TokenType::Struct => output.structs.push(parse_struct(&mut tokens)?),
            TokenType::Extern => output.externs.push(tokens.next().unwrap()),

            _ =>
            {
                return Err(Box::from(UnexpectedError
                {
                    expected: TokenType::Func,
                    got: Some(token)
                }));
            },
        }
    }

    Ok(output)
}

