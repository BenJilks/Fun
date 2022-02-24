mod register;
mod allocator;
use register::X86Register;
use allocator::{Allocator, AllocationType};
use crate::intermediate::{IR, IROperation, IRFunction, IRProgram};
use crate::intermediate::IRStorage;
use std::io::Write;
use std::collections::HashMap;
use std::error::Error;

struct X86Output<W>
{
    stream: W,
    allocator: Allocator,
    strings: HashMap<String, usize>,
}

impl<W> X86Output<W>
    where W: Write
{

    fn new(stream: W) -> Self
    {
        Self
        {
            stream,
            allocator: Allocator::new(),
            strings: HashMap::new(),
        }
    }

    fn emit(&mut self, line: String)
        -> Result<(), Box<dyn Error>>
    {
        writeln!(self.stream, "{}", line)?;
        Ok(())
    }

    fn get_string_id(&mut self, s: &str) -> usize
    {
        match self.strings.get(s)
        {
            Some(id) => *id,
            None =>
            {
                self.strings.insert(s.to_owned(), self.strings.len());
                self.strings.len() - 1
            },
        }
    }

    fn generate_set_i32(&mut self, to: &IRStorage, i: i32)
        -> Result<(), Box<dyn Error>>
    {
        let to_str = self.value_of(4, to);
        self.emit(format!("mov {}, {}", to_str, i))?;
        Ok(())
    }

    fn generate_set_i8(&mut self, to: &IRStorage, i: i8)
        -> Result<(), Box<dyn Error>>
    {
        let to_str = self.value_of(1, to);
        self.emit(format!("mov {}, {}", to_str, i))?;
        Ok(())
    }

    fn generate_set_string(&mut self, to: &IRStorage, s: &str)
        -> Result<(), Box<dyn Error>>
    {
        let to_str = self.value_of(4, to);
        let id = self.get_string_id(s);
        self.emit(format!("mov {}, str{}", to_str, id))?;
        Ok(())
    }

    fn generate_set_ref(&mut self, to: &IRStorage, value: &IRStorage)
        -> Result<(), Box<dyn Error>>
    {
        let to_str = self.value_of(4, to);
        self.emit(format!("mov {}, ebp", to_str))?;

        match value
        {
            IRStorage::Register(_) => panic!(),

            IRStorage::Param(offset) =>
                self.emit(format!("add {}, {}", to_str, offset + 8))?,

            IRStorage::Local(offset) =>
                self.emit(format!("sub {}, {}", to_str, offset))?,
        }

        Ok(())
    }

    fn generate_deref(&mut self, to: &IRStorage, value: &IRStorage, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        assert!(size <= 4);
        let to_str = self.value_of(size, to);
        let value_str = self.value_of(4, value);
        self.emit(format!("mov {}, [{}]", to_str, value_str))?;
        Ok(())
    }

    fn value_of(&mut self, size: usize, storage: &IRStorage) -> String
    {
        match storage
        {
            IRStorage::Register(register) =>
            {
                assert_eq!(self.allocator.allocation_type(*register), AllocationType::Register);
                let x86_register = self.allocator.register_for(*register);
                format!("{}", x86_register)
            },

            IRStorage::Param(offset) =>
                format!("{}", X86Register::ebp().offset(size, *offset as i32 + 8)),

            IRStorage::Local(offset) =>
                format!("{}", X86Register::ebp().offset(size, -(*offset as i32))),
        }
    }

    fn offset_of(&mut self, storage: &IRStorage) -> (X86Register, i32)
    {
        match storage
        {
            IRStorage::Register(register) =>
            {
                assert_eq!(self.allocator.allocation_type(*register), AllocationType::Stack);
                let stack_offset = self.allocator.stack_offset(*register);
                (X86Register::esp(), stack_offset as i32)
            },

            IRStorage::Param(offset) =>
                (X86Register::ebp(), *offset as i32 + 8),

            IRStorage::Local(offset) =>
                (X86Register::ebp(), -(*offset as i32)),
        }
    }

    fn generate_move_to_offset(&mut self,
                               offset: usize,
                               to: &IRStorage,
                               from: &IRStorage,
                               size: usize)
        -> Result<(), Box<dyn Error>>
    {
        let from_value = self.value_of(size, from);
        match to
        {
            IRStorage::Register(register) =>
            {
                assert_eq!(self.allocator.allocation_type(*register), AllocationType::Stack);
                let stack_offset = self.allocator.stack_offset(*register);
                self.emit(format!("mov {}, {}",
                    X86Register::esp().offset(size, stack_offset as i32 + offset as i32),
                    from_value))?;
            },

            IRStorage::Param(param_offset) =>
            {
                self.emit(format!("mov {}, {}",
                    X86Register::ebp().offset(size, *param_offset as i32 + 8 + offset as i32),
                    from_value))?;
            },

            IRStorage::Local(local_offset) =>
            {
                self.emit(format!("mov {}, {}",
                    X86Register::ebp().offset(size, -(*local_offset as i32) + offset as i32),
                    from_value))?;
            },
        }

        Ok(())
    }

    fn generate_copy(&mut self,
                     to_register: X86Register,
                     to_offset: i32,
                     from_register: X86Register,
                     from_offset: i32,
                     size: usize)
        -> Result<(), Box<dyn Error>>
    {
        let scratch_register = self.allocator.allocate_scratch_register(4);
        for i in (0..size as i32).step_by(4)
        {
            self.emit(format!("mov {}, {}",
                scratch_register, from_register.offset(4, from_offset + i)))?;
            self.emit(format!("mov {}, {}",
                to_register.offset(4, to_offset + i), scratch_register))?;
        }

        self.allocator.free_scratch_register(scratch_register);
        Ok(())
    }

    fn generate_large_move(&mut self, to: &IRStorage, from: &IRStorage, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        let (to_register, to_offset) = self.offset_of(to);
        let (from_register, from_offset) = self.offset_of(from);
        self.generate_copy(to_register, to_offset, from_register, from_offset, size)
    }

    fn generate_move(&mut self, to: &IRStorage, from: &IRStorage, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        if size > 4 {
            return self.generate_large_move(to, from, size);
        }

        let to_value = self.value_of(size, to);
        let from_value = self.value_of(size, from);
        self.emit(format!("mov {}, {}", to_value, from_value))?;
        Ok(())
    }

    fn is_eax(&mut self, value: &IRStorage) -> bool
    {
        match value
        {
            IRStorage::Register(register) =>
            {
                if self.allocator.allocation_type(*register) == AllocationType::Register {
                    self.allocator.register_for(*register) == X86Register::eax()
                } else {
                    false
                }
            },

            _ => false,
        }
    }

    fn generate_return(&mut self, value: &IRStorage, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        if !self.is_eax(value) && size <= 4
        {
            let value_str = self.value_of(size, value);
            self.emit(format!("mov eax, {}", value_str))?;
        }

        self.emit(format!("mov esp, ebp"))?;
        self.emit(format!("pop ebp"))?;
        self.emit(format!("ret"))?;
        Ok(())
    }

    fn generate_push_u8(&mut self, i: i8)
        -> Result<(), Box<dyn Error>>
    {
        self.emit(format!("sub esp, 1"))?;
        self.emit(format!("mov byte [esp], {}", i))?;
        Ok(())
    }

    fn generate_push_string(&mut self, s: &str)
        -> Result<(), Box<dyn Error>>
    {
        let id = self.get_string_id(s);
        self.emit(format!("push str{}", id))?;
        Ok(())
    }

    fn generate_call(&mut self, function: &str, return_value: &IRStorage, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        let is_eax = self.is_eax(return_value);
        let is_eax_in_use = self.allocator.is_in_use('e');
        if !is_eax && is_eax_in_use {
            self.emit(format!("push eax"))?;
        }

        self.emit(format!("call {}", function))?;
        if !is_eax && is_eax_in_use
        {
            if size > 0
            {
                let return_str = self.value_of(size, return_value);
                self.emit(format!("mov {}, eax", return_str))?;
            }
            self.emit(format!("pop eax"))?;
        }
        Ok(())
    }

    fn generate_push(&mut self, value: &IRStorage, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        if size > 4
        {
            let (from_register, from_offset) = self.offset_of(value);
            self.emit(format!("sub esp, {}", size))?;
            self.generate_copy(X86Register::esp(), 0, from_register, from_offset, size)?;
        }
        else
        {
            let value_str = self.value_of(size, value);
            self.emit(format!("push {}", value_str))?;
        }

        Ok(())
    }

    fn i32_operation_str(operation: &IROperation) -> (bool, &str)
    {
        match operation
        {
            IROperation::Add => (false, "add"),
            IROperation::Subtract => (false, "sub"),
            IROperation::Multiply => (false, "imul"),
            IROperation::GreaterThan => (true, "setg"),
            IROperation::LessThan=> (true, "setl"),
        }
    }

    fn generate_i32_operation(&mut self,
                              operation: &IROperation,
                              to: &IRStorage,
                              lhs: &IRStorage,
                              rhs: &IRStorage)
        -> Result<(), Box<dyn Error>>
    {
        let (is_comparison, operation_str) = Self::i32_operation_str(operation);
        let to_str = self.value_of(4, to);
        let lhs_str = self.value_of(4, lhs);
        let rhs_str = self.value_of(4, rhs);
        if is_comparison
        {
            let scratch_register = self.allocator.allocate_scratch_register(4);
            self.emit(format!("mov {}, {}", scratch_register, lhs_str))?;
            self.emit(format!("cmp {}, {}", scratch_register, rhs_str))?;
            self.emit(format!("{} {}", operation_str, to_str))?;
            self.allocator.free_scratch_register(scratch_register);
        }
        else
        {
            self.emit(format!("mov {}, {}", to_str, lhs_str))?;
            self.emit(format!("{} {}, {}", operation_str, to_str, rhs_str))?;
        }

        Ok(())
    }

    fn generate_i32_constant_operation(&mut self,
                                       operation: &IROperation,
                                       to: &IRStorage,
                                       lhs: &IRStorage,
                                       i: i32)
        -> Result<(), Box<dyn Error>>
    {
        let (is_comparison, operation_str) = Self::i32_operation_str(operation);
        let to_str = self.value_of(4, to);
        let lhs_str = self.value_of(4, lhs);
        if is_comparison
        {
            self.emit(format!("cmp {}, {}", lhs_str, i))?;
            self.emit(format!("{} {}", operation_str, to_str))?;
        }
        else
        {
            self.emit(format!("mov {}, {}", to_str, lhs_str))?;
            self.emit(format!("{} {}, {}", operation_str, to_str, i))?;
        }

        Ok(())
    }

    fn generate_goto_if_not(&mut self, label: &str, condition: &IRStorage)
        -> Result<(), Box<dyn Error>>
    {
        let condition_str = self.value_of(1, condition);
        self.emit(format!("cmp {}, 0", condition_str))?;
        self.emit(format!("jz {}", label))?;
        Ok(())
    }

    fn generate_function(&mut self, function: &IRFunction)
        -> Result<(), Box<dyn Error>>
    {
        self.emit(format!("{}:", function.name))?;
        self.emit(format!("push ebp"))?;
        self.emit(format!("mov ebp, esp"))?;
        if function.stack_frame_size > 0 {
            self.emit(format!("sub esp, {}", function.stack_frame_size))?;
        }

        for ir in &function.code
        {
            self.emit(format!("; {}", ir))?;
            match ir
            {
                IR::AllocateRegister(register, size) => 
                {
                    match self.allocator.allocate(*register, *size)
                    {
                        AllocationType::Register => {},
                        AllocationType::Stack => self.emit(format!("sub esp, {}", size))?,
                    }
                },
                IR::FreeRegister(register) =>
                {
                    let (allocation_type, size) = self.allocator.free(*register);
                    match allocation_type
                    {
                        AllocationType::Register => {},
                        AllocationType::Stack => self.emit(format!("add esp, {}", size))?,
                    }
                },

                IR::SetI32(to, i) => self.generate_set_i32(to, *i)?,
                IR::SetI8(to, i) => self.generate_set_i8(to, *i)?,
                IR::SetString(to, s) => self.generate_set_string(to, s)?,
                IR::SetRef(to, value) => self.generate_set_ref(to, value)?,
                IR::Deref(to, value, size) => self.generate_deref(to, value, *size)?,
                IR::Move(to, from, size) => self.generate_move(to, from, *size)?,

                IR::MoveToOffset(offset, to, from, size) =>
                    self.generate_move_to_offset(*offset, to, from, *size)?,

                // IR::MoveFromOffset(_, _, _, _) => panic!(),
                IR::PushI32(i) => self.emit(format!("push {}", i))?,
                IR::PushI8(i) => self.generate_push_u8(*i)?,
                IR::PushString(s) => self.generate_push_string(s)?,
                IR::Push(value, size) => self.generate_push(value, *size)?,
                IR::Pop(count) => self.emit(format!("add esp, {}", count))?,

                IR::I32ConstantOperation(op, to, lhs, i) => self.generate_i32_constant_operation(op, to, lhs, *i)?,
                IR::I32Operation(op, to, lhs, rhs) => self.generate_i32_operation(op, to, lhs, rhs)?,

                IR::Call(function, return_value, size) => self.generate_call(function, return_value, *size)?,
                IR::Label(label) => self.emit(format!("{}:", label))?,
                IR::Goto(label) => self.emit(format!("jmp {}", label))?,
                IR::GotoIfNot(label, condition) => self.generate_goto_if_not(label, condition)?,
                IR::Return(value, size) => self.generate_return(value, *size)?,
            }
        }

        self.emit(format!(""))?;
        Ok(())
    }

    pub fn generate_extern(&mut self, extern_: &str)
        -> Result<(), Box<dyn Error>>
    {
        self.emit(format!("extern {}", extern_))?;
        Ok(())
    }

    pub fn generate_header(&mut self)
        -> Result<(), Box<dyn Error>>
    {
        self.emit(format!("global main"))?;
        self.emit(format!("section .text"))?;
        self.emit(format!(""))?;
        Ok(())
    }

    pub fn generate_footer(&mut self)
        -> Result<(), Box<dyn Error>>
    {
        self.emit(format!("section .data"))?;
        for (string, id) in self.strings.clone() {
            self.emit(format!("str{}: db \"{}\", 0", id, string))?;
        }

        self.emit(format!(""))?;
        Ok(())
    }

}

pub fn generate(program: IRProgram, stream: &mut impl Write)
    -> Result<(), Box<dyn Error>>
{
    let mut output = X86Output::new(stream);
    output.generate_header()?;

    for function in &program.functions {
        output.generate_function(function)?;
    }
    for extern_ in &program.externs {
        output.generate_extern(extern_)?;
    }

    output.generate_footer()?;
    Ok(())
}

