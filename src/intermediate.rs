use std::fmt;

pub type IRRegister = usize;
pub type IRParam = usize;
pub type IRLocal = usize;

#[derive(Clone)]
pub enum IRStorage
{
    Register(IRRegister),
    Param(IRParam),
    Local(IRLocal),
}

impl fmt::Display for IRStorage
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Self::Register(register) => write!(f, "r{}", register),
            Self::Param(param) => write!(f, "p{}", param),
            Self::Local(local) => write!(f, "l{}", local),
        }
    }

}

#[derive(Clone)]
pub enum IROperation
{
    Add,
    Subtract,
    Multiply,
    GreaterThan,
    LessThan,
}

impl fmt::Display for IROperation
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Self::Add => write!(f, "add"),
            Self::Subtract => write!(f, "subtract"),
            Self::Multiply => write!(f, "multiply"),
            Self::GreaterThan => write!(f, "greater than"),
            Self::LessThan => write!(f, "less than"),
        }
    }

}

#[derive(Clone)]
pub enum IR
{
    AllocateRegister(IRRegister, usize),
    FreeRegister(IRRegister),

    SetI32(IRStorage, i32),
    SetI8(IRStorage, i8),
    SetString(IRStorage, String),
    SetRef(IRStorage, IRStorage),
    Deref(IRStorage, IRStorage, usize),
    Move(IRStorage, IRStorage, usize),
    MoveToOffset(usize, IRStorage, IRStorage, usize),
    // MoveFromOffset(IRStorage, usize, IRStorage, usize),
    PushI32(i32),
    PushI8(i8),
    PushString(String),
    Push(IRStorage, usize),
    Pop(usize),

    I32ConstantOperation(IROperation, IRStorage, IRStorage, i32),
    I32Operation(IROperation, IRStorage, IRStorage, IRStorage),

    Call(String, IRStorage, usize),
    Label(String),
    Goto(String),
    GotoIfNot(String, IRStorage),
    Return(IRStorage, usize),
}

impl fmt::Display for IR
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Self::AllocateRegister(register, size) => write!(f, "allocate {}, {}", register, size),
            Self::FreeRegister(register) => write!(f, "free {}", register),
            Self::SetI32(storage, i) => write!(f, "set {}, {}", storage, i),
            Self::SetI8(storage, i) => write!(f, "set {}, {}", storage, i),
            Self::SetString(storage, s) => write!(f, "set {}, {}", storage, s),
            Self::SetRef(to, value) => write!(f, "set {}, ref {}", to, value),
            Self::Deref(to, value, size) => write!(f, "set {}, deref {} {}", to, value, size),
            Self::Move(from, to, size) => write!(f, "move {}, {}, {}", from, to, size),
            Self::MoveToOffset(offset, from, to, size) => write!(f, "move {}[{}], {}, {}", from, offset, to, size),
            // Self::MoveFromOffset(from, offset, to, size) => write!(f, "move {}, {}[{}], {}", from, to, offset, size),
            Self::PushI32(value) => write!(f, "push {}", value),
            Self::PushI8(value) => write!(f, "push {}", value),
            Self::PushString(value) => write!(f, "push '{}'", value),
            Self::Push(value, size) => write!(f, "push {}, {}", value, size),
            Self::Pop(count) => write!(f, "pop {}", count),
            Self::I32ConstantOperation(op, to, lhs, rhs) => write!(f, "{} {}, {}, {}", op, to, lhs, rhs),
            Self::I32Operation(op, to, lhs, rhs) => write!(f, "{} {}, {}, {}", op, to, lhs, rhs),
            Self::Call(function, return_value, size) => write!(f, "call {}, {}, {}", function, return_value, size),
            Self::Label(label) => write!(f, "{}:", label),
            Self::Goto(label) => write!(f, "goto {}", label),
            Self::GotoIfNot(label, condition) => write!(f, "goto if not {}, {}", label, condition),
            Self::Return(value, size) => write!(f, "return {}, {}", value, size),
        }
    }

}

#[derive(Clone)]
pub struct IRFunction
{
    pub name: String,
    pub code: Vec<IR>,
    pub stack_frame_size: usize,
}

pub struct IRProgram
{
    pub functions: Vec<IRFunction>,
    pub externs: Vec<String>,
}

impl IRProgram
{
    pub fn new() -> Self
    {
        Self
        {
            functions: Vec::new(),
            externs: Vec::new(),
        }
    }
}

