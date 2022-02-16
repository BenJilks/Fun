use super::code_genorator::CodeGenortator;
use super::name_table::{Scope, NameTable, FunctionType, TypedStructType};
use super::data_type::{size_of, derive_data_type, call_signature};
use super::error::CompilerError;
use crate::tokenizer::Token;
use crate::ast::{Expression, Operation, OperationType, Call, InitializerList};
use crate::data_type::DataType;
use std::rc::Rc;
use std::error::Error;

fn compile_identifier<Value>(scope: &mut Scope<Value>,
                             name_token: &Token)
    -> Result<Rc<Value>, Box<dyn Error>>
{
    let name = name_token.content();
    let value_or_none = scope.values().lookup(name);
    if value_or_none.is_none() {
        panic!(); // TODO
    }

    let (value, _) = value_or_none.unwrap();
    Ok(value)
}

fn compile_initilizer_list<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                                       initilizer_list: &InitializerList)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let struct_or_none = match &initilizer_list.data_type
    {
        DataType::Struct(struct_name) =>
            scope.structs().lookup(&struct_name),

        DataType::Generic(argument, struct_name) =>
        {
            match scope.typed_structs().lookup(&struct_name)
            {
                Some(typed_struct) =>
                    Some(layout_for_typed_struct(gen, scope, argument, typed_struct)?),
                None => None,
            }
        },

        _ => panic!(),
    };
    assert!(struct_or_none.is_some());

    let struct_layout = struct_or_none.unwrap();
    let struct_size = size_of(scope, &initilizer_list.data_type)?;
    let field_count = initilizer_list.list.len();
    let compile_field = |gen: &mut Gen, index: usize|
    {
        let (name, expression) = &initilizer_list.list[index];
        let field_or_none = struct_layout.lookup(name.content());
        assert!(field_or_none.is_some());

        let (field, _) = field_or_none.unwrap();
        let value = compile_expression(gen, scope, expression)?;
        Ok((field, value))
    };

    Ok(gen.emit_struct_data(struct_size, field_count, compile_field)?)
}

fn compile_array_literal<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                                     array: &Vec<Expression>)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    assert!(array.len() > 0);
    let item_type = derive_data_type(scope, &array[0])?;
    let item_size = size_of(scope, &item_type)?;
    let item_count = array.len();
    let compile_item = move |gen: &mut Gen, index: usize|
    {
        let item = &array[index];
        Ok(compile_expression(gen, scope, item)?)
    };

    Ok(gen.emit_array_literal(item_count, compile_item, item_size)?)
}

fn compile_add<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                           lhs: &Expression, rhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    gen.add(lhs_value, rhs_value)
}

fn compile_subtract<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                           lhs: &Expression, rhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    gen.subtract(lhs_value, rhs_value)
}

fn compile_greater_than<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                                    lhs: &Expression, rhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    gen.greater_than(lhs_value, rhs_value)
}

fn compile_less_than<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                                 lhs: &Expression, rhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    gen.less_than(lhs_value, rhs_value)
}

fn layout_for_typed_struct<Gen, Value>(gen: &mut Gen,
                                       scope: &mut Scope<Value>,
                                       argument: &DataType,
                                       typed_struct: TypedStructType)
        -> Result<NameTable<(Rc<Value>, DataType)>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let variable = &typed_struct.variable;
    let mut layout = NameTable::new(None);
    let mut last_offset = 0;
    for (name, field) in &typed_struct.fields
    {
        let is_varable_type = match field
        {
            DataType::Struct(name) => name == variable,
            _ => false,
        };

        let data_type = if is_varable_type { argument } else { field };
        let size = size_of(scope, data_type)?;
        let value = gen.emit_struct_offset(last_offset, size);
        layout.put(name, (value, data_type.clone()));
        last_offset += size as i32;
    }

    Ok(layout)
}

fn compile_access<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                              lhs: &Expression, rhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let lhs_data_type = derive_data_type(scope, lhs)?;
    let lhs_value = compile_expression(gen, scope, lhs)?;
    match rhs
    {
        Expression::Identifier(name) =>
        {
            match lhs_data_type
            {
                DataType::Struct(struct_name) =>
                {
                    // FIXME: We have duplicate code for this in data_type.rs
                    let struct_or_none = scope.structs().lookup(&struct_name);
                    if struct_or_none.is_none()
                    {
                        return Err(CompilerError::new(name, format!(
                            "Could not find struct '{}'", struct_name)));
                    }

                    let struct_ = struct_or_none.unwrap();
                    let field_name = name.content();
                    let field_or_none = struct_.lookup(field_name);
                    if field_or_none.is_none()
                    {
                        return Err(CompilerError::new(name, format!(
                            "Could not find field '{}' in struct '{}'",
                            field_name, struct_name)));
                    }

                    let (field, _) = field_or_none.unwrap();
                    gen.access(lhs_value, field)
                },

                DataType::Generic(argument, struct_name) =>
                {
                    // FIXME: We have duplicate code for this in data_type.rs
                    let struct_or_none = scope.typed_structs().lookup(&struct_name);
                    if struct_or_none.is_none()
                    {
                        return Err(CompilerError::new(name, format!(
                            "Could not find struct '{}'", struct_name)));
                    }

                    let typed_struct = struct_or_none.unwrap();
                    let layout = layout_for_typed_struct(gen, scope, &*argument, typed_struct)?;

                    let field_name = name.content();
                    let field_or_none = layout.lookup(field_name);
                    if field_or_none.is_none()
                    {
                        return Err(CompilerError::new(name, format!(
                            "Could not find field '{}' in struct '{}'",
                            field_name, struct_name)));
                    }

                    let (field, _) = field_or_none.unwrap();
                    gen.access(lhs_value, field)
                },

                _ => panic!(),
            }
        },

        _ => panic!(),
    }
}

fn compile_indexed<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                               lhs: &Expression, rhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let item_type = match derive_data_type(scope, &lhs)?
    {
        DataType::Array(item_type, _) => item_type,
        _ => panic!(),
    };
    let item_size = size_of(scope, &item_type)?;

    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    let lhs_ref = gen.ref_of(lhs_value)?;
    let address =
        if item_size == 1
        {
            gen.add(lhs_ref, rhs_value)?
        }
        else
        {
            let item_size_value = gen.emit_int(item_size as i32)?;
            let offset = gen.mul(rhs_value, item_size_value)?;
            gen.add(lhs_ref, offset)?
        };

    Ok(gen.deref(address, item_size)?)
}

fn compile_ref<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                           lhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let value = compile_expression(gen, scope, lhs)?;
    gen.ref_of(value)
}

fn compile_assign<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                              lhs: &Expression, rhs: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let to_data_type = derive_data_type(scope, lhs)?;
    let value_data_type = derive_data_type(scope, rhs)?;
    if to_data_type != value_data_type
    {
        // FIXME: What happens if we don't have a token?
        return Err(CompilerError::new(rhs.token().unwrap(), format!(
            "Can not assign value of type '{:?}' to type '{:?}'",
            value_data_type, to_data_type)));
    }

    let to = compile_expression(gen, scope, lhs)?;
    let value = compile_expression(gen, scope, rhs)?;

    let size = size_of(scope, &value_data_type)?;
    gen.mov(to, value, size)?;
    Ok(gen.emit_null())
}

fn compile_operation<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                                 operation: &Operation)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    match operation.operation_type
    {
        OperationType::Add => compile_add(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Subtract => compile_subtract(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::GreaterThan => compile_greater_than(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::LessThan => compile_less_than(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Access => compile_access(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Indexed => compile_indexed(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Ref => compile_ref(gen, scope, &operation.lhs),
        OperationType::Assign => compile_assign(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
    }
}

fn find_function_for_call<Value>(scope: &mut Scope<Value>,
                                 function_name_token: &Token,
                                 call: &Call)
    -> Result<(String, FunctionType), Box<dyn Error>>
{
    let function_name = function_name_token.content();
    let signature = call_signature(scope, function_name, call)?;
    let function_or_none = scope.functions().lookup(&signature);
    if function_or_none.is_some() {
        return Ok((signature, function_or_none.unwrap()));
    }

    let extern_or_none = scope.functions().lookup(function_name);
    if extern_or_none.is_none()
    {
        return Err(CompilerError::new(function_name_token, format!(
            "Could not find function '{}'", function_name)));
    }

    Ok((function_name.to_owned(), extern_or_none.unwrap()))
}

fn compile_call<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>, call: &Call)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    let function_name_token = 
        match call.callable.as_ref()
        {
            Expression::Identifier(function_name_token) => function_name_token,
            _ => panic!(),
        };

    let function_name = function_name_token.content();
    let (signature, function) = find_function_for_call(scope, function_name_token, call)?;
    if !function.is_extern && function.params.len() != call.arguments.len()
    {
        return Err(CompilerError::new(function_name_token, format!(
            "Expected {} argument(s) to function '{}', got {} instead",
            function.params.len(), function_name, call.arguments.len())));
    }

    let return_size = match &function.return_type
    {
        Some(return_type) => size_of(scope, return_type)?,
        None => 0,
    };

    let argument_count = call.arguments.len();
    let compile_argument = |gen: &mut Gen, index: usize| -> Result<_, Box<dyn Error>>
    {
        let argument_expression = &call.arguments[index];
        let data_type = derive_data_type(scope, argument_expression)?;
        if !function.is_extern && index < function.params.len() && data_type != function.params[index]
        {
            let token = argument_expression.token().unwrap_or(function_name_token);
            return Err(CompilerError::new(token, format!(
                "Expected argument of type '{:?}', got '{:?}' instead",
                function.params[index], data_type)));
        }

        let value = compile_expression(gen, scope, argument_expression)?;
        let size = size_of(scope, &data_type)?;
        Ok((value, size))
    };

    gen.call(&signature, argument_count, compile_argument, return_size)
}

pub fn compile_expression<Gen, Value>(gen: &mut Gen, scope: &mut Scope<Value>,
                                      expression: &Expression)
        -> Result<Rc<Value>, Box<dyn Error>>
    where Gen: CodeGenortator<Value>
{
    match expression
    {
        Expression::IntLiteral(i) => gen.emit_int(*i),
        Expression::BoolLiteral(b) => gen.emit_char(if *b { 1u8 } else { 0u8 } as char),
        Expression::StringLiteral(s) => gen.emit_string(s.content()),
        Expression::CharLiteral(c) => gen.emit_char(c.content().chars().nth(0).unwrap()),
        Expression::Identifier(name) => compile_identifier(scope, name),
        Expression::InitializerList(list) => compile_initilizer_list(gen, scope, list),
        Expression::ArrayLiteral(array) => compile_array_literal(gen, scope, array),
        Expression::Operation(operation) => compile_operation(gen, scope, operation),
        Expression::Call(call) => compile_call(gen, scope, call),
    }
}

