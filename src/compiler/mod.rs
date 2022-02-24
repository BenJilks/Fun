mod intermediate;
mod name_table;
mod data_type;
mod error;
mod function;
mod expression;
mod statement;
use intermediate::IRGenorator;
use name_table::{Scope, CompiledFunction, FunctionDescriptionType, TypedStructType};
use data_type::size_of;
use function::compile_function;
use crate::ast::SourceFile;
use crate::ast::{Function, Struct};
use crate::intermediate::IRProgram;
use std::collections::{HashMap, HashSet};
use std::error::Error;

fn register_typed_struct(scope: &mut Scope,
                         struct_: &Struct)
    -> Result<(), Box<dyn Error>>
{
    let mut fields = Vec::new();
    for field in &struct_.fields
    {
        let name = field.name.content();
        let data_type = field.data_type.clone();
        fields.push((name.to_owned(), data_type));
    }

    let variable_token = struct_.type_variable.clone().unwrap();
    let name = struct_.name.content();
    scope.put_typed_struct(name.to_owned(), TypedStructType
    {
        variable: variable_token.content().to_owned(),
        fields,
    });
    Ok(())
}

fn register_struct(gen: &mut IRGenorator, scope: &mut Scope,
                               struct_: &Struct)
        -> Result<(), Box<dyn Error>>
{
    if struct_.type_variable.is_some() {
        return register_typed_struct(scope, struct_);
    }

    let mut last_offset = 0;
    let mut struct_layout = HashMap::new();
    for field in &struct_.fields
    {
        let name = field.name.content();
        let data_type = field.data_type.clone();
        let size = size_of(scope, &data_type)?;

        let value = gen.emit_struct_offset(last_offset, size);
        last_offset += size as i32;
        struct_layout.insert(name.to_owned(), (value, data_type));
    }

    let name = struct_.name.content();
    scope.put_struct(name.to_owned(), struct_layout);
    Ok(())
}

fn register_function(scope: &mut Scope,
                            function: &Function)
    -> Result<(), Box<dyn Error>>
{
    let return_type = function.return_type.clone();
    let params = function.params
        .iter()
        .map(|param| param.data_type_description.clone())
        .collect::<Vec<_>>();

    let name = function.name.content();
    let type_variable = function.type_variable.as_ref().map(|x| x.content().to_owned());
    scope.put_function_description(name.to_owned(), FunctionDescriptionType
    {
        params,
        type_variable,
        return_type,
    });
    Ok(())
}

fn register_extern(scope: &mut Scope,
                          name: String)
    -> Result<(), Box<dyn Error>>
{
    scope.put_extern(name);
    Ok(())
}

pub fn compile(ast: SourceFile)
    -> Result<IRProgram, Box<dyn Error>>
{
    let mut scope = Scope::new(None);
    for function in &ast.functions {
        register_function(&mut scope, function)?;
    }
    for extern_ in &ast.externs {
        register_extern(&mut scope, extern_.content().to_owned())?;
    }

    let mut compiled_functions = HashSet::<CompiledFunction>::new();
    let mut functions_to_compile = Vec::<CompiledFunction>::new();
    functions_to_compile.push(CompiledFunction
    {
        name: "main".to_owned(),
        description: FunctionDescriptionType
        {
            params: Vec::new(),
            type_variable: None,
            return_type: None,
        },
        params: Vec::new(),
        type_variable: None,
        return_type: None,
    });

    let mut gen = IRGenorator::new();
    for struct_ in &ast.structs {
        register_struct(&mut gen, &mut scope, struct_)?;
    }

    while functions_to_compile.len() > 0
    {
        let function_data = functions_to_compile.pop().unwrap();
        let function = ast.find_function(&function_data.name, &function_data.description.params);
        let functions_used = compile_function(
            &mut gen,
            &mut scope,
            function.unwrap(),
            &function_data)?;

        for function in functions_used
        {
            if compiled_functions.contains(&function) {
                continue;
            }
            compiled_functions.insert(function.clone());
            functions_to_compile.push(function);
        }
    }

    for extern_ in &ast.externs {
        gen.emit_extern(extern_.content());
    }

    Ok(gen.program())
}

