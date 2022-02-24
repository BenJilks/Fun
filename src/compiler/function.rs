use super::intermediate::IRGenorator;
use super::intermediate::value::IRValue;
use super::statement::compile_statement;
use super::name_table::{FunctionDescriptionType, CompiledFunction};
use super::name_table::Scope;
use super::data_type::{size_of, derive_data_type};
use super::data_type::{function_signature, call_signature};
use super::error::CompilerError;
use crate::tokenizer::Token;
use crate::ast::{Function, Statement, Call};
use crate::data_type::DataType;
use std::rc::Rc;
use std::collections::HashSet;
use std::error::Error;

fn function_from_description(scope: &mut Scope,
                             description: FunctionDescriptionType,
                             function_name: &str,
                             call: &Call)
    -> Result<(String, CompiledFunction), Box<dyn Error>>
{
    let params = call.arguments
        .iter()
        .map(|argument| derive_data_type(scope, argument))
        .collect::<Result<Vec<_>, _>>()?;

    let signature = call_signature(scope, function_name, call)?;
    let return_type = description.return_type.clone();
    let function = CompiledFunction
    {
        name: function_name.to_owned(),
        params,
        return_type,
    };

    scope.put_used_function(function.clone());
    Ok((signature, function))
}

pub fn find_function_for_call(scope: &mut Scope,
                              function_name_token: &Token,
                              call: &Call)
    -> Result<(String, CompiledFunction), Box<dyn Error>>
{
    let function_name = function_name_token.content();
    let possible_functions = scope.lookup_function_descriptions(function_name);
    for function_description in possible_functions
    {
        if function_description.params.len() != call.arguments.len() {
            continue;
        }
        
        let param_arguements = function_description.params.iter().zip(&call.arguments);
        let mut did_match = true;
        for (param, argument) in param_arguements
        {
            let argument_type = derive_data_type(scope, argument)?;
            if !param.matches(&argument_type)
            {
                did_match = false;
                break;
            }
        }

        if did_match
        {
            return Ok(function_from_description(
                scope, function_description, function_name, call)?);
        }
    }

    let extern_or_none = scope.lookup_extern(function_name);
    if extern_or_none.is_none()
    {
        return Err(CompilerError::new(function_name_token, format!(
            "Could not find function '{}'", function_name)));
    }

    Ok((function_name.to_owned(), CompiledFunction
    {
        name: function_name.to_owned(),
        params: Vec::new(),
        return_type: None,
    }))
}

fn compile_params(gen: &mut IRGenorator,
                  scope: &mut Scope,
                  function: &Function,
                  param_types: Vec<DataType>)
    -> Result<Option<Rc<IRValue>>, Box<dyn Error>>
{
    let mut param_sizes = param_types
        .iter()
        .map(|param| size_of(scope, &param))
        .collect::<Result<Vec<_>, _>>()?;

    let return_size = match &function.return_type
    {
        Some(return_type) => size_of(scope, return_type)?,
        None => 0,
    };

    let is_big_return = return_size > 4;
    if is_big_return {
        param_sizes.push(return_size);
    }

    let signature = function_signature(function, &param_types);
    let mut params = gen.start_function(&signature, param_sizes.into_iter());
    let return_to = if is_big_return { params.pop() } else { None };

    let param_and_names = params.into_iter().zip(&function.params).zip(param_types);
    for ((value, param), data_type) in param_and_names
    {
        // TODO: Handle duplicate name error here
        let name = param.name.content();
        assert!(scope.put_value(name.to_owned(), value, data_type.clone()));
    }

    Ok(return_to)
}

pub fn compile_function(gen: &mut IRGenorator,
                        scope: &'_ mut Scope<'_>,
                        function: &Function,
                        param_types: Vec<DataType>)
    -> Result<HashSet<CompiledFunction>, Box<dyn Error>>
{
    if function.body.is_none() {
        return Ok(Default::default());
    }

    let mut local_scope = Scope::new(Some(scope));
    let return_to = compile_params(gen, &mut local_scope, function, param_types)?;
    let return_type = function.return_type.as_ref();

    let mut did_return = false;
    for statement in function.body.as_ref().unwrap()
    {
        match &statement
        {
            Statement::Return(_) => did_return = true,
            _ => {},
        }

        compile_statement(gen, &mut local_scope,
            statement, return_type, return_to.clone(), None)?;
    }

    if !did_return
    {
        let zero = gen.emit_int(0);
        gen.ret(zero, 4);
    }

    Ok(local_scope.used_functions())
}

