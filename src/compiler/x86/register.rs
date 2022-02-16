use std::fmt;
use std::collections::HashSet;

pub const DWORD: usize = 4;
pub const BYTE: usize = 4;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum X86Register
{
    General(char, usize),
    Special(&'static str),
}

impl X86Register
{
    
    pub fn eax() -> Self { Self::General('a', DWORD) }
    pub fn esp() -> Self { Self::Special("esp") }
    pub fn ebp() -> Self { Self::Special("ebp") }

    pub fn offset(&self, size: usize, offset: i32) -> String
    {
        let size_name = match size
        {
            x if x == DWORD => "dword",
            x if x == BYTE => "byte",
            _ => panic!(),
        };

        if offset == 0 {
            format!("{} [{}]", size_name, self)
        } else if offset >= 0 {
            format!("{} [{}+{}]", size_name, self, offset)
        } else {
            format!("{} [{}-{}]", size_name, self, -offset)
        }
    }

}

impl fmt::Display for X86Register
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Self::General(letter, size) =>
                match *size
                {
                    x if x == DWORD => write!(f, "e{}x", letter),
                    x if x == BYTE => write!(f, "{}l", letter),
                    _ => panic!(),
                },

            Self::Special(reg) =>
                write!(f, "{}", reg),
        }
    }

}

#[derive(Debug)]
pub struct RegisterAllocator
{
    registers_in_use: HashSet<char>,
    data_stack_size: usize,
}

impl RegisterAllocator
{

    pub fn new() -> RegisterAllocator
    {
        RegisterAllocator
        {
            registers_in_use: HashSet::new(),
            data_stack_size: 0,
        }
    }

    pub fn in_use(&self, register: X86Register) -> bool
    {
        match register
        {
            X86Register::General(letter, _) => self.registers_in_use.contains(&letter),
            _ => panic!(),
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<X86Register>
    {
        // TODO: We don't need to take up a whole register
        //       if we're only using a single byte.

        for letter in &['a', 'b', 'c', 'd']
        {
            if self.registers_in_use.contains(&letter) {
                continue;
            }
            eprintln!("Allocate {}", letter);

            let register = X86Register::General(*letter, size);
            self.registers_in_use.insert(*letter);
            return Some(register);
        }

        None
    }

    pub fn free(&mut self, register: X86Register)
    {
        match register
        {
            X86Register::General(letter, _) =>
            {
                eprintln!("Free {}", letter);
                if !self.registers_in_use.remove(&letter)
                {
                    // NOTE: Double free
                    panic!();
                }
            },

            _ => panic!(),
        };
    }

    pub fn pushed_value_on_stack(&mut self, size: usize) -> usize
    {
        eprintln!("Allocate stack {} bytes", size);
        self.data_stack_size += size;
        self.data_stack_size
    }

    pub fn poped_value_on_stack(&mut self, position: usize, size: usize)
    {
        eprintln!("Free stack {} bytes", size);
        assert_eq!(position, self.data_stack_size);
        self.data_stack_size -= size;
    }

    pub fn is_stack_postion_top(&self, position: usize) -> bool
    {
        self.data_stack_size == position
    }

}

