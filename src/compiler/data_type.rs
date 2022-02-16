use super::name_table::Scope;
use super::error::CompilerError;
use crate::tokenizer::Token;
use crate::ast::{Function, Expression};
use crate::ast::{Call, Operation, OperationType};
use crate::data_type::DataType;

fn derive_access_type<Value>(name_table: &mut Scope<Value>,
                             lhs_type: DataType,
                             field_name_token: &Token)
    -> Result<DataType, Box<CompilerError>>
{
    match lhs_type
    {
        DataType::Struct(struct_name) =>
        {
            let struct_or_none = name_table.structs().lookup(&struct_name);
            if struct_or_none.is_none()
            {
                return Err(CompilerError::new_no_position(format!(
                    "Could not find struct '{}'", struct_name)));
            }

            let struct_ = struct_or_none.unwrap();
            let field_name = field_name_token.content();
            let field_or_none = struct_.lookup(field_name);
            if field_or_none.is_none()
            {
                return Err(CompilerError::new_no_position(format!(
                    "Could not find field '{}' in struct '{}'",
                    field_name, struct_name)));
            }

            let (_, data_type) = field_or_none.unwrap();
            Ok(data_type)
        },

        DataType::Generic(argument, struct_name) =>
        {
            let struct_or_none = name_table.typed_structs().lookup(&struct_name);
            if struct_or_none.is_none()
            {
                return Err(CompilerError::new_no_position(format!(
                    "Could not find struct '{}'", struct_name)));
            }

            let typed_struct = struct_or_none.unwrap();
            let field_name = field_name_token.content();
            let field_or_none = typed_struct.fields.iter().find(|(x, _)| x == field_name);
            if field_or_none.is_none()
            {
                return Err(CompilerError::new_no_position(format!(
                    "Could not find field '{}' in struct '{}'",
                    field_name, struct_name)));
            }

            let (_, data_type) = field_or_none.unwrap();
            let is_varable_type = match &data_type
            {
                DataType::Struct(name) => name == &typed_struct.variable,
                _ => false,
            };

            if is_varable_type {
                Ok(*argument)
            } else {
                Ok(data_type.clone())
            }
        },

        _ => panic!(),
    }
}

fn derive_operation_type<Value>(name_table: &mut Scope<Value>,
                                operation: &Operation)
    -> Result<DataType, Box<CompilerError>>
{
    let lhs_type = derive_data_type(name_table, &operation.lhs)?;
    match operation.operation_type
    {
        OperationType::Access =>
        {
            let rhs = operation.rhs.as_ref().unwrap();
            match rhs.as_ref()
            {
                Expression::Identifier(field_name) =>
                    derive_access_type(name_table, lhs_type, field_name),

                _ => panic!(),
            }
        },

        OperationType::Indexed =>
        {
            match lhs_type
            {
                DataType::Array(data_type, _) => Ok(*data_type),
                _ => panic!(),
            }
        },

        // TODO: This should actually derive type.
        OperationType::Add => Ok(DataType::Int),
        OperationType::Subtract => Ok(DataType::Int),
        OperationType::GreaterThan => Ok(DataType::Bool),
        OperationType::LessThan => Ok(DataType::Bool),

        OperationType::Ref => Ok(DataType::Ref(Box::from(lhs_type))),
        OperationType::Assign => Ok(DataType::Null),
    }
}

pub fn call_signature<Value>(scope: &mut Scope<Value>,
                         function_name: &str,
                         call: &Call)
    -> Result<String, Box<CompilerError>>
{
    let mut signature = function_name.to_owned();
    for argument in &call.arguments
    {
        let data_type = derive_data_type(scope, argument)?;
        signature += &data_type_signature(&data_type);
    }

    Ok(signature)
}

fn derive_call_type<Value>(scope: &mut Scope<Value>, call: &Call)
    -> Result<DataType, Box<CompilerError>>
{
    match call.callable.as_ref()
    {
        Expression::Identifier(name) =>
        {
            let signature = call_signature(scope, name.content(), call)?;
            let function_or_none = scope.functions().lookup(&signature);
            assert!(function_or_none.is_some());

            let function = function_or_none.unwrap();
            assert!(function.return_type.is_some());

            Ok(function.return_type.unwrap())
        },

        _ => panic!(),
    }
}

pub fn derive_data_type<Value>(scope: &mut Scope<Value>,
                               expression: &Expression)
    -> Result<DataType, Box<CompilerError>>
{
    match expression
    {
        Expression::IntLiteral(_) => Ok(DataType::Int),
        Expression::BoolLiteral(_) => Ok(DataType::Bool),
        Expression::StringLiteral(_) => Ok(DataType::Ref(Box::from(DataType::Char))),
        Expression::CharLiteral(_) => Ok(DataType::Char),

        Expression::Operation(operation) =>
            derive_operation_type(scope, operation),

        Expression::Call(call) =>
            derive_call_type(scope, call),

        Expression::Identifier(name) =>
        {
            match scope.values().lookup(name.content())
            {
                Some((_, data_type)) => Ok(data_type),
                None => 
                {
                    Err(CompilerError::new(name, format!(
                        "Could not find '{}'", name.content())))
                },
            }
        }

        Expression::InitializerList(initilizer_list) =>
            Ok(initilizer_list.data_type.clone()),

        Expression::ArrayLiteral(items) =>
        {
            assert!(items.len() > 0);

            let item_data_type = derive_data_type(scope, &items[0])?;
            Ok(DataType::Array(Box::from(item_data_type), items.len()))
        }
    }
}

fn size_of_struct<Value>(scope: &mut Scope<Value>, name: &str)
    -> Result<usize, Box<CompilerError>>
{
    let struct_of_none = scope.structs().lookup(name);
    if struct_of_none.is_none()
    {
        return Err(CompilerError::new_no_position(format!(
            "Could not find struct '{}'", name)));
    }

    let struct_ = struct_of_none.unwrap();
    let mut total_size = 0;
    for (_, (_, data_type)) in struct_.values() {
        total_size += size_of(scope, data_type)?;
    }

    Ok(total_size)
}

fn size_of_typed_struct<Value>(scope: &mut Scope<Value>,
                               name: &str,
                               argument_type: &DataType)
    -> Result<usize, Box<CompilerError>>
{
    let typed_struct_of_none = scope.typed_structs().lookup(name);
    if typed_struct_of_none.is_none()
    {
        return Err(CompilerError::new_no_position(format!(
            "Could not find struct '{}'", name)));
    }

    let typed_struct = typed_struct_of_none.unwrap();
    let mut total_size = 0;
    for (_, data_type) in typed_struct.fields
    {
        let is_varable_type = match &data_type
        {
            DataType::Struct(name) => name == &typed_struct.variable,
            _ => false,
        };

        if is_varable_type {
            total_size += size_of(scope, argument_type)?;
        } else {
            total_size += size_of(scope, &data_type)?;
        }
    }

    Ok(total_size)
}

pub fn size_of<Value>(scope: &mut Scope<Value>, data_type: &DataType)
    -> Result<usize, Box<CompilerError>>
{
    Ok(match data_type
    {
        DataType::Null => 0,
        DataType::Int => 4,
        DataType::Char => 1,
        DataType::Bool => 1,
        DataType::Ref(_) => 4,

        DataType::Struct(name) =>
            size_of_struct(scope, name)?,

        DataType::Array(item_type, size) => 
            size_of(scope, item_type)? * size,

        DataType::Generic(argument, name) =>
            size_of_typed_struct(scope, name, &*argument)?,
    })
}

fn data_type_signature(data_type: &DataType)
    -> String
{
    match data_type
    {
        DataType::Null => "null".to_owned(),
        DataType::Int => "int".to_owned(),
        DataType::Char => "char".to_owned(),
        DataType::Bool => "bool".to_owned(),
        DataType::Struct(token) => token.to_owned(),

        DataType::Array(data_type, size) =>
            format!("{}{}", data_type_signature(data_type), size),

        DataType::Ref(data_type) =>
            format!("ref{}", data_type_signature(data_type)),

        DataType::Generic(argument, token) =>
            format!("{}of{}", data_type_signature(argument), token),
    }
}

pub fn function_signature(function: &Function)
    -> String
{
    let mut signature = function.name.content().to_owned();
    for param in &function.params {
        signature += &data_type_signature(&param.data_type);
    }

    signature
}
