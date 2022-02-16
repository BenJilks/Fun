use crate::data_type::DataType;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct NameTable<Item>
{
    parent: Option<Box<NameTable<Item>>>,
    values: HashMap<String, Item>,
}

#[derive(Clone)]
pub struct FunctionType
{
    pub is_extern: bool,
    pub params: Vec<DataType>,
    pub return_type: Option<DataType>,
}

#[derive(Clone)]
pub struct TypedStructType
{
    pub variable: String,
    pub fields: Vec<(String, DataType)>,
}

pub struct Scope<Value>
{
    values: NameTable<(Rc<Value>, DataType)>,
    structs: NameTable<NameTable<(Rc<Value>, DataType)>>,
    typed_structs: NameTable<TypedStructType>,
    functions: NameTable<FunctionType>,
}

impl<Value> Clone for Scope<Value>
{

    fn clone(&self) -> Self
    {
        Self
        {
            values: self.values.clone(),
            structs: self.structs.clone(),
            typed_structs: self.typed_structs.clone(),
            functions: self.functions.clone(),
        }
    }

}

impl<Item> NameTable<Item>
    where Item: Clone
{

    pub fn new(parent: Option<Box<NameTable<Item>>>) -> Self
    {
        Self
        {
            parent: parent,
            values: HashMap::new(),
        }
    }

    pub fn values(&self) -> impl Iterator<Item = (&String, &Item)>
    {
        self.values.iter()
    }

    pub fn put(&mut self, name: &str, value: Item) -> bool
    {
        match self.values.insert(name.to_owned(), value)
        {
            None => true,
            Some(_) => false,
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Item>
    {
        let value = self.values.get(name);
        if value.is_some() {
            return Some(value.unwrap().clone());
        }

        match &self.parent
        {
            Some(parent) => parent.lookup(name),
            None => None,
        }
    }

}

impl<Value> Scope<Value>
{

    pub fn new(parent: Option<Box<Scope<Value>>>) -> Self
    {
        match parent
        {
            Some(parent) =>
                Self
                {
                    values: NameTable::new(Some(Box::from(parent.values))),
                    structs: NameTable::new(Some(Box::from(parent.structs))),
                    typed_structs: NameTable::new(Some(Box::from(parent.typed_structs))),
                    functions: NameTable::new(Some(Box::from(parent.functions))),
                },
            
            None =>
                Self
                {
                    values: NameTable::new(None),
                    structs: NameTable::new(None),
                    typed_structs: NameTable::new(None),
                    functions: NameTable::new(None),
                },
        }
    }

    pub fn values(&mut self) -> &mut NameTable<(Rc<Value>, DataType)>
    {
        &mut self.values
    }

    pub fn structs(&mut self) -> &mut NameTable<NameTable<(Rc<Value>, DataType)>>
    {
        &mut self.structs
    }

    pub fn typed_structs(&mut self) -> &mut NameTable<TypedStructType>
    {
        &mut self.typed_structs
    }

    pub fn functions(&mut self) -> &mut NameTable<FunctionType>
    {
        &mut self.functions
    }

}

