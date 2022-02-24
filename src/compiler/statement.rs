use super::intermediate::IRGenorator;
use super::intermediate::value::IRValue;
use super::name_table::Scope;
use super::data_type::{size_of, derive_data_type};
use super::error::CompilerError;
use super::expression::compile_expression;
use crate::ast::{Expression, Let, If, Statement};
use crate::data_type::DataType;
use std::rc::Rc;
use std::error::Error;

fn compile_let(gen: &mut IRGenorator, scope: &mut Scope, let_: &Let)
    -> Result<(), Box<dyn Error>>
{
    let data_type = derive_data_type(scope, &let_.value)?;
    let local = gen.allocate_local(size_of(scope, &data_type)?);
    let value = compile_expression(gen, scope, &let_.value)?;
    gen.mov(local.clone(), value);

    // TODO: Proper error handling here.
    assert!(scope.put_value(let_.name.content().to_owned(), local, data_type));

    Ok(())
}

fn compile_return(gen: &mut IRGenorator,
                  scope: &mut Scope,
                  expression: &Expression,
                  return_type: Option<&DataType>,
                  return_to: Option<Rc<IRValue>>)
    -> Result<(), Box<dyn Error>>
{
    let data_type = derive_data_type(scope, &expression)?;
    if return_type.is_some() && return_type.unwrap() != &data_type
    {
        // FIXME: What happens if we don't have a token?
        return Err(CompilerError::new(expression.token().unwrap(), format!(
            "Can not return type '{:?}' from a function with return type '{:?}'",
            data_type, return_type.unwrap())));
    }

    let value = compile_expression(gen, scope, expression)?;
    let size = size_of(scope, &data_type)?;
    if return_to.is_some()
    {
        let return_to = return_to.unwrap();
        gen.mov(return_to.clone(), value);
        return Ok(gen.ret(return_to, size));
    }

    Ok(gen.ret(value, size))
}

fn compile_block(gen: &mut IRGenorator,
                 scope: &mut Scope,
                 block: &Vec<Statement>,
                 return_type: Option<&DataType>,
                 return_to: Option<Rc<IRValue>>,
                 end_label: Option<&str>)
    -> Result<(), Box<dyn Error>>
{
    let mut local_scope = Scope::new(Some(scope));
    for statement in block
    {
        compile_statement(gen, &mut local_scope,
            statement, return_type, return_to.clone(), end_label)?;
    }

    Ok(())
}

fn compile_if(gen: &mut IRGenorator,
              scope: &mut Scope,
              if_: &If,
              return_type: Option<&DataType>,
              return_to: Option<Rc<IRValue>>,
              end_label: Option<&str>)
    -> Result<(), Box<dyn Error>>
{
    let else_label = gen.create_label("else");
    let end_if_label = gen.create_label("end_if");

    let condition_value = compile_expression(gen, scope, &if_.condition)?;
    gen.goto_if_not(&else_label, condition_value);

    compile_block(gen, scope, &if_.block,
        return_type, return_to.clone(), end_label)?;
    gen.goto(&end_if_label);

    gen.emit_label(&else_label);
    match &if_.else_block
    {
        Some(block) =>
        {
            compile_block(gen, scope, block,
                return_type, return_to, end_label)?
        },
        None => {},
    }

    gen.emit_label(&end_if_label);
    Ok(())
}

fn compile_loop(gen: &mut IRGenorator,
                scope: &mut Scope,
                block: &Vec<Statement>,
                return_type: Option<&DataType>,
                return_to: Option<Rc<IRValue>>)
    -> Result<(), Box<dyn Error>>
{
    let start_label = gen.create_label("loop_start");
    let end_label = gen.create_label("loop_end");

    gen.emit_label(&start_label);
    compile_block(gen, scope, block,
        return_type, return_to, Some(&end_label))?;
    gen.goto(&start_label);
    gen.emit_label(&end_label);

    Ok(())
}

fn compile_while(gen: &mut IRGenorator,
                 scope: &mut Scope,
                 condition: &Expression,
                 block: &Vec<Statement>,
                 return_type: Option<&DataType>,
                 return_to: Option<Rc<IRValue>>)
    -> Result<(), Box<dyn Error>>
{
    let start_label = gen.create_label("while_start");
    let end_label = gen.create_label("while_end");

    gen.emit_label(&start_label);

    let condition_value = compile_expression(gen, scope, condition)?;
    gen.goto_if_not(&end_label, condition_value);

    compile_block(gen, scope, block,
        return_type, return_to, Some(&end_label))?;
    gen.goto(&start_label);
    gen.emit_label(&end_label);

    Ok(())
}

fn compile_break(gen: &mut IRGenorator, loop_end: Option<&str>)
{
    assert!(loop_end.is_some());
    gen.goto(loop_end.unwrap());
}

pub fn compile_statement(gen: &mut IRGenorator,
                         scope: &mut Scope,
                         statement: &Statement,
                         return_type: Option<&DataType>,
                         return_to: Option<Rc<IRValue>>,
                         loop_end: Option<&str>)
    -> Result<(), Box<dyn Error>>
{
    match statement
    {
        Statement::Expression(expression) =>
            { compile_expression(gen, scope, expression)?; },

        Statement::Let(let_) =>
            compile_let(gen, scope, let_)?,

        Statement::If(if_) =>
            compile_if(gen, scope, if_, return_type, return_to, loop_end)?,

        Statement::Return(expression) =>
            compile_return(gen, scope, expression, return_type, return_to)?,

        Statement::Loop(block) =>
            compile_loop(gen, scope, block, return_type, return_to)?,

        Statement::While(condition, block) =>
            compile_while(gen, scope, condition, block, return_type, return_to)?,

        Statement::Break =>
            compile_break(gen, loop_end),
    };

    Ok(())
}

