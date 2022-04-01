use super::intermediate::IRGenorator;
use super::intermediate::value::IRValue;
use super::statement::compile_statement;
use super::name_table::{FunctionDescriptionType, CompiledFunction};
use super::name_table::Scope;
use super::data_type::{size_of, derive_data_type, resolve_type_aliases};
use super::data_type::{function_signature, call_signature};
use super::data_type::{doas_type_exist, type_variable_name};
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
                             call: &Call,
                             type_variable: Option<DataType>)
    -> Result<(String, CompiledFunction), Box<dyn Error>>
{
    let params = call.arguments
        .iter()
        .map(|argument| derive_data_type(scope, argument))
        .collect::<Result<Vec<_>, _>>()?;

    let type_alias = description.type_variable.as_ref().zip(type_variable.as_ref());
    let return_type = description.return_type.clone();
    let signature = call_signature(scope, function_name, call, type_alias, &return_type)?;
    let function = CompiledFunction
    {
        name: function_name.to_owned(),
        description: description,
        params,
        type_variable,
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
        let mut type_variable_value = match &call.type_variable
        {
            Some(type_variable) => Some(resolve_type_aliases(scope, type_variable.clone())),
            None => None,
        };

        let type_variable_name = match &function_description.type_variable
        {
            Some(type_variable) =>
            {
                if doas_type_exist(scope, type_variable)
                {
                    if type_variable_value.is_none() || 
                       type_variable_value.as_ref().unwrap() != type_variable 
                    {
                        continue;
                    }

                    None
                }
                else
                {
                    Some(type_variable_name(type_variable))
                }
            },
            None => None,
        };

        let mut did_match = true;
        for (param, argument) in param_arguements
        {
            let argument_type = derive_data_type(scope, argument)?;
            let (matches, param_type_variable_value) = param.matches(
                &argument_type, &type_variable_name);

            if !matches ||
                (type_variable_value.is_some() &&
                param_type_variable_value.is_some() &&
                type_variable_value != param_type_variable_value)
            {
                did_match = false;
                break;
            }

            if param_type_variable_value.is_some() {
                type_variable_value = param_type_variable_value;
            }
        }

        if did_match
        {
            return Ok(function_from_description(
                scope, function_description, function_name, call,
                type_variable_value)?);
        }
    }

    return Err(CompilerError::new(function_name_token, format!(
        "Could not find function '{}'", function_name)));
}

pub fn create_local_scope<'a>(scope: &'a mut Scope, function: &CompiledFunction)
    -> Scope<'a>
{
    let mut local_scope = Scope::new(Some(scope));
    if function.type_variable.is_some()
    {
        let name = function.description.type_variable.clone().unwrap();
        let value = function.type_variable.clone().unwrap();
        if name != value
        {
            assert!(!doas_type_exist(scope, &name));
            local_scope.put_type_alias(type_variable_name(&name).to_owned(), value);
        }
    }

    local_scope
}

fn compile_params(gen: &mut IRGenorator,
                  scope: &mut Scope,
                  function: &Function,
                  param_types: &Vec<DataType>,
                  return_type: &Option<DataType>)
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

    let signature = function_signature(function, &param_types, return_type);
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
                        function_data: &CompiledFunction)
    -> Result<HashSet<CompiledFunction>, Box<dyn Error>>
{
    if function.body.is_none() {
        return Ok(Default::default());
    }

    let mut local_scope = create_local_scope(scope, function_data);
    let return_type = match &function.return_type
    {
        Some(return_type) =>
            Some(resolve_type_aliases(&mut local_scope, return_type.clone())),
        None => None,
    };
    let return_to = compile_params(gen, &mut local_scope,
        function, &function_data.params, &return_type)?;

    let mut did_return = false;
    for statement in function.body.as_ref().unwrap()
    {
        match &statement
        {
            Statement::Return(_) => did_return = true,
            _ => {},
        }

        compile_statement(gen, &mut local_scope,
            statement, return_type.as_ref(), return_to.clone(), None)?;
    }

    if !did_return
    {
        let zero = gen.emit_int(0);
        gen.ret(zero, 4);
    }

    Ok(local_scope.used_functions())
}

