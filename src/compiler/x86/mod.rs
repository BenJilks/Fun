mod register;
mod value;
use register::{X86Register, RegisterAllocator};
use value::{X86Value, X86StorageLocation};
use super::CodeGenortator;
use std::str::from_utf8;
use std::io::Write;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;

pub struct X86CodeGenorator<Output>
    where Output: Write + 'static
{
    output: Output,
    register_allocator: Rc<RefCell<RegisterAllocator>>,
    
    label_count: usize,
    strings: HashMap<String, Rc<X86Value>>,
    externs: HashSet<String>,

    function_buffer: Option<Rc<RefCell<Vec<u8>>>>,
    stack_frame_size: i32,
}

impl<Output> X86CodeGenorator<Output>
    where Output: Write + 'static
{

    pub fn new(output: Output) -> Result<Self, Box<dyn Error>>
    {
        let mut gen = Self
        {
            output: output,
            register_allocator: Rc::from(RefCell::from(RegisterAllocator::new())),
            label_count: 0,
            strings: HashMap::new(),
            externs: HashSet::new(),

            function_buffer: None,
            stack_frame_size: 0,
        };

        gen.emit_header()?;
        Ok(gen)
    }

    fn new_value(&self, location: X86StorageLocation)
        -> Rc<X86Value>
    {
        Rc::from(X86Value
        {
            location,
            allocator: self.register_allocator.clone(),
            output: self.function_buffer.clone().unwrap(),
        })
    }

    fn emit_line(&mut self, line: String) -> Result<(), Box<dyn Error>>
    {
        match &mut self.function_buffer
        {
            Some(buffer) => writeln!(buffer.borrow_mut(), "{}", line)?,
            None => writeln!(self.output, "{}", line)?,
        }
        Ok(())
    }

    fn emit_header(&mut self) -> Result<(), Box<dyn Error>>
    {
        self.emit_line(format!("global main"))?;
        self.emit_line(format!("section .text"))?;
        Ok(())
    }

    fn emit_footer(&mut self) -> Result<(), Box<dyn Error>>
    {
        self.end_current_function()?;

        self.emit_line(format!(""))?;
        self.emit_line(format!("section .data"))?;
        for (s, name) in self.strings.clone() {
            self.emit_line(format!("{}: db \"{}\", 0", name.location, s))?;
        }

        self.emit_line(format!(""))?;
        for name in self.externs.clone() {
            self.emit_line(format!("extern {}", name))?;
        }

        self.emit_line(format!(""))?;
        Ok(())
    }

    fn move_to_register(&mut self, stack_value: Rc<X86Value>)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        let register_or_none = self.register_allocator.borrow_mut().allocate(register::DWORD);
        if register_or_none.is_none() {
            panic!(); // TODO
        }

        let register = register_or_none.unwrap();
        match stack_value.location
        {
            X86StorageLocation::I32(i) if i == 0 =>
                self.emit_line(format!("xor {}, {}", register, register))?,

            _ => self.emit_line(format!("mov {}, {}", register, stack_value.location))?,
        }

        Ok(self.new_value(X86StorageLocation::Register(register)))
    }

    fn mov_into_struct(&mut self, to: Rc<X86Value>, from: Rc<X86Value>)
        -> Result<(), Box<dyn Error>>
    {
        match &from.location
        {
            X86StorageLocation::StructData(data) =>
            {
                for (offset, value, size) in data
                {
                    let field = self.new_value(X86StorageLocation::Local(*offset, *size));
                    let field_value = self.access(to.clone(), field)?;
                    self.mov(field_value, value.clone(), *size)?;
                }
            },

            _ => panic!(),
        }

        Ok(())
    }

    fn mov_into_array(&mut self, to: Rc<X86Value>, from: Rc<X86Value>)
        -> Result<(), Box<dyn Error>>
    {
        match &from.location
        {
            X86StorageLocation::ArrayData(data, item_size) =>
            {
                for (i, value) in data.into_iter().enumerate()
                {
                    let offset = (i * item_size) as i32;
                    let item = self.new_value(X86StorageLocation::Local(
                        offset, *item_size));
                    
                    let item_value = self.access(to.clone(), item)?;
                    self.mov(item_value, value.clone(), *item_size)?;
                }
            },

            _ => panic!(),
        }

        Ok(())
    }

    fn ensure_moveable(&mut self, value: Rc<X86Value>)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        match value.location
        {
            X86StorageLocation::Null => Ok(value),
            X86StorageLocation::Register(_) => Ok(value),
            X86StorageLocation::Deref(_, _) => Ok(value),
            X86StorageLocation::Local(_, _) => self.move_to_register(value),
            X86StorageLocation::Stack(_, _) => self.move_to_register(value),
            X86StorageLocation::Constant(_) => Ok(value),
            X86StorageLocation::I32(_) => Ok(value),
            X86StorageLocation::I8(_) => Ok(value),

            // TODO: Proper error here.
            X86StorageLocation::StructData(_) => panic!(),
            X86StorageLocation::ArrayData(_, _) => panic!(),
        }
    }

    fn ensure_register(&mut self, value: Rc<X86Value>)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        match value.location
        {
            X86StorageLocation::Null => Ok(value),
            X86StorageLocation::Register(_) => Ok(value),
            X86StorageLocation::Deref(_, _) => Ok(value),
            X86StorageLocation::Local(_, _) => self.move_to_register(value),
            X86StorageLocation::Stack(_, _) => self.move_to_register(value),
            X86StorageLocation::Constant(_) => self.move_to_register(value),
            X86StorageLocation::I32(_) => self.move_to_register(value),
            X86StorageLocation::I8(_) => self.move_to_register(value),

            // TODO: Proper error here.
            X86StorageLocation::StructData(_) => panic!(),
            X86StorageLocation::ArrayData(_, _) => panic!(),
        }
    }

    fn end_current_function(&mut self) -> Result<(), Box<dyn Error>>
    {
        if self.function_buffer.is_none() {
            return Ok(());
        }

        let buffer_str = from_utf8(&self.function_buffer.as_ref().unwrap().borrow())?.to_owned();
        self.function_buffer = None;
        self.stack_frame_size = 0;

        if self.stack_frame_size > 0 {
            self.emit_line(format!("sub esp, {}", self.stack_frame_size))?;
        }
        self.emit_line(format!("{}", buffer_str))?;
        Ok(())
    }

    fn mem_copy(&mut self, size: usize,
                to_register: X86Register, to_offset: i32,
                from_register: X86Register, from_offset: i32)
        -> Result<(), Box<dyn Error>>
    {
        self.label(&format!("Copy {} bytes {}:{} <- {}:{}",
            size, to_register, to_offset, from_register, from_offset))?;

        // TODO: We should use the stack if no registers are available.
        let temp_or_none = self.register_allocator.borrow_mut().allocate(register::DWORD);
        assert!(temp_or_none.is_some());

        let temp = temp_or_none.unwrap();
        for i in (0..size as i32).step_by(4)
        {
            self.emit_line(format!("mov {}, {}",
                temp, from_register.offset(register::DWORD, from_offset + i)))?;

            self.emit_line(format!("mov {}, {}",
                to_register.offset(register::DWORD, to_offset + i), temp))?;
        }

        self.register_allocator.borrow_mut().free(temp);
        Ok(())
    }

    fn mov_large(&mut self, to: Rc<X86Value>, from: Rc<X86Value>, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        match to.location
        {
            X86StorageLocation::Local(to_offset, _) =>
            {
                match &from.location
                {
                    X86StorageLocation::Local(from_offset, from_size) =>
                    {
                        assert_eq!(size, *from_size);
                        self.mem_copy(size,
                            X86Register::ebp(), to_offset, 
                            X86Register::ebp(), *from_offset)?;
                    },

                    X86StorageLocation::Stack(_, from_size) =>
                    {
                        assert_eq!(size, *from_size);
                        self.mem_copy(size,
                            X86Register::ebp(), to_offset,
                            X86Register::esp(), 0)?;
                    },

                    // TODO: Implement this
                    X86StorageLocation::Constant(_) => panic!(),

                    // TODO: Proper error here.
                    _ => panic!(),
                }
            },

            // TODO: Proper error here.
            _ => panic!(),
        }

        Ok(())
    }

    fn push(&mut self, value: Rc<X86Value>, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        if size > 4
        {
            match &value.location
            {
                X86StorageLocation::Local(offset, _) =>
                {
                    self.label("Push local")?;

                    self.emit_line(format!("sub esp, {}", size))?;
                    self.mem_copy(size,
                        X86Register::esp(), 0, 
                        X86Register::ebp(), *offset)?;
                },

                // NOTE: Do nothing, as this is already on the stack
                X86StorageLocation::Stack(_, _) =>
                {
                    self.label("Already on stack")?;
                },

                X86StorageLocation::StructData(data) =>
                {
                    let size = data.iter().map(|(_, _, x)| x).sum();
                    let stack_top = self.new_value(X86StorageLocation::Stack(0, size));
                    self.mov_into_struct(stack_top.clone(), value)?;
                    std::mem::forget(stack_top);
                },

                X86StorageLocation::ArrayData(values, item_size) =>
                {
                    let size = item_size * values.len();
                    let stack_top = self.new_value(X86StorageLocation::Stack(0, size));
                    self.mov_into_array(stack_top.clone(), value)?;
                    std::mem::forget(stack_top);
                },

                // TODO: Implement this
                X86StorageLocation::Constant(_) => panic!(),

                // TODO: Proper error here.
                _ => panic!(),
            }

            return Ok(());
        }

        let computed_value = self.apply_deref(value)?;
        if size == 4 || size == 2
        {
            self.emit_line(format!("push {}", computed_value.location))?;
        }
        else 
        {
            let temp_register_or_none = self.register_allocator.borrow_mut().allocate(register::BYTE);
            assert!(temp_register_or_none.is_some());

            let temp_register = temp_register_or_none.unwrap();
            self.emit_line(format!("sub esp, {}", size))?;
            self.emit_line(format!("mov {}, {}", temp_register, computed_value.location))?;
            self.emit_line(format!("mov [esp], {}", temp_register))?;
            self.register_allocator.borrow_mut().free(temp_register);
        }
        Ok(())
    }

    fn deref_large(&mut self, value_reg: Rc<X86Value>, size: usize)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        match &value_reg.location
        {
            X86StorageLocation::Deref(from_register, _) =>
            {
                self.emit_line(format!("sub esp, {}", size))?;
                self.mem_copy(size,
                    X86Register::esp(), 0,
                    from_register.clone(), 0)?;
            }

            _ => panic!(),
        }

        let position = self.register_allocator.borrow_mut().pushed_value_on_stack(size);
        Ok(self.new_value(X86StorageLocation::Stack(position, size)))
    }

    fn apply_deref(&mut self, value: Rc<X86Value>) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        let size = match &value.location
        {
            X86StorageLocation::Deref(_, size) => *size,
            _ => return Ok(value),
        };

        self.label("Deref")?;

        let value_reg = self.ensure_register(value)?;
        if size > 4 {
            return self.deref_large(value_reg, size);
        }

        let to_register_or_none = self.register_allocator.borrow_mut().allocate(register::DWORD);
        assert!(to_register_or_none.is_some());

        let to_register = to_register_or_none.unwrap();
        self.emit_line(format!("mov {}, {}",
            to_register, value_reg.location))?;

        Ok(self.new_value(X86StorageLocation::Register(to_register)))
    }

    fn arithmatic_operation(&mut self, lhs: Rc<X86Value>, rhs: Rc<X86Value>, operator: &str)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        let computed_lhs = self.apply_deref(lhs)?;
        let computed_rhs = self.apply_deref(rhs)?;

        let lhs_reg = self.ensure_register(computed_lhs)?;
        self.emit_line(format!("{} {}, {}",
            operator, lhs_reg.location, computed_rhs.location))?;

        Ok(lhs_reg)
    }

    fn logic_operation(&mut self, lhs: Rc<X86Value>, rhs: Rc<X86Value>, operator: &str)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        let computed_lhs = self.apply_deref(lhs)?;
        let computed_rhs = self.apply_deref(rhs)?;

        let lhs_reg = self.ensure_register(computed_lhs)?;
        self.emit_line(format!("cmp {}, {}", lhs_reg.location, computed_rhs.location))?;

        let result_register_or_none = self.register_allocator.borrow_mut().allocate(register::BYTE);
        assert!(result_register_or_none.is_some());

        let result_register = result_register_or_none.unwrap();
        self.emit_line(format!("{} {}", operator, result_register))?;

        Ok(self.new_value(X86StorageLocation::Register(result_register)))
    }

    fn label(&mut self, label: &str) -> Result<(), Box<dyn Error>>
    {
        self.emit_line(format!("; {}", label))?;
        Ok(())
    }

}

impl<Output> Drop for X86CodeGenorator<Output>
    where Output: Write + 'static
{

    fn drop(&mut self)
    {
        self.emit_footer().unwrap();
    }

}

impl<Output> CodeGenortator<X86Value> for X86CodeGenorator<Output>
    where Output: Write + 'static
{

    fn create_label(&mut self, name: &str) -> String
    {
        let label = format!("{}{}", name, self.label_count);
        self.label_count += 1;
        label
    }

    fn emit_null(&mut self) -> Rc<X86Value>
    {
        self.new_value(X86StorageLocation::Null)
    }

    fn emit_int(&mut self, i: i32) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        Ok(self.new_value(X86StorageLocation::I32(i)))
    }

    fn emit_string(&mut self, s: &str) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        if !self.strings.contains_key(s)
        {
            let name = format!("str{}", self.strings.len());
            self.strings.insert(s.to_owned(), self.new_value(
                X86StorageLocation::Constant(name)));
        }
       
        Ok(self.strings[s].clone())
    }

    fn emit_char(&mut self, c: char) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        Ok(self.new_value(X86StorageLocation::I8(c as i8)))
    }

    fn emit_extern(&mut self, name: &str)
    {
        self.externs.insert(name.to_owned());
    }

    fn emit_struct_offset(&mut self, offset: i32, size: usize) -> Rc<X86Value>
    {
        self.new_value(X86StorageLocation::Local(offset, size))
    }

    fn emit_struct_data(&mut self, data: impl IntoIterator<Item = (Rc<X86Value>, Rc<X86Value>, usize)>)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        let mut stored_struct_data = Vec::new();
        for (field, value, size) in data
        {
            match field.location
            {
                X86StorageLocation::Local(offset, _) =>
                    stored_struct_data.push((offset, value, size)),

                // TODO: Proper error here.
                _ => panic!(),
            }
        }

        Ok(self.new_value(X86StorageLocation::StructData(stored_struct_data)))
    }

    fn emit_array_literal(&mut self, array: Vec<Rc<X86Value>>, item_size: usize)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        Ok(self.new_value(X86StorageLocation::ArrayData(array, item_size)))
    }

    fn emit_label(&mut self, label: &str) -> Result<(), Box<dyn Error>>
    {
        self.emit_line(format!("{}:", label))?;
        Ok(())
    }

    fn mov(&mut self, to: Rc<X86Value>, from: Rc<X86Value>, size: usize)
        -> Result<(), Box<dyn Error>>
    {
        match &from.location
        {
            X86StorageLocation::StructData(_) =>
                return self.mov_into_struct(to, from),

            X86StorageLocation::ArrayData(_, _) =>
                return self.mov_into_array(to, from),

            _ => {},
        }

        let computed_from = self.apply_deref(from)?;
        if size > 4 {
            return self.mov_large(to, computed_from, size);
        }

        let from_reg = self.ensure_moveable(computed_from)?;
        self.emit_line(format!("mov {}, {}", to.location, from_reg.location))?;
        Ok(())
    }

    fn add(&mut self, lhs: Rc<X86Value>, rhs: Rc<X86Value>) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        self.label("Add")?;
        self.arithmatic_operation(lhs, rhs, "add")
    }

    fn subtract(&mut self, lhs: Rc<X86Value>, rhs: Rc<X86Value>) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        self.label("Subtract")?;
        self.arithmatic_operation(lhs, rhs, "sub")
    }

    fn mul(&mut self, lhs: Rc<X86Value>, rhs: Rc<X86Value>) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        self.label("Mul")?;
        self.arithmatic_operation(lhs, rhs, "imul")
    }

    fn greater_than(&mut self, lhs: Rc<X86Value>, rhs: Rc<X86Value>) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        self.label("Greater Than")?;
        self.logic_operation(lhs, rhs, "setg")
    }

    fn less_than(&mut self, lhs: Rc<X86Value>, rhs: Rc<X86Value>) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        self.label("Less Than")?;
        self.logic_operation(lhs, rhs, "setl")
    }

    fn ref_of(&mut self, value: Rc<X86Value>) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        self.label("Ref")?;
        
        let mut should_forget = false;
        let result = match &value.location
        {
            X86StorageLocation::Local(offset, _) =>
            {
                let register_or_none = self.register_allocator.borrow_mut().allocate(register::DWORD);
                if register_or_none.is_none() {
                    panic!(); // TODO: Proper error here.
                }

                let register = register_or_none.unwrap();
                self.emit_line(format!("mov {}, ebp", register))?;
                if *offset > 0 {
                    self.emit_line(format!("add {}, {}", register, offset))?;
                } else if *offset < 0 {
                    self.emit_line(format!("sub {}, {}", register, -offset))?;
                }

                self.new_value(X86StorageLocation::Register(register))
            },

            X86StorageLocation::Deref(register, _) =>
            {
                should_forget = true;
                self.new_value(X86StorageLocation::Register(register.clone()))
            },

            // TODO: Proper error here.
            _ => panic!(),
        };

        if should_forget {
            std::mem::forget(value);
        }
        Ok(result)
    }

    fn deref(&mut self, value: Rc<X86Value>, size: usize) -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        let result = match &value.location
        {
            X86StorageLocation::Register(register) =>
                self.new_value(X86StorageLocation::Deref(register.clone(), size)),

            _ => panic!(),
        };

        std::mem::forget(value);
        Ok(result)
    }

    fn access(&mut self, value: Rc<X86Value>, field: Rc<X86Value>)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        let (field_offset, field_size) = match field.location
        {
            X86StorageLocation::Local(offset, size) => (offset, size),
            _ => panic!(),
        };

        match &value.location
        {
            X86StorageLocation::Local(offset, _) =>
            {
                Ok(self.new_value(X86StorageLocation::Local(
                    offset + field_offset, field_size)))
            },

            X86StorageLocation::Stack(position, _) =>
            {
                assert!(self.register_allocator.borrow().is_stack_postion_top(*position));
                Ok(self.new_value(X86StorageLocation::Stack(
                    field_offset as usize, field_size)))
            },

            X86StorageLocation::Deref(register, _) =>
            {
                self.emit_line(format!("add {}, {}", register, field_offset))?;
                Ok(self.new_value(X86StorageLocation::Deref(
                    register.clone(), field_size)))
            },

            _ => panic!(),
        }
    }

    // FIXME: This is a really long function, it needs cleaning up.
    fn call<Arguments>(&mut self, function_name: &str, arguments: Arguments,
                       return_size: usize)
            -> Result<Rc<X86Value>, Box<dyn Error>>
        where Arguments: DoubleEndedIterator<Item = (Rc<X86Value>, usize)> + Clone
    {
        // FIXME: We're clobbering a lot of registers here,
        //        we should probable account for that.

        for (argument, _) in arguments.clone()
        {
            // FIXME: This seems a little hacky to me, there's probably a much 
            //        better way of doing this.
            
            // NOTE: If this value is on the stack, don't free it.
            if !matches!(argument.location, X86StorageLocation::Stack{..}) {
                // FIXME: What happens here?
            }
        }

        let is_eax_in_use = self.register_allocator.borrow_mut().in_use(X86Register::eax());
        if is_eax_in_use {
            self.emit_line(format!("push eax"))?;
        }

        let is_big_return = return_size > 4;
        if is_big_return 
        {
            self.label("Allocate return memory")?;
            self.emit_line(format!("sub esp, {}", return_size))?;
        }

        // FIXME: We don't handle this case
        assert!(!(is_eax_in_use && is_big_return));

        let mut total_argument_size = 0;
        for (argument, size) in arguments.rev()
        {
            self.label("Argument")?;
            self.push(argument, size)?;
            total_argument_size += size;
        }

        self.label("Call")?;
        self.emit_line(format!("call {}", function_name))?;
        self.emit_line(format!("add esp, {}", total_argument_size))?;

        if is_big_return
        {
            let position = self.register_allocator.borrow_mut().pushed_value_on_stack(return_size);
            return Ok(self.new_value(X86StorageLocation::Stack(position, return_size)));
        }

        let result_register_or_none = self.register_allocator.borrow_mut().allocate(register::DWORD);
        assert!(result_register_or_none.is_some());

        let result_register = result_register_or_none.unwrap();
        match result_register
        {
            X86Register::General('a', register::DWORD) => {},
            _ => self.emit_line(format!("mov {}, eax", result_register))?,
        }

        if is_eax_in_use {
            self.emit_line(format!("pop eax"))?;
        }

        Ok(self.new_value(X86StorageLocation::Register(result_register)))
    }

    fn ret(&mut self, value: Rc<X86Value>, size: usize, to: Option<Rc<X86Value>>)
        -> Result<(), Box<dyn Error>>
    {
        assert!(self.function_buffer.is_some());

        self.label("Return")?;
        match to
        {
            Some(to) =>
                self.mov(to, value, size)?,

            None =>
            {
                assert!(size <= 4);

                let value_reg = self.ensure_register(value)?;
                match &value_reg.location
                {
                    X86StorageLocation::Register(register) if register != &X86Register::eax() =>
                        self.emit_line(format!("mov {}, {}", X86Register::eax(), value_reg.location))?,
                    _ => {},
                }
            },
        }

        self.emit_line(format!("mov esp, ebp"))?;
        self.emit_line(format!("pop ebp"))?;
        self.emit_line(format!("ret"))?;
        Ok(())
    }

    fn goto(&mut self, label: &str) -> Result<(), Box<dyn Error>>
    {
        self.emit_line(format!("jmp {}", label))?;
        Ok(())
    }

    fn goto_if_not(&mut self, label: &str, condition: Rc<X86Value>)
        -> Result<(), Box<dyn Error>>
    {
        let computed_condition = self.apply_deref(condition)?;
        self.emit_line(format!("cmp {}, 0", computed_condition.location))?;
        self.emit_line(format!("jz {}", label))?;
        Ok(())
    }

    fn start_function(&mut self, function_name: &str,
                      param_sizes: impl Iterator<Item = usize>)
        -> Result<Vec<Rc<X86Value>>, Box<dyn Error>>
    {
        self.end_current_function()?;

        self.emit_line(format!(""))?;
        self.emit_line(format!("{}:", function_name))?;
        self.emit_line(format!("push ebp"))?;
        self.emit_line(format!("mov ebp, esp"))?;
        self.function_buffer = Some(Default::default());

        let mut params = Vec::new();
        let mut last_offset = 0;
        for param_size in param_sizes
        {
            params.push(self.new_value(X86StorageLocation::Local(
                last_offset + 8, param_size)));

            last_offset += param_size as i32;
        }
        Ok(params)
    }

    fn allocate_local(&mut self, size: usize)
        -> Result<Rc<X86Value>, Box<dyn Error>>
    {
        self.stack_frame_size += size as i32;
        Ok(self.new_value(X86StorageLocation::Local(
            -self.stack_frame_size, size)))
    }

}

