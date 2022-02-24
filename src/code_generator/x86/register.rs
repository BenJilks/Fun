use std::fmt;

const DWORD: usize = 4;
const BYTE: usize = 1;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum X86Register
{
    General(char, usize),
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

    pub fn of_size(&self, size: usize) -> X86Register
    {
        match self
        {
            Self::General(letter, _) => X86Register::General(*letter, size),
            _ => panic!(),
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

