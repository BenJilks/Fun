
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataType
{
    Null,
    Int,
    Char,
    Bool,
    Struct(String),
    Array(Box<DataType>, usize),
    Ref(Box<DataType>),
    Generic(Box<DataType>, String),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum DataTypeDescription
{
    Exact(DataType),
    Any,
}

fn data_type_matches(expected: &DataType, data_type: &DataType, type_variable: &Option<&str>)
    -> (bool, Option<DataType>)
{
    match expected
    {
        DataType::Struct(expected_name) =>
        {
            if type_variable.is_some() &&
               type_variable.as_ref().unwrap() == expected_name
            {
                return (true, Some(data_type.clone()));
            }

            match data_type
            {
                DataType::Struct(name) => (expected_name == name, None),
                _ => (false, None),
            }
        },

        DataType::Array(expected_array_type, expected_size) =>
        {
            match data_type
            {
                DataType::Array(array_type, size) =>
                {
                    if size == expected_size {
                        data_type_matches(expected_array_type, array_type, type_variable)
                    } else {
                        (false, None)
                    }
                },
                _ => (false, None),
            }
        },

        DataType::Ref(expected_ref_type) =>
        {
            match data_type
            {
                DataType::Ref(ref_type) =>
                    data_type_matches(expected_ref_type, ref_type, type_variable),
                _ => (false, None),
            }
        },

        DataType::Generic(expected_generic_type, expected_name) =>
        {
            if type_variable.is_some() &&
               type_variable.as_ref().unwrap() == expected_name
            {
                return (true, Some(data_type.clone()));
            }

            match data_type
            {
                DataType::Generic(generic_type, name) =>
                {
                    let (matches, type_variable_value) = data_type_matches(
                        expected_generic_type, generic_type, type_variable);

                    if matches && name == expected_name {
                        (true, type_variable_value)
                    } else {
                        (false, None)
                    }
                },

                _ => (false, None),
            }
        },

        other => (other == data_type, None),
    }
}

impl DataTypeDescription
{

    pub fn matches(&self, data_type: &DataType, type_variable: &Option<&str>)
        -> (bool, Option<DataType>)
    {
        match self
        {
            DataTypeDescription::Exact(expected) =>
                data_type_matches(expected, data_type, type_variable),
            DataTypeDescription::Any => (true, None),
        }
    }

}

