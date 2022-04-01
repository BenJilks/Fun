use super::name_table::Scope;
use super::function::{find_function_for_call, create_local_scope};
use super::error::CompilerError;
use crate::tokenizer::Token;
use crate::ast::{Function, Expression};
use crate::ast::{Call, Operation, OperationType};
use crate::data_type::DataType;
use std::error::Error;

fn derive_access_type(name_table: &mut Scope,
                      lhs_type: DataType,
                      field_name_token: &Token)
    -> Result<DataType, Box<dyn Error>>
{
    match lhs_type
    {
        DataType::Struct(struct_name) =>
        {
            let struct_or_none = name_table.lookup_struct(&struct_name);
            if struct_or_none.is_none()
            {
                return Err(CompilerError::new_no_position(format!(
                    "Could not find struct '{}'", struct_name)));
            }

            let struct_ = struct_or_none.unwrap();
            let field_name = field_name_token.content();
            let field_or_none = struct_.get(field_name);
            if field_or_none.is_none()
            {
                return Err(CompilerError::new_no_position(format!(
                    "Could not find field '{}' in struct '{}'",
                    field_name, struct_name)));
            }

            let (_, data_type) = field_or_none.unwrap();
            Ok(data_type.clone())
        },

        DataType::Generic(argument, struct_name) =>
        {
            let struct_or_none = name_table.lookup_typed_struct(&struct_name);
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

        DataType::Ref(ref_type) =>
            derive_access_type(name_table, *ref_type, field_name_token),

        _ => panic!(),
    }
}

fn derive_operation_type(name_table: &mut Scope,
                         operation: &Operation)
    -> Result<DataType, Box<dyn Error>>
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
                DataType::Ref(data_type) => Ok(*data_type),
                _ => panic!(),
            }
        },

        OperationType::Deref =>
        {
            match lhs_type
            {
                DataType::Ref(ref_type) => Ok(*ref_type),
                _ => panic!(),
            }
        },

        // TODO: This should actually derive type.
        OperationType::Add => Ok(DataType::Int),
        OperationType::Subtract => Ok(DataType::Int),
        OperationType::Multiply => Ok(DataType::Int),
        OperationType::GreaterThan => Ok(DataType::Bool),
        OperationType::LessThan => Ok(DataType::Bool),

        OperationType::Ref => Ok(DataType::Ref(Box::from(lhs_type))),
        OperationType::Sizeof => Ok(DataType::Int),
        OperationType::Assign => Ok(DataType::Null),
    }
}

pub fn call_signature(scope: &mut Scope,
                      function_name: &str,
                      call: &Call,
                      type_variable: Option<(&DataType, &DataType)>,
                      return_type: &Option<DataType>)
    -> Result<String, Box<dyn Error>>
{
    let mut local_scope = Scope::new(Some(scope));
    if let Some((name, value)) = type_variable 
    {
        if name != value
        {
            assert!(!doas_type_exist(&local_scope, name));
            local_scope.put_type_alias(
                type_variable_name(name).to_owned(),
                value.clone());
        }
    }

    let mut signature = function_name.to_owned() + "_";
    for argument in &call.arguments
    {
        let data_type = derive_data_type(&mut local_scope, argument)?;
        signature += &data_type_signature(&data_type);
    }

    match return_type
    {
        Some(return_type) =>
        {
            signature += &data_type_signature(
                &resolve_type_aliases(&mut local_scope, return_type.clone()))
        },
        None => {}
    }

    if signature == "main_" {
        Ok("main".to_owned())
    } else {
        Ok(signature)
    }
}

fn derive_call_type(scope: &mut Scope, call: &Call)
    -> Result<DataType, Box<dyn Error>>
{
    match call.callable.as_ref()
    {
        Expression::Identifier(name) =>
        {
            let (_, function) = find_function_for_call(scope, name, call)?;
            assert!(function.return_type.is_some());

            let mut local_scope = create_local_scope(scope, &function);
            Ok(resolve_type_aliases(&mut local_scope, function.return_type.unwrap()))
        },

        _ => panic!(),
    }
}

pub fn resolve_type_aliases(scope: &mut Scope,
                            data_type: DataType)
    -> DataType
{
    match data_type
    {
        DataType::Struct(name) =>
        {
            match scope.lookup_type_alias(&name)
            {
                Some(alias) => alias,
                None => DataType::Struct(name),
            }
        },

        DataType::Array(array_type, size) =>
            DataType::Array(Box::from(resolve_type_aliases(scope, *array_type)), size),

        DataType::Ref(ref_type) =>
            DataType::Ref(Box::from(resolve_type_aliases(scope, *ref_type))),

        DataType::Generic(generic_type, name) =>
            DataType::Generic(Box::from(resolve_type_aliases(scope, *generic_type)), name),

        other => other,
    }
}

pub fn derive_data_type(scope: &mut Scope,
                        expression: &Expression)
    -> Result<DataType, Box<dyn Error>>
{
    let result: Result<_, Box<dyn Error>> = match expression
    {
        Expression::IntLiteral(_) => Ok(DataType::Int),
        Expression::BoolLiteral(_) => Ok(DataType::Bool),
        Expression::StringLiteral(_) => Ok(DataType::Ref(Box::from(DataType::Char))),
        Expression::CharLiteral(_) => Ok(DataType::Char),

        Expression::Operation(operation) =>
            derive_operation_type(scope, operation),

        Expression::Call(call) =>
            derive_call_type(scope, call),

        Expression::ExternCall(call) =>
            Ok(call.type_variable.clone().unwrap()),

        Expression::Identifier(name) =>
        {
            match scope.lookup_value(name.content())
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
    };

    match result
    {
        Ok(result) => Ok(resolve_type_aliases(scope, result)),
        Err(err) => Err(err),
    }
}

fn size_of_struct(scope: &Scope, name: &str)
    -> Result<usize, Box<CompilerError>>
{
    let struct_of_none = scope.lookup_struct(name);
    if struct_of_none.is_none()
    {
        return Err(CompilerError::new_no_position(format!(
            "Could not find struct '{}'", name)));
    }

    let struct_ = struct_of_none.unwrap();
    let mut total_size = 0;
    for (_, (_, data_type)) in struct_ {
        total_size += size_of(scope, &data_type)?;
    }

    Ok(total_size)
}

fn size_of_typed_struct(scope: &Scope,
                        name: &str,
                        argument_type: &DataType)
    -> Result<usize, Box<CompilerError>>
{
    let typed_struct_of_none = scope.lookup_typed_struct(name);
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

pub fn size_of(scope: &Scope, data_type: &DataType)
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
        {
            match scope.lookup_type_alias(name)
            {
                Some(alias) => size_of(scope, &alias)?,
                None => size_of_struct(scope, name)?,
            }
        },

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

pub fn function_signature(function: &Function,
                          params: &Vec<DataType>,
                          return_type: &Option<DataType>)
    -> String
{
    let mut signature = function.name.content().to_owned() + "_";
    for param in params {
        signature += &data_type_signature(param);
    }

    match return_type
    {
        Some(return_type) =>
            signature += &data_type_signature(return_type),
        None => {},
    }

    if signature == "main_" {
        "main".to_owned()
    } else {
        signature
    }
}

pub fn doas_type_exist(scope: &Scope, data_type: &DataType) -> bool
{
    match data_type
    {
        DataType::Struct(name) => scope.lookup_struct(name).is_some(),
        DataType::Array(array_type, _) => doas_type_exist(scope, array_type),
        DataType::Ref(ref_type) => doas_type_exist(scope, ref_type),

        DataType::Generic(generic_type, name) =>
        {
            doas_type_exist(scope, generic_type) && 
                scope.lookup_typed_struct(name).is_some()
        },

        _ => true,
    }
}

pub fn type_variable_name(type_variable: &DataType) -> &str
{
    match type_variable
    {
        DataType::Struct(name) => name,
        _ => panic!(),
    }
}

