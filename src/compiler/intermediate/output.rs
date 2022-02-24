use super::value::{IRLocation, Allocator};
use crate::intermediate::{IR, IRProgram, IRFunction};
use crate::intermediate::IRStorage;
use std::rc::Rc;
use std::cell::RefCell;

pub struct IROutput
{
    pub program: Option<IRProgram>,
    pub current_function: Option<IRFunction>,
    pub allocator: Allocator,
}

impl IROutput
{

    pub fn new() -> Rc<RefCell<Self>>
    {
        Rc::from(RefCell::from(Self
        {
            program: Some(IRProgram::new()),
            current_function: None,
            allocator: Allocator::new(),
        }))
    }

    pub fn emit_ir(&mut self, instruction: IR)
    {
        match &mut self.current_function
        {
            Some(function) => function.code.push(instruction),
            None => panic!(),
        }
    }

    pub fn free_location(&mut self, location: &IRLocation)
    {
        match location
        {
            IRLocation::Storage(storage, _) =>
            {
                match storage
                {
                    IRStorage::Register(register) =>
                    {
                        self.emit_ir(IR::FreeRegister(*register));
                        self.allocator.free(*register);
                    },

                    _ => {},
                }
            },

            _ => {},
        }
    }

    pub fn add_function(&mut self, function: IRFunction)
    {
        let program = self.program.as_mut().unwrap();
        program.functions.push(function);
    }

    pub fn add_extern(&mut self, extern_: String)
    {
        let program = self.program.as_mut().unwrap();
        program.externs.push(extern_);
    }

}


