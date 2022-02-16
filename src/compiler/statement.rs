use super::code_genorator::CodeGenortator;
use super::name_table::Scope;
use super::data_type::{size_of, derive_data_type};
use super::error::CompilerError;
use super::expression::compile_expression;
use crate::ast::{Expression, Let, If, Statement};
use crate::data_type::DataType;
use std::rc::Rc;
use std::error::Error;

fn compile_let<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>, let_: &Let)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let data_type = derive_data_type(scope, &let_.value)?;
    let size = size_of(scope, &data_type)?;

    let local = gen.allocate_local(size_of(scope, &data_type)?)?;
    let value = compile_expression(gen, scope, &let_.value)?;
    gen.mov(local.clone(), value, size)?;

    // TODO: Proper error handling here.
    assert!(scope.values().put(let_.name.content(), (local, data_type)));

    Ok(())
}

fn compile_return<Gen, Value>(gen: &mut Gen,
                              scope: &mut Scope<Value>,
                              expression: &Expression,
                              return_type: Option<&DataType>,
                              return_to: Option<Rc<Value>>)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
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
    gen.ret(value, size, return_to)
}

fn compile_block<Gen, Value>(gen: &mut Gen,
                             scope: &mut Scope<Value>,
                             block: &Vec<Statement>,
                             return_type: Option<&DataType>,
                             return_to: Option<Rc<Value>>,
                             end_label: Option<&str>)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let mut local_scope = Scope::new(Some(Box::from(scope.clone())));
    for statement in block
    {
        compile_statement(gen, &mut local_scope,
            statement, return_type, return_to.clone(), end_label)?;
    }

    Ok(())
}

fn compile_if<Gen, Value>(gen: &mut Gen,
                          scope: &mut Scope<Value>,
                          if_: &If,
                          return_type: Option<&DataType>,
                          return_to: Option<Rc<Value>>,
                          end_label: Option<&str>)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let else_label = gen.create_label("else");
    let end_if_label = gen.create_label("end_if");

    let condition_value = compile_expression(gen, scope, &if_.condition)?;
    gen.goto_if_not(&else_label, condition_value)?;

    compile_block(gen, scope, &if_.block,
        return_type, return_to.clone(), end_label)?;
    gen.goto(&end_if_label)?;

    gen.emit_label(&else_label)?;
    match &if_.else_block
    {
        Some(block) =>
        {
            compile_block(gen, scope, block,
                return_type, return_to, end_label)?
        },
        None => {},
    }

    gen.emit_label(&end_if_label)?;
    Ok(())
}

fn compile_loop<Gen, Value>(gen: &mut Gen,
                            scope: &mut Scope<Value>,
                            block: &Vec<Statement>,
                            return_type: Option<&DataType>,
                            return_to: Option<Rc<Value>>)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let start_label = gen.create_label("loop_start");
    let end_label = gen.create_label("loop_end");

    gen.emit_label(&start_label)?;
    compile_block(gen, scope, block,
        return_type, return_to, Some(&end_label))?;
    gen.goto(&start_label)?;
    gen.emit_label(&end_label)?;

    Ok(())
}

fn compile_while<Gen, Value>(gen: &mut Gen,
                             scope: &mut Scope<Value>,
                             condition: &Expression,
                             block: &Vec<Statement>,
                             return_type: Option<&DataType>,
                             return_to: Option<Rc<Value>>)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let start_label = gen.create_label("while_start");
    let end_label = gen.create_label("while_end");

    gen.emit_label(&start_label)?;

    let condition_value = compile_expression(gen, scope, condition)?;
    gen.goto_if_not(&end_label, condition_value)?;

    compile_block(gen, scope, block,
        return_type, return_to, Some(&end_label))?;
    gen.goto(&start_label)?;
    gen.emit_label(&end_label)?;

    Ok(())
}

fn compile_break<Gen, Value>(gen: &mut Gen, loop_end: Option<&str>)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    assert!(loop_end.is_some());
    gen.goto(loop_end.unwrap())?;
    Ok(())
}

pub fn compile_statement<Gen, Value>(gen: &mut Gen,
                                     scope: &mut Scope<Value>,
                                     statement: &Statement,
                                     return_type: Option<&DataType>,
                                     return_to: Option<Rc<Value>>,
                                     loop_end: Option<&str>)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
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
            compile_break(gen, loop_end)?,
    };

    Ok(())
}

