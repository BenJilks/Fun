use super::memory::{X86Register, Allocator};
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
    StackValue(usize, usize),
    StackReference(usize, usize, usize),
    Constant(String),
    I32(i32),
    I8(i8),
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

            // FIXME: Check this is top of stack still?
            X86StorageLocation::StackValue(_, size) =>
                write!(f, "{}", X86Register::esp().offset(*size, 0)),
            X86StorageLocation::StackReference(_, esp_offset, size) =>
                write!(f, "{}", X86Register::esp().offset(*size, *esp_offset as i32)),
            
            // TODO: Proper error here.
            X86StorageLocation::Null => panic!(),
        }
    }
}

pub struct X86Value
{
    pub location: X86StorageLocation,
    pub allocator: Rc<RefCell<Allocator>>,
    pub output: Rc<RefCell<Option<Vec<u8>>>>,
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

            X86StorageLocation::StackValue(position, size) =>
            {
                self.allocator.borrow_mut().poped_value_on_stack(*position, *size);
                match &mut *self.output.borrow_mut()
                {
                    Some(output) =>
                    {
                        writeln!(output, "; Free stack value").unwrap();
                        writeln!(output, "add esp, {}", size).unwrap();
                    },
                    None => {},
                }
            },

            X86StorageLocation::Null => {},
            X86StorageLocation::StackReference(_, _, _) => {},
            X86StorageLocation::Local(_, _) => {},
            X86StorageLocation::Constant(_) => {},
            X86StorageLocation::I32(_) => {},
            X86StorageLocation::I8(_) => {},
        }
    }

}

