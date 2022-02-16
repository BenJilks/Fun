use crate::tokenizer::Token;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct CompilerError
{
    token: Option<Token>,
    message: String,
}

impl CompilerError
{

    pub fn new_no_position(message: String) -> Box<Self>
    {
        Box::from(Self
        {
            token: None,
            message,
        })
    }

    pub fn new(token: &Token, message: String) -> Box<Self>
    {
        Box::from(Self
        {
            token: Some(token.clone()),
            message,
        })
    }

}

impl fmt::Display for CompilerError
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match &self.token
        {
            Some(token) =>
            {
                writeln!(f, "{}: {}", token.position(), self.message)?;
                write!(f, "{}", token.show())
            },

            None =>
            {
                writeln!(f, "{}", self.message)
            },
        }
    }

}

impl Error for CompilerError {}

