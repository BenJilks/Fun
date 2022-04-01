use super::TokenStream;
use super::data_type::parse_data_type;
use crate::tokenizer::{Token, TokenType};
use crate::ast::{Expression, Operation, OperationType};
use crate::ast::{Call, InitializerList};
use std::iter::Peekable;
use std::error::Error;

const TERM_OPERATIONS: [(TokenType, OperationType); 2] =
[
    (TokenType::Dot, OperationType::Access),
    (TokenType::OpenSquare, OperationType::Indexed),
];

const ARITHMATIC_OPERATIONS: [(TokenType, OperationType); 3] =
[
    (TokenType::Plus, OperationType::Add),
    (TokenType::Star, OperationType::Multiply),
    (TokenType::Minus, OperationType::Subtract),
];

const LOGIC_OPERATIONS: [(TokenType, OperationType); 2] =
[
    (TokenType::GreaterThan, OperationType::GreaterThan),
    (TokenType::LessThan, OperationType::LessThan),
];

const EXPRESSION_OPERATIONS: [(TokenType, OperationType); 1] =
[
    (TokenType::Equals, OperationType::Assign),
];

fn parse_unary(tokens: &mut Peekable<impl Iterator<Item = Token>>,
               operation_type: OperationType)
    -> Result<Option<Expression>, Box<dyn Error>>
{
    tokens.next();

    let value = parse_expression(tokens)?;
    if value.is_none() {
        return Ok(None);
    }

    Ok(Some(Expression::Operation(Operation
    {
        operation_type,
        lhs: Box::from(value.unwrap()),
        rhs: None,
    })))
}

fn parse_initializer_list(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Expression, Box<dyn Error>>
{
    tokens.expect(TokenType::New)?;
    let data_type = parse_data_type(tokens)?;

    tokens.expect(TokenType::OpenSquiggly)?;
    let mut initializer_list = Vec::new();
    loop
    {
        if !tokens.is_next(TokenType::Identifier) {
            break;
        }

        let field_name = tokens.expect(TokenType::Identifier)?;
        tokens.expect(TokenType::Equals)?;
        let field_value = parse_expression(tokens)?;

        // TODO: Proper error checking here
        assert_eq!(field_value.is_some(), true);

        initializer_list.push((field_name, field_value.unwrap()));

        if !tokens.is_next(TokenType::Comma) {
            break;
        }
        tokens.expect(TokenType::Comma)?;
    }

    tokens.expect(TokenType::CloseSquiggly)?;
    Ok(Expression::InitializerList(InitializerList
    {
        data_type,
        list: initializer_list,
    }))
}

fn parse_array(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Expression, Box<dyn Error>>
{
    tokens.expect(TokenType::OpenSquare)?;

    let mut array = Vec::new();
    loop
    {
        let item = parse_expression(tokens)?;
        if item.is_none() {
            break;
        }
        array.push(item.unwrap());

        if !tokens.is_next(TokenType::Comma) {
            break;
        }
        tokens.next();
    }

    tokens.expect(TokenType::CloseSquare)?;
    Ok(Expression::ArrayLiteral(array))
}

fn parse_extern_call(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Expression, Box<dyn Error>>
{
    tokens.expect(TokenType::Extern)?;
    match parse_expression(tokens)?
    {
        Some(Expression::Call(call)) =>
            Ok(Expression::ExternCall(call)),
        _ => panic!(),
    }
}

pub fn parse_value(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Expression>, Box<dyn Error>>
{
    let next = tokens.peek();
    if next.is_none() {
        return Ok(None);
    }

    Ok(match next.unwrap().token_type()
    {
        TokenType::IntLiteral => 
            Some(Expression::IntLiteral(tokens.next().unwrap().content().parse::<i32>().unwrap())),

        TokenType::StringLiteral => 
            Some(Expression::StringLiteral(tokens.next().unwrap())),

        TokenType::CharLiteral => 
            Some(Expression::CharLiteral(tokens.next().unwrap())),

        TokenType::BoolLiteral => 
            Some(Expression::BoolLiteral(tokens.next().unwrap().content() == "true")),

        TokenType::Identifier =>
            Some(Expression::Identifier(tokens.next().unwrap())),

        TokenType::New =>
            Some(parse_initializer_list(tokens)?),

        TokenType::Extern =>
            Some(parse_extern_call(tokens)?),

        TokenType::OpenSquare =>
            Some(parse_array(tokens)?),

        TokenType::Ref =>
            parse_unary(tokens, OperationType::Ref)?,

        TokenType::Sizeof =>
            parse_unary(tokens, OperationType::Sizeof)?,

        TokenType::Deref =>
            parse_unary(tokens, OperationType::Deref)?,

        _ =>
            None,
    })
}

fn parse_call(tokens: &mut Peekable<impl Iterator<Item = Token>>,
              value: Expression)
    -> Result<Expression, Box<dyn Error>>
{
    tokens.expect(TokenType::OpenBracket)?;

    let mut arguments = Vec::new();
    loop
    {
        let argument = parse_expression(tokens)?;
        if argument.is_none() {
            break;
        }

        arguments.push(argument.unwrap());
        if !tokens.is_next(TokenType::Comma) {
            break;
        }
        tokens.next();
    }

    tokens.expect(TokenType::CloseBracket)?;
    let type_variable = 
        if tokens.is_next(TokenType::Of)
        {
            tokens.next();
            Some(parse_data_type(tokens)?)
        }
        else 
        {
            None
        };

    return Ok(Expression::Call(Call
    {
        callable: Box::from(value),
        arguments,
        type_variable,
    }))
}

fn parse_computed_value(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Expression>, Box<dyn Error>>
{
    let value = parse_value(tokens)?;
    if value.is_none() {
        return Ok(None);
    }

    let next = tokens.peek();
    if next.is_none() {
        return Ok(value);
    }

    match next.unwrap().token_type()
    {
        TokenType::OpenBracket =>
            Ok(Some(parse_call(tokens, value.unwrap())?)),

        _ => Ok(value),
    }
}

fn next_operation(tokens: &mut Peekable<impl Iterator<Item = Token>>,
                  operations: &[(TokenType, OperationType)])
    -> Option<OperationType>
{
    let next = tokens.peek();
    if next.is_none() {
        return None;
    }

    let token_type = next.unwrap().token_type();
    for (operation_token_type, operation_type) in operations
    {
        if token_type == operation_token_type {
            return Some(operation_type.clone());
        }
    }
    
    None
}

fn parse_operation<ParseFunc, Tokens>(tokens: &mut Peekable<Tokens>,
                                      lhs: Expression, parse_rhs: ParseFunc,
                                      operation: OperationType)
        -> Result<Expression, Box<dyn Error>>
    where ParseFunc: Fn(&mut Peekable<Tokens>) -> Result<Option<Expression>, Box<dyn Error>>,
          Tokens: Iterator<Item = Token>
{
    let rhs =
        match parse_rhs(tokens)?
        {
            Some(rhs) => Some(Box::from(rhs)),
            None => None,
        };

    Ok(Expression::Operation(Operation
    {
        operation_type: operation,
        lhs: Box::from(lhs),
        rhs: rhs,
    }))
}

fn parse_operation_order<F, T>(tokens: &mut Peekable<T>,
                               parse_operand: &F,
                               operations: &[(TokenType, OperationType)])
        -> Result<Option<Expression>, Box<dyn Error>>
    where F: Fn(&mut Peekable<T>) -> Result<Option<Expression>, Box<dyn Error>>,
          T: Iterator<Item = Token>,
{
    let lhs_or_none = parse_operand(tokens)?;
    if lhs_or_none.is_none() {
        return Ok(None);
    }

    let mut lhs = lhs_or_none.unwrap();
    loop
    {
        let operation_or_none = next_operation(tokens, operations);
        if operation_or_none.is_none() {
            break;
        }

        let operation = operation_or_none.unwrap();
        tokens.next();
        lhs = parse_operation(tokens, lhs,
            parse_operand, operation.clone())?;

        // NOTE: Little hacky
        if operation == OperationType::Indexed {
            tokens.expect(TokenType::CloseSquare)?;
        }
    }

    Ok(Some(lhs))
}

fn parse_term(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Expression>, Box<dyn Error>>
{
    parse_operation_order(tokens, &parse_computed_value, &TERM_OPERATIONS)
}

fn parse_arithmatic(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Expression>, Box<dyn Error>>
{
    parse_operation_order(tokens, &parse_term, &ARITHMATIC_OPERATIONS)
}

fn parse_logic(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Expression>, Box<dyn Error>>
{
    parse_operation_order(tokens, &parse_arithmatic, &LOGIC_OPERATIONS)
}

pub fn parse_expression(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<Option<Expression>, Box<dyn Error>>
{
    parse_operation_order(tokens, &parse_logic, &EXPRESSION_OPERATIONS)
}

