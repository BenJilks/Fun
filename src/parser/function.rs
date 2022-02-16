use super::TokenStream;
use super::statement::parse_block;
use super::data_type::parse_data_type;
use crate::tokenizer::{Token, TokenType};
use crate::ast::{Function, Param, Statement};
use crate::data_type::DataType;
use std::iter::Peekable;
use std::error::Error;

fn parse_function_params(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Vec<Param>, Box<dyn Error>>
{
    let mut params = Vec::<Param>::new();
    tokens.expect(TokenType::OpenBracket)?;

    loop
    {
        if !tokens.is_next(TokenType::Identifier) {
            break;
        }

        let name = tokens.expect(TokenType::Identifier)?;
        tokens.expect(TokenType::Colon)?;
        let data_type = parse_data_type(tokens)?;

        params.push(Param
        {
            name,
            data_type,
        });

        if !tokens.is_next(TokenType::Comma) {
            break;
        }
        tokens.expect(TokenType::Comma)?;
    }

    tokens.expect(TokenType::CloseBracket)?;
    Ok(params)
}

pub fn parse_function_body(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Vec<Statement>>, Box<dyn Error>>
{
    if !tokens.is_next(TokenType::OpenSquiggly) {
        return Ok(None);
    }

    let body = parse_block(tokens)?;
    Ok(Some(body))
}

fn parse_function_return_type(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<DataType>, Box<dyn Error>>
{
    if !tokens.is_next(TokenType::Arrow) {
        return Ok(None);
    }

    tokens.expect(TokenType::Arrow)?;
    Ok(Some(parse_data_type(tokens)?))
}

pub fn parse_function(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Function, Box<dyn Error>>
{
    let name = tokens.expect(TokenType::Identifier)?;
    let params = parse_function_params(tokens)?;
    let return_type = parse_function_return_type(tokens)?;
    let body = parse_function_body(tokens)?;

    Ok(Function
    {
        name,
        params,
        return_type,
        body,
    })
}

