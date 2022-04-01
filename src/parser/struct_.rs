use super::TokenStream;
use super::data_type::{parse_data_type, parse_type_variable};
use crate::tokenizer::{Token, TokenType};
use crate::ast::{Struct, Field};
use std::iter::Peekable;
use std::error::Error;

pub fn parse_struct(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Struct, Box<dyn Error>>
{
    let name = tokens.expect(TokenType::Identifier)?;
    let type_variable = parse_type_variable(tokens)?;

    tokens.expect(TokenType::OpenSquiggly)?;
    let mut fields = Vec::new();
    loop
    {
        if !tokens.is_next(TokenType::Identifier) {
            break;
        }

        let field_name = tokens.next().unwrap();
        tokens.expect(TokenType::Colon)?;
        let field_data_type = parse_data_type(tokens)?;

        fields.push(Field
        {
            name: field_name,
            data_type: field_data_type,
        });

        if !tokens.is_next(TokenType::Comma) {
            break;
        }
        tokens.expect(TokenType::Comma)?;
    }
    tokens.expect(TokenType::CloseSquiggly)?;

    Ok(Struct
    {
        name,
        type_variable,
        fields,
    })
}

