use crate::tokenizer::Token;
use crate::data_type::{DataType, DataTypeDescription};

#[derive(Debug)]
pub struct Field
{
    pub name: Token,
    pub data_type: DataType,
}

#[derive(Debug)]
pub struct Struct
{
    pub name: Token,
    pub type_variable: Option<Token>,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperationType
{
    Add,
    Subtract,
    GreaterThan,
    LessThan,
    Ref,
    Indexed,
    Access,
    Assign,
}

#[derive(Debug, Clone)]
pub struct Operation
{
    pub operation_type: OperationType,
    pub lhs: Box<Expression>,
    pub rhs: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub struct Call
{
    pub callable: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct InitializerList
{
    pub data_type: DataType,
    pub list: Vec<(Token, Expression)>,
}

#[derive(Debug, Clone)]
pub enum Expression
{
    Operation(Operation),
    Call(Call),
    InitializerList(InitializerList),
    ArrayLiteral(Vec<Expression>),
    IntLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(Token),
    CharLiteral(Token),
    Identifier(Token),
}

#[derive(Debug)]
pub struct Let
{
    pub name: Token,
    pub value: Expression,
}

#[derive(Debug)]
pub struct If
{
    pub condition: Expression,
    pub block: Vec<Statement>,
    pub else_block: Option<Vec<Statement>>,
}

#[derive(Debug)]
pub enum Statement
{
    Expression(Expression),
    Return(Expression),
    Let(Let),
    If(If),
    Loop(Vec<Statement>),
    While(Expression, Vec<Statement>),
    Break,
}

#[derive(Debug)]
pub struct Param
{
    pub name: Token,
    pub data_type_description: DataTypeDescription,
}

#[derive(Debug)]
pub struct Function
{
    pub name: Token,
    pub params: Vec<Param>,
    pub return_type: Option<DataType>,
    pub body: Option<Vec<Statement>>,
}

#[derive(Debug)]
pub struct SourceFile
{
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
    pub externs: Vec<Token>,
}

impl Default for SourceFile
{
    fn default() -> Self
    {
        Self
        {
            functions: Vec::new(),
            structs: Vec::new(),
            externs: Vec::new(),
        }
    }
}

impl SourceFile
{

    pub fn find_function(&self,
                         name: &str,
                         params: &Vec<DataTypeDescription>)
        -> Option<&Function>
    {
        self.functions
            .iter()
            .find(|f|
        {
            if f.name.content() != name {
                return false;
            }
            if f.params.len() != params.len() {
                return false;
            }
            for (param, expected) in f.params.iter().zip(params)
            {
                if &param.data_type_description != expected {
                    return false;
                }
            }
            true
        })
    }

}

impl Expression
{

    pub fn token(&self) -> Option<&Token>
    {
        match self
        {
            Self::Operation(operation) => operation.lhs.token(),
            Self::Call(call) => call.callable.token(),
            Self::InitializerList(list) => Some(&list.list.get(0)?.0),
            Self::ArrayLiteral(arr) => arr.get(0)?.token(),
            Self::IntLiteral(_) => None,
            Self::BoolLiteral(_) => None,
            Self::StringLiteral(token) => Some(token),
            Self::CharLiteral(token) => Some(token),
            Self::Identifier(token) => Some(token),
        }
    }

}

