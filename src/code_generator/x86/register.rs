use std::fmt;

const DWORD: usize = 4;
const WORD: usize = 2;
const BYTE: usize = 1;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum X86Register
{
    General(char, usize, bool),
    Special(&'static str),
}

impl X86Register
{
    
    pub fn ebp() -> Self { Self::Special("ebp") }
    pub fn esp() -> Self { Self::Special("esp") }

    pub fn offset(&self, size: usize, offset: i32) -> String
    {
        let size_name = match size
        {
            x if x == DWORD => "dword",
            x if x == WORD => "word",
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

    pub fn of_size(&self, size: usize, is_high: bool) -> X86Register
    {
        match self
        {
            Self::General(letter, _, _) => X86Register::General(*letter, size, is_high),
            _ => panic!(),
        }
    }

    pub fn size(&self) -> usize
    {
        match self
        {
            Self::General(_, size, _) => *size,
            Self::Special(_) => 4,
        }
    }

}

impl fmt::Display for X86Register
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Self::General(letter, size, is_high) =>
                match *size
                {
                    x if x == DWORD => write!(f, "e{}x", letter),
                    x if x == WORD => write!(f, "{}x", letter),
                    x if x == BYTE =>
                    {
                        if *is_high {
                            write!(f, "{}h", letter)
                        } else {
                            write!(f, "{}l", letter)
                        }
                    },
                    _ => panic!(),
                },

            Self::Special(reg) =>
                write!(f, "{}", reg),
        }
    }

}

