
#[derive(Debug, Clone, PartialEq)]
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

