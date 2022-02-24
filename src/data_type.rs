
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

impl DataTypeDescription
{

    pub fn matches(&self, data_type: &DataType) -> bool
    {
        match self
        {
            DataTypeDescription::Exact(expected) => expected == data_type,
            DataTypeDescription::Any => true,
        }
    }

}

