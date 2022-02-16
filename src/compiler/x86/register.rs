use std::fmt;
use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum X86Size
{
    Dword,
    Byte,
    Big(usize),
}

impl fmt::Display for X86Size
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Self::Dword => write!(f, "dword"),
            Self::Byte => write!(f, "byte"),
            Self::Big(_) => panic!(),
        }
    }
}

impl X86Size
{

    pub fn from_bytes(bytes: usize) -> Self
    {
        match bytes
        {
            4 => Self::Dword,
            1 => Self::Byte,
            bytes => Self::Big(bytes),
        }
    }

    pub fn bytes(&self) -> usize
    {
        match self
        {
            Self::Dword => 4,
            Self::Byte => 1,
            Self::Big(bytes) => *bytes,
        }
    }

}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum X86Register
{
    General(char, X86Size),
    Special(&'static str),
}

impl X86Register
{
    
    pub fn eax() -> Self { Self::General('a', X86Size::Dword) }
    pub fn esp() -> Self { Self::Special("esp") }
    pub fn ebp() -> Self { Self::Special("ebp") }

    pub fn offset(&self, size: X86Size, offset: i32) -> String
    {
        if offset == 0 {
            format!("{} [{}]", size, self)
        } else if offset >= 0 {
            format!("{} [{}+{}]", size, self, offset)
        } else {
            format!("{} [{}-{}]", size, self, -offset)
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
                match size
                {
                    X86Size::Dword => write!(f, "e{}x", letter),
                    X86Size::Byte => write!(f, "{}l", letter),
                    X86Size::Big(_) => panic!(),
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

    pub fn allocate(&mut self, size: X86Size) -> Option<X86Register>
    {
        // TODO: We don't need to take up a whole register
        //       if we're only using a single byte.

        for letter in &['a', 'b', 'c']
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

