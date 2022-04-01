use super::intermediate::value::IRValue;
use crate::data_type::{DataType, DataTypeDescription};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FunctionDescriptionType
{
    pub params: Vec<DataTypeDescription>,
    pub type_variable: Option<DataType>,
    pub return_type: Option<DataType>,
}

#[derive(Clone)]
pub struct TypedStructType
{
    pub variable: String,
    pub fields: Vec<(String, DataType)>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct CompiledFunction
{
    pub name: String,
    pub description: FunctionDescriptionType,
    pub params: Vec<DataType>,
    pub type_variable: Option<DataType>,
    pub return_type: Option<DataType>,
}

pub struct Scope<'a>
{
    parent: Option<&'a Scope<'a>>,
    values: HashMap<String, (Rc<IRValue>, DataType)>,
    structs: HashMap<String, HashMap<String, (Rc<IRValue>, DataType)>>,
    typed_structs: HashMap<String, TypedStructType>,
    function_descriptions: HashMap<String, Vec<FunctionDescriptionType>>,
    type_aliases: HashMap<String, DataType>,

    // FIXME: This is hacky, externs should be no different from a 
    //        normal, compiled function.
    externs: HashMap<String, ()>,

    used_functions: HashSet<CompiledFunction>,
}

impl<'a> Scope<'a>
{

    pub fn new(parent: Option<&'a Scope>) -> Self
    {
        Self
        {
            parent,
            values: Default::default(),
            structs: Default::default(),
            typed_structs: Default::default(),
            function_descriptions: Default::default(),
            type_aliases: Default::default(),

            externs: Default::default(),
            used_functions: HashSet::default(),
        }
    }

    pub fn put_used_function(&mut self, function: CompiledFunction)
    {
        self.used_functions.insert(function);
    }
    pub fn used_functions(&self) -> HashSet<CompiledFunction>
    {
        self.used_functions.clone()
    }

    pub fn put_value(&mut self, name: String, value: Rc<IRValue>, data_type: DataType) -> bool
    {
        self.values.insert(name, (value, data_type)).is_none()
    }
    pub fn put_struct(&mut self, name: String, value: HashMap<String, (Rc<IRValue>, DataType)>) -> bool
    {
        self.structs.insert(name, value).is_none()
    }
    pub fn put_typed_struct(&mut self, name: String, value: TypedStructType) -> bool
    {
        self.typed_structs.insert(name, value).is_none()
    }
    pub fn put_function_description(&mut self, name: String, value: FunctionDescriptionType)
    {
        match self.function_descriptions.get_mut(&name)
        {
            Some(descriptions) => descriptions.push(value),
            None => { self.function_descriptions.insert(name, vec![value]); },
        }
    }
    pub fn put_type_alias(&mut self, name: String, value: DataType) -> bool
    {
        self.type_aliases.insert(name, value).is_none()
    }
    pub fn put_extern(&mut self, name: String) -> bool
    {
        self.externs.insert(name, ()).is_none()
    }

    fn lookup<T, F>(&self, name: &str, get: F) -> Option<T>
        where F: Fn(&Self, &str) -> Option<T>, T: Clone
    {
        let value = get(self, name);
        if value.is_some() {
            return value;
        }

        match self.parent
        {
            Some(parent) => parent.lookup(name, get),
            None => None,
        }
    }

    pub fn lookup_value(&self, name: &str) -> Option<(Rc<IRValue>, DataType)>
    {
        self.lookup(name, |s, n| s.values.get(n).cloned())
    }
    pub fn lookup_struct(&self, name: &str) -> Option<HashMap<String, (Rc<IRValue>, DataType)>>
    {
        self.lookup(name, |s, n| s.structs.get(n).cloned())
    }
    pub fn lookup_typed_struct(&self, name: &str) -> Option<TypedStructType>
    {
        self.lookup(name, |s, n| s.typed_structs.get(n).cloned())
    }
    pub fn lookup_function_descriptions(&self, name: &str) -> Vec<FunctionDescriptionType>
    {
        self.lookup(name, |s, n| s.function_descriptions.get(n).cloned())
            .unwrap_or(Vec::new())
    }
    pub fn lookup_type_alias(&self, name: &str) -> Option<DataType>
    {
        self.lookup(name, |s, n| s.type_aliases.get(n).cloned())
    }
    pub fn lookup_extern(&self, name: &str) -> Option<()>
    {
        self.lookup(name, |s, n| s.externs.get(n).cloned())
    }

}

