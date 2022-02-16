use super::TokenStream;
use super::expression::parse_expression;
use crate::tokenizer::{Token, TokenType};
use crate::ast::{Statement, Let, If};
use std::iter::Peekable;
use std::error::Error;

fn parse_return_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    tokens.next();

    match parse_expression(tokens)?
    {
        Some(value) => Ok(Some(Statement::Return(value))),
        None => Ok(None),
    }
}

fn parse_expression_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    match parse_expression(tokens)?
    {
        Some(value) => Ok(Some(Statement::Expression(value))),
        None => Ok(None),
    }
}

fn parse_let_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    tokens.expect(TokenType::Let)?;
    let name = tokens.expect(TokenType::Identifier)?;
    tokens.expect(TokenType::Equals)?;
    let value = parse_expression(tokens)?;
    if value.is_none() {
        return Ok(None);
    }

    return Ok(Some(Statement::Let(Let
    {
        name: name,
        value: value.unwrap(),
    })));
}

pub fn parse_block(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Vec<Statement>, Box<dyn Error>>
{
    if !tokens.is_next(TokenType::OpenSquiggly) 
    {
        let statement = parse_statement(tokens)?;
        return Ok(vec![statement.unwrap()]);
    }

    tokens.expect(TokenType::OpenSquiggly)?;
    let mut block = Vec::new();
    loop
    {
        let statement = parse_statement(tokens)?;
        if statement.is_none() {
            break;
        }
        block.push(statement.unwrap());
    }

    tokens.expect(TokenType::CloseSquiggly)?;
    Ok(block)
}

fn parse_else(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Vec<Statement>>, Box<dyn Error>>
{
    if tokens.is_next(TokenType::Else)
    {
        tokens.expect(TokenType::Else)?;
        tokens.expect(TokenType::Arrow)?;
        Ok(Some(parse_block(tokens)?))
    }
    else
    {
        Ok(None)
    }
}

fn parse_if_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    tokens.expect(TokenType::If)?;
    let condition = parse_expression(tokens)?;
    assert!(condition.is_some());

    tokens.expect(TokenType::Arrow)?;
    let block = parse_block(tokens)?;
    let else_block = parse_else(tokens)?;
    Ok(Some(Statement::If(If
    {
        condition: condition.unwrap(),
        block,
        else_block,
    })))
}

fn parse_loop_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    tokens.expect(TokenType::Loop)?;
    let block = parse_block(tokens)?;
    Ok(Some(Statement::Loop(block)))
}

fn parse_while_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    tokens.expect(TokenType::While)?;
    let condition = parse_expression(tokens)?;
    assert!(condition.is_some());

    tokens.expect(TokenType::Arrow)?;
    let block = parse_block(tokens)?;
    Ok(Some(Statement::While(condition.unwrap(), block)))
}

fn parse_break_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    tokens.expect(TokenType::Break)?;
    Ok(Some(Statement::Break))
}

pub fn parse_statement(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Statement>, Box<dyn Error>>
{
    let next = tokens.peek();
    if next.is_none() {
        return Ok(None);
    }

    match next.unwrap().token_type()
    {
        TokenType::Return => Ok(parse_return_statement(tokens)?),
        TokenType::Let => Ok(parse_let_statement(tokens)?),
        TokenType::If => Ok(parse_if_statement(tokens)?),
        TokenType::Loop => Ok(parse_loop_statement(tokens)?),
        TokenType::While => Ok(parse_while_statement(tokens)?),
        TokenType::Break => Ok(parse_break_statement(tokens)?),
        _ => Ok(parse_expression_statement(tokens)?),
    }
}

