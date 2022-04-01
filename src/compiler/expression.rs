use super::intermediate::IRGenorator;
use super::intermediate::value::IRValue;
use super::name_table::{Scope, TypedStructType};
use super::data_type::{size_of, derive_data_type};
use super::function::{find_function_for_call, create_local_scope};
use super::error::CompilerError;
use crate::tokenizer::Token;
use crate::ast::{Expression, Operation, OperationType, Call, InitializerList};
use crate::data_type::DataType;
use std::rc::Rc;
use std::collections::HashMap;
use std::error::Error;

fn compile_identifier(scope: &Scope,
                      name_token: &Token)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let name = name_token.content();
    let value_or_none = scope.lookup_value(name);
    if value_or_none.is_none() {
        panic!(); // TODO
    }

    let (value, _) = value_or_none.unwrap();
    Ok(value)
}

fn compile_initilizer_list(gen: &mut IRGenorator, scope: &mut Scope,
                           initilizer_list: &InitializerList)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let struct_or_none = match &initilizer_list.data_type
    {
        DataType::Struct(struct_name) =>
            scope.lookup_struct(&struct_name),

        DataType::Generic(argument, struct_name) =>
        {
            match scope.lookup_typed_struct(&struct_name)
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
    let compile_field = |gen: &mut IRGenorator, index: usize|
    {
        let (name, expression) = &initilizer_list.list[index];
        let field_or_none = struct_layout.get(name.content());
        assert!(field_or_none.is_some());

        let (field, _) = field_or_none.unwrap();
        let value = compile_expression(gen, scope, expression)?;
        Ok((field.clone(), value))
    };

    Ok(gen.emit_struct_data(struct_size, field_count, compile_field)?)
}

fn compile_array_literal(gen: &mut IRGenorator, scope: &mut Scope,
                         array: &Vec<Expression>)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    assert!(array.len() > 0);
    let item_type = derive_data_type(scope, &array[0])?;
    let item_size = size_of(scope, &item_type)?;
    let item_count = array.len();
    let compile_item = move |gen: &mut IRGenorator, index: usize|
    {
        let item = &array[index];
        Ok(compile_expression(gen, scope, item)?)
    };

    Ok(gen.emit_array_literal(item_count, compile_item, item_size)?)
}

fn compile_add(gen: &mut IRGenorator, scope: &mut Scope,
               lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    Ok(gen.add(lhs_value, rhs_value))
}

fn compile_subtract(gen: &mut IRGenorator, scope: &mut Scope,
                    lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    Ok(gen.subtract(lhs_value, rhs_value))
}

fn compile_multiply(gen: &mut IRGenorator, scope: &mut Scope,
                    lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    Ok(gen.mul(lhs_value, rhs_value))
}

fn compile_greater_than(gen: &mut IRGenorator, scope: &mut Scope,
                        lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    Ok(gen.greater_than(lhs_value, rhs_value))
}

fn compile_less_than(gen: &mut IRGenorator, scope: &mut Scope,
                     lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    Ok(gen.less_than(lhs_value, rhs_value))
}

fn layout_for_typed_struct(gen: &mut IRGenorator,
                           scope: &Scope,
                           argument: &DataType,
                           typed_struct: TypedStructType)
    -> Result<HashMap<String, (Rc<IRValue>, DataType)>, Box<dyn Error>>
{
    let variable = &typed_struct.variable;
    let mut layout = HashMap::new();
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
        layout.insert(name.to_owned(), (value, data_type.clone()));
        last_offset += size as i32;
    }

    Ok(layout)
}

fn field_of(gen: &mut IRGenorator, scope: &mut Scope,
            struct_type: &DataType, field_name_token: &Token)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    match struct_type
    {
        DataType::Struct(struct_name) =>
        {
            // FIXME: We have duplicate code for this in data_type.rs
            let struct_or_none = scope.lookup_struct(&struct_name);
            if struct_or_none.is_none()
            {
                return Err(CompilerError::new(field_name_token, format!(
                    "Could not find struct '{}'", struct_name)));
            }

            let struct_ = struct_or_none.unwrap();
            let field_name = field_name_token.content();
            let field_or_none = struct_.get(field_name);
            if field_or_none.is_none()
            {
                return Err(CompilerError::new(field_name_token, format!(
                    "Could not find field '{}' in struct '{}'",
                    field_name, struct_name)));
            }

            let (field, _) = field_or_none.unwrap();
            Ok(field.clone())
        },

        DataType::Generic(argument, struct_name) =>
        {
            // FIXME: We have duplicate code for this in data_type.rs
            let struct_or_none = scope.lookup_typed_struct(&struct_name);
            if struct_or_none.is_none()
            {
                return Err(CompilerError::new(field_name_token, format!(
                    "Could not find struct '{}'", struct_name)));
            }

            let typed_struct = struct_or_none.unwrap();
            let layout = layout_for_typed_struct(gen, scope, &*argument, typed_struct)?;

            let field_name = field_name_token.content();
            let field_or_none = layout.get(field_name);
            if field_or_none.is_none()
            {
                return Err(CompilerError::new(field_name_token, format!(
                    "Could not find field '{}' in struct '{}'",
                    field_name, struct_name)));
            }

            let (field, _) = field_or_none.unwrap();
            Ok(field.clone())
        },

        _ => panic!(),
    }
}

fn compile_access(gen: &mut IRGenorator, scope: &mut Scope,
                  lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let field_name = match rhs
    {
        Expression::Identifier(field_name) => field_name,
        _ => panic!(),
    };

    let lhs_data_type = derive_data_type(scope, lhs)?;
    let lhs_value = compile_expression(gen, scope, lhs)?;
    match lhs_data_type
    {
        DataType::Struct(_) | DataType::Generic(_, _) =>
        {
            let field = field_of(gen, scope, &lhs_data_type, field_name)?;
            let lhs_ref = gen.ref_of(lhs_value);
            Ok(gen.access(lhs_ref, field))
        },

        DataType::Ref(ref_type) =>
        {
            let field = field_of(gen, scope, &*ref_type, field_name)?;
            Ok(gen.access(lhs_value, field))
        },

        _ => panic!(),
    }
}

fn compile_indexed(gen: &mut IRGenorator, scope: &mut Scope,
                   lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let lhs_value = compile_expression(gen, scope, lhs)?;
    let rhs_value = compile_expression(gen, scope, rhs)?;
    
    let (lhs_ref, item_type) = match derive_data_type(scope, &lhs)?
    {
        DataType::Array(item_type, _) =>
            (gen.ref_of(lhs_value), item_type),

        DataType::Ref(item_type) =>
            (lhs_value, item_type),

        _ => panic!(),
    };

    let item_size = size_of(scope, &item_type)?;
    let address =
        if item_size == 1
        {
            gen.add(lhs_ref, rhs_value)
        }
        else
        {
            let item_size_value = gen.emit_int(item_size as i32);
            let offset = gen.mul(rhs_value, item_size_value);
            gen.add(lhs_ref, offset)
        };

    Ok(gen.deref(address, item_size))
}

fn compile_ref(gen: &mut IRGenorator, scope: &mut Scope,
               lhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let value = compile_expression(gen, scope, lhs)?;
    Ok(gen.ref_of(value))
}

fn compile_deref(gen: &mut IRGenorator, scope: &mut Scope,
                 lhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let value = compile_expression(gen, scope, lhs)?;
    let data_type = derive_data_type(scope, lhs)?;
    let size = match data_type
    {
        DataType::Ref(ref_type) => size_of(scope, &*ref_type)?,
        _ => panic!(),
    };

    Ok(gen.deref(value, size))
}

fn compile_sizeof(gen: &mut IRGenorator, scope: &mut Scope,
                 lhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let data_type = derive_data_type(scope, lhs)?;
    let size = size_of(scope, &data_type)?;
    Ok(gen.emit_int(size as i32))
}

fn compile_assign(gen: &mut IRGenorator, scope: &mut Scope,
                  lhs: &Expression, rhs: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
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
    gen.mov(to, value);
    Ok(gen.emit_null())
}

fn compile_operation(gen: &mut IRGenorator, scope: &mut Scope,
                     operation: &Operation)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    match operation.operation_type
    {
        OperationType::Add => compile_add(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Subtract => compile_subtract(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Multiply => compile_multiply(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::GreaterThan => compile_greater_than(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::LessThan => compile_less_than(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Access => compile_access(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Indexed => compile_indexed(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
        OperationType::Ref => compile_ref(gen, scope, &operation.lhs),
        OperationType::Deref => compile_deref(gen, scope, &operation.lhs),
        OperationType::Sizeof => compile_sizeof(gen, scope, &operation.lhs),
        OperationType::Assign => compile_assign(gen, scope, &operation.lhs, &operation.rhs.as_ref().unwrap()),
    }
}

fn compile_call(gen: &mut IRGenorator, scope: &mut Scope, call: &Call)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let function_name_token =
        match call.callable.as_ref()
        {
            Expression::Identifier(function_name_token) => function_name_token,
            _ => panic!(),
        };

    let (signature, function) = find_function_for_call(scope, function_name_token, call)?;
    let mut local_scope = create_local_scope(scope, &function);
    let return_size = match &function.return_type
    {
        Some(return_type) => size_of(&mut local_scope, return_type)?,
        None => 0,
    };

    let argument_count = call.arguments.len();
    let compile_argument = |gen: &mut IRGenorator, index: usize| -> Result<_, Box<dyn Error>>
    {
        let argument_expression = &call.arguments[index];
        let data_type = derive_data_type(scope, argument_expression)?;
        let value = compile_expression(gen, scope, argument_expression)?;
        let size = size_of(scope, &data_type)?;
        Ok((value, size))
    };

    gen.call(&signature, argument_count, compile_argument, return_size)
}

fn compile_extern_call(gen: &mut IRGenorator, scope: &mut Scope, call: &Call)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    let function_name_token =
        match call.callable.as_ref()
        {
            Expression::Identifier(function_name_token) => function_name_token,
            _ => panic!(),
        };

    let function_name = function_name_token.content();
    let extern_or_none = scope.lookup_extern(function_name);
    if extern_or_none.is_none()
    {
        return Err(CompilerError::new(function_name_token, format!(
            "Could not find external function '{}'", function_name)));
    }

    let return_size = match &call.type_variable
    {
        Some(return_type) => size_of(scope, return_type)?,
        None => 0,
    };

    let argument_count = call.arguments.len();
    let compile_argument = |gen: &mut IRGenorator, index: usize| -> Result<_, Box<dyn Error>>
    {
        let argument_expression = &call.arguments[index];
        let data_type = derive_data_type(scope, argument_expression)?;
        let value = compile_expression(gen, scope, argument_expression)?;
        let size = size_of(scope, &data_type)?;
        Ok((value, size))
    };

    gen.call(function_name, argument_count, compile_argument, return_size)
}

pub fn compile_expression(gen: &mut IRGenorator, scope: &mut Scope,
                          expression: &Expression)
    -> Result<Rc<IRValue>, Box<dyn Error>>
{
    match expression
    {
        Expression::IntLiteral(i) => Ok(gen.emit_int(*i)),
        Expression::BoolLiteral(b) => Ok(gen.emit_char(if *b { 1u8 } else { 0u8 } as char)),
        Expression::StringLiteral(s) => Ok(gen.emit_string(s.content())),
        Expression::CharLiteral(c) => Ok(gen.emit_char(c.content().chars().nth(0).unwrap())),
        Expression::Identifier(name) => compile_identifier(scope, name),
        Expression::InitializerList(list) => compile_initilizer_list(gen, scope, list),
        Expression::ArrayLiteral(array) => compile_array_literal(gen, scope, array),
        Expression::Operation(operation) => compile_operation(gen, scope, operation),
        Expression::Call(call) => compile_call(gen, scope, call),
        Expression::ExternCall(call) => compile_extern_call(gen, scope, call),
    }
}

