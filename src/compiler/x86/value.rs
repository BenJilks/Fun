use super::register::{X86Register, RegisterAllocator};
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::io::Write;

pub enum X86StorageLocation
{
    Null,
    Register(X86Register),
    Deref(X86Register, usize),
    Local(i32, usize),
    Stack(usize, usize),
    Constant(String),
    I32(i32),
    I8(i8),
    StructData(Vec<(i32, Rc<X86Value>, usize)>),
    ArrayData(Vec<Rc<X86Value>>, usize),
}

impl fmt::Display for X86StorageLocation
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            X86StorageLocation::Register(register) => write!(f, "{}", register),
            X86StorageLocation::Deref(register, size) => write!(f, "{}", register.offset(*size, 0)),
            X86StorageLocation::Constant(name) => write!(f, "{}", name),
            X86StorageLocation::I32(i) => write!(f, "{}", i),
            X86StorageLocation::I8(i) => write!(f, "{}", i),

            X86StorageLocation::Local(ebp_offset, size) =>
                write!(f, "{}", X86Register::ebp().offset(*size, *ebp_offset)),

            X86StorageLocation::Stack(esp_offset, size) =>
                write!(f, "{}", X86Register::esp().offset(*size, *esp_offset as i32)),
            
            // TODO: Proper error here.
            X86StorageLocation::Null => panic!(),
            X86StorageLocation::StructData(_) => panic!(),
            X86StorageLocation::ArrayData(_, _) => panic!(),
        }
    }
}

pub struct X86Value
{
    pub location: X86StorageLocation,
    pub allocator: Rc<RefCell<RegisterAllocator>>,
    pub output: Rc<RefCell<dyn Write>>,
}

impl Drop for X86Value
{

    fn drop(&mut self)
    {
        match &self.location
        {
            X86StorageLocation::Register(register) => 
                self.allocator.borrow_mut().free(register.clone()),

            X86StorageLocation::Deref(register, _) => 
                self.allocator.borrow_mut().free(register.clone()),

            X86StorageLocation::Stack(position, size) =>
            {
                self.allocator.borrow_mut().poped_value_on_stack(*position, *size);
                writeln!(self.output.borrow_mut(), "; Free stack value").unwrap();
                writeln!(self.output.borrow_mut(), "add esp, {}", size).unwrap();
            },

            X86StorageLocation::Null => {},
            X86StorageLocation::Local(_, _) => {},
            X86StorageLocation::Constant(_) => {},
            X86StorageLocation::I32(_) => {},
            X86StorageLocation::I8(_) => {},
            X86StorageLocation::StructData(_) => {},
            X86StorageLocation::ArrayData(_, _) => {},
        }
    }

}

