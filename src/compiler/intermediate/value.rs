use super::output::IROutput;
use crate::intermediate::{IRRegister, IRStorage};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

pub enum IRLocation
{
    Null,
    I32(i32),
    I8(i8),
    String(String),
    Field(usize, usize),
    Storage(IRStorage, usize),
}

pub struct IRValue
{
    pub location: IRLocation,
    pub output: Rc<RefCell<IROutput>>,
}

pub struct Allocator
{
    registers_in_use: HashSet<IRRegister>,
}

impl IRValue
{
    
    pub fn storage(&self) -> IRStorage
    {
        match &self.location
        {
            IRLocation::Storage(storage, _) => storage.clone(),
            _ => panic!(),
        }
    }

}

impl Drop for IRValue
{

    fn drop(&mut self)
    {
        let mut output = self.output.borrow_mut();
        output.free_location(&self.location);
    }

}

impl Allocator
{

    pub fn new() -> Self
    {
        Self
        {
            registers_in_use: HashSet::new(),
        }
    }

    pub fn allocate(&mut self) -> IRRegister
    {
        let mut register = 0;
        while self.registers_in_use.contains(&register) {
            register += 1;
        }

        self.registers_in_use.insert(register);
        register
    }

    pub fn free(&mut self, register: IRRegister)
    {
        assert!(self.registers_in_use.remove(&register))
    }

}

