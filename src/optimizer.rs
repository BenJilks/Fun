use crate::ast::{SourceFile, Function, Statement, If};
use crate::ast::{Expression, Operation, OperationType};

enum PreComputedValue
{
    Int(i32),
    Bool(bool),
    Unkown,
}

fn pre_compute_operation(operation: &mut Operation) -> PreComputedValue
{
    if operation.rhs.is_none() {
        return PreComputedValue::Unkown;
    }

    let pre_computed_lhs = pre_compute_expression(&mut operation.lhs);
    let pre_computed_rhs = pre_compute_expression(operation.rhs.as_mut().unwrap());
    match pre_computed_lhs
    {
        PreComputedValue::Int(lhs) =>
        {
            match pre_computed_rhs
            {
                PreComputedValue::Int(rhs) =>
                {
                    match operation.operation_type
                    {
                        OperationType::Add => PreComputedValue::Int(lhs + rhs),
                        OperationType::Subtract => PreComputedValue::Int(lhs - rhs),
                        OperationType::GreaterThan => PreComputedValue::Bool(lhs > rhs),
                        OperationType::LessThan => PreComputedValue::Bool(lhs < rhs),
                        _ => PreComputedValue::Unkown,
                    }
                },

                _ => PreComputedValue::Unkown,
            }
        },

        _ => PreComputedValue::Unkown,
    }
}

fn pre_compute_expression(expression: &mut Expression) -> PreComputedValue
{
    let value = match expression
    {
        Expression::Operation(operation) => pre_compute_operation(operation),
        Expression::IntLiteral(i) => PreComputedValue::Int(*i),
        _ => PreComputedValue::Unkown,
    };

    match value
    {
        PreComputedValue::Int(i) => *expression = Expression::IntLiteral(i),
        PreComputedValue::Bool(b) => *expression = Expression::BoolLiteral(b),
        _ => {},
    };
    value
}

fn optimize_expression(expression: &mut Expression)
{
    pre_compute_expression(expression);
}

fn optimize_block(block: &mut Vec<Statement>)
{
    for statement in block {
        optimize_statement(statement);
    }
}

fn optimize_if(if_: &mut If)
{
    optimize_expression(&mut if_.condition);
    optimize_block(&mut if_.block);
    if if_.else_block.is_some() {
        optimize_block(if_.else_block.as_mut().unwrap());
    }
}

fn optimize_loop(block: &mut Vec<Statement>)
{
    optimize_block(block);
}

fn optimize_while(condition: &mut Expression, block: &mut Vec<Statement>)
{
    optimize_expression(condition);
    optimize_block(block);
}

fn optimize_statement(statement: &mut Statement)
{
    match statement
    {
        Statement::Expression(expression) => optimize_expression(expression),
        Statement::Return(expression) => optimize_expression(expression),
        Statement::Let(let_) => optimize_expression(&mut let_.value),
        Statement::If(if_) => optimize_if(if_),
        Statement::Loop(block) => optimize_loop(block),
        Statement::While(condition, block) => optimize_while(condition, block),
        Statement::Break => {},
    }
}

fn optimize_function(function: &mut Function)
{
    if function.body.is_none() {
        return;
    }

    for statement in function.body.as_mut().unwrap() {
        optimize_statement(statement);
    }
}

pub fn optimize(ast: &mut SourceFile)
{
    for function in &mut ast.functions {
        optimize_function(function);
    }
}

