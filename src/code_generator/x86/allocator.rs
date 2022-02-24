use super::register::X86Register;
use crate::intermediate::IRRegister;
use std::collections::{HashSet, HashMap};

pub struct Allocator
{
    registers_in_use: HashSet<char>,
    ir_to_x86: HashMap<IRRegister, X86Register>,
    ir_to_stack_offset: HashMap<IRRegister, (usize, usize)>,
    stack_size: usize,
}

#[derive(PartialEq, Debug)]
pub enum AllocationType
{
    Register,
    Stack,
}

impl Allocator
{

    pub fn new() -> Self
    {
        Self
        {
            registers_in_use: HashSet::new(),
            ir_to_x86: HashMap::new(),
            ir_to_stack_offset: HashMap::new(),
            stack_size: 0,
        }
    }

    pub fn is_in_use(&mut self, letter: char) -> bool
    {
        self.registers_in_use.contains(&letter)
    }

    pub fn allocation_type(&mut self, register: IRRegister) -> AllocationType
    {
        if self.ir_to_x86.contains_key(&register) {
            AllocationType::Register
        } else if self.ir_to_stack_offset.contains_key(&register) {
            AllocationType::Stack
        } else {
            panic!()
        }
    }

    pub fn register_for(&mut self, register: IRRegister) -> X86Register
    {
        self.ir_to_x86[&register].clone()
    }

    pub fn stack_offset(&mut self, register: IRRegister) -> usize
    {
        let (offset, _) = self.ir_to_stack_offset[&register];
        self.stack_size - offset
    }

    fn next_available_register(&mut self, size: usize) -> Option<(X86Register, char)>
    {
        for letter in ['a', 'b', 'c', 'd']
        {
            if !self.registers_in_use.contains(&letter) {
                return Some((X86Register::General(letter, size), letter));
            }
        }

        None
    }

    fn allocate_register(&mut self, register: IRRegister, size: usize) -> AllocationType
    {
        // FIXME: Use stack if we don't have any registers left.
        let (x86_register, letter) = self.next_available_register(size)
            .expect("We have an available register");

        self.registers_in_use.insert(letter);
        self.ir_to_x86.insert(register, x86_register);
        return AllocationType::Register;
    }

    fn allocate_stack(&mut self, register: IRRegister, size: usize) -> AllocationType
    {
        self.stack_size += size;
        self.ir_to_stack_offset.insert(register, (self.stack_size, size));
        AllocationType::Stack
    }

    pub fn allocate(&mut self, register: IRRegister, size: usize) -> AllocationType
    {
        assert!(!self.ir_to_x86.contains_key(&register));
        assert!(!self.ir_to_stack_offset.contains_key(&register));
        if size <= 4 {
            self.allocate_register(register, size)
        } else {
            self.allocate_stack(register, size)
        }
    }

    pub fn free(&mut self, register: IRRegister) -> (AllocationType, usize)
    {
        if self.ir_to_x86.contains_key(&register)
        {
            let x86_register = self.ir_to_x86.remove(&register).unwrap();
            let size = match x86_register
            {
                X86Register::General(letter, size) =>
                {
                    assert!(self.registers_in_use.remove(&letter));
                    size
                },
                _ => panic!(),
            };

            return (AllocationType::Register, size);
        }

        if self.ir_to_stack_offset.contains_key(&register)
        {
            let (stack_offset, size) = self.ir_to_stack_offset.remove(&register).unwrap();
            assert_eq!(stack_offset, self.stack_size);
            self.stack_size -= size;

            return (AllocationType::Stack, size);
        }

        eprintln!("{}", register);
        panic!();
    }

    pub fn allocate_scratch_register(&mut self, size: usize) -> X86Register
    {
        // FIXME: If no registers are available, reuse one by pushing 
        //        its value onto the stack.
        let (x86_register, letter) = self.next_available_register(size)
            .expect("We have an available register");

        self.registers_in_use.insert(letter);
        x86_register
    }

    pub fn free_scratch_register(&mut self, register: X86Register)
    {
        match register
        {
            X86Register::General(letter, _) =>
                assert!(self.registers_in_use.remove(&letter)),
            _ => panic!(),
        }
    }

}

