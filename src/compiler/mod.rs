pub mod x86;
mod code_genorator;
mod name_table;
mod data_type;
mod error;
mod expression;
mod statement;
use code_genorator::CodeGenortator;
use name_table::{NameTable, Scope, FunctionType, TypedStructType};
use data_type::{size_of, function_signature};
use statement::compile_statement;
use crate::ast::SourceFile;
use crate::ast::{Function, Struct, Statement};
use std::rc::Rc;
use std::error::Error;

fn compile_params<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                              function: &Function)
        -> Result<Option<Rc<Value>>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let mut param_sizes = function.params
        .iter()
        .map(|param| size_of(scope, &param.data_type))
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

    let signature = function_signature(function);
    let mut params = gen.start_function(&signature, param_sizes.into_iter())?;
    let return_to = if is_big_return { params.pop() } else { None };

    let param_and_names = params.into_iter().zip(&function.params);
    for (value, param) in param_and_names
    {
        // TODO: Handle duplicate name error here
        let name = param.name.content();
        let data_type = param.data_type.clone();
        assert!(scope.values().put(name, (value, data_type)));
    }

    Ok(return_to)
}

fn compile_function<Gen, Value>(gen: &mut Gen, scope: Scope<Value>, function: &Function)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    if function.body.is_none() {
        return Ok(());
    }

    let mut local_scope = Scope::<Value>::new(Some(Box::from(scope)));
    let return_to = compile_params(gen, &mut local_scope, function)?;
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
        let zero = gen.emit_int(0)?;
        gen.ret(zero, 4, None)?;
    }
    Ok(())
}

fn register_typed_struct<Value>(scope: &mut Scope<Value>,
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
    scope.typed_structs().put(name, TypedStructType
    {
        variable: variable_token.content().to_owned(),
        fields,
    });
    Ok(())
}

fn register_struct<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                               struct_: &Struct)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    if struct_.type_variable.is_some() {
        return register_typed_struct(scope, struct_);
    }

    let mut last_offset = 0;
    let mut struct_layout = NameTable::new(None);
    for field in &struct_.fields
    {
        let name = field.name.content();
        let data_type = field.data_type.clone();
        let size = size_of(scope, &data_type)?;

        let value = gen.emit_struct_offset(last_offset, size);
        last_offset += size as i32;
        struct_layout.put(name, (value, data_type));
    }

    let name = struct_.name.content();
    scope.structs().put(name, struct_layout);
    Ok(())
}

fn register_function<Value>(scope: &mut Scope<Value>,
                            function: &Function)
    -> Result<(), Box<dyn Error>>
{
    let return_type = function.return_type.clone();
    let params = function.params
        .iter()
        .map(|param| param.data_type.clone())
        .collect::<Vec<_>>();

    let signature = function_signature(function);
    scope.functions().put(&signature, FunctionType
    {
        is_extern: false,
        params,
        return_type,
    });
    Ok(())
}

fn register_extern<Value>(scope: &mut Scope<Value>,
                          name: &str)
    -> Result<(), Box<dyn Error>>
{
    scope.functions().put(name, FunctionType
    {
        is_extern: true,
        params: Vec::new(),
        return_type: None,
    });
    Ok(())
}

pub fn compile<Gen, Value>(gen: &mut Gen, ast: SourceFile)
        -> Result<(), Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let mut scope = Scope::<Value>::new(None);
    for struct_ in &ast.structs {
        register_struct(gen, &mut scope, struct_)?;
    }
    for function in &ast.functions {
        register_function(&mut scope, function)?;
    }
    for extern_ in &ast.externs {
        register_extern(&mut scope, extern_.content())?;
    }

    for extern_ in &ast.externs {
        gen.emit_extern(extern_.content());
    }
    for function in &ast.functions {
        compile_function(gen, scope.clone(), function)?
    }
    Ok(())
}

