use super::TokenStream;
use crate::tokenizer::{Token, TokenType};
use crate::data_type::DataType;
use std::iter::Peekable;
use std::error::Error;

pub fn parse_data_type(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<DataType, Box<dyn Error>>
{
    let next_or_none = tokens.next();
    if next_or_none.is_none() {
        panic!();
    }

    let next = next_or_none.unwrap();
    let mut data_type = match next.token_type()
    {
        TokenType::Int => DataType::Int,
        TokenType::Char => DataType::Char,
        TokenType::Identifier => DataType::Struct(next.content().to_owned()),
        TokenType::Ref => DataType::Ref(Box::from(parse_data_type(tokens)?)),
        _ => panic!(),
    };

    while tokens.is_next(TokenType::OpenSquare)
    {
        tokens.expect(TokenType::OpenSquare)?;
        let size_token = tokens.expect(TokenType::IntLiteral)?;
        tokens.expect(TokenType::CloseSquare)?;

        let size = size_token.content().parse::<usize>()?;
        data_type = DataType::Array(Box::from(data_type), size);
    }

    while tokens.is_next(TokenType::Identifier)
    {
        let struct_name = tokens.next().unwrap();
        data_type = DataType::Generic(
            Box::from(data_type),
            struct_name.content().to_owned());
    }

    Ok(data_type)
}

