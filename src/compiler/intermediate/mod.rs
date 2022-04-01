mod output;
pub mod value;
use output::IROutput;
use value::{IRLocation, IRValue};
use crate::intermediate::{IR, IROperation, IRProgram, IRFunction};
use crate::intermediate::IRStorage;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;

pub struct IRGenorator
{
    output: Rc<RefCell<IROutput>>,
    last_label_id: usize,
}

impl IRGenorator
{

    pub fn new() -> Self
    {
        Self
        {
            output: IROutput::new(),
            last_label_id: 0,
        }
    }

    pub fn program(&mut self) -> IRProgram
    {
        let mut output = self.output.borrow_mut();
        if output.current_function.is_some()
        {
            let function = output.current_function.take().unwrap();
            output.add_function(function);
        }

        output.program.take().unwrap()
    }

    fn allocate(&mut self, size: usize) -> IRStorage
    {
        let mut output = self.output.borrow_mut();
        let register = output.allocator.allocate();
        output.emit_ir(IR::AllocateRegister(register, size));
        IRStorage::Register(register)
    }

    fn new_value(&mut self, location: IRLocation) -> Rc<IRValue>
    {
        Rc::from(IRValue
        {
            location,
            output: self.output.clone(),
        })
    }

    fn emit_ir(&mut self, instruction: IR)
    {
        let mut output = self.output.borrow_mut();
        output.emit_ir(instruction);
    }

    fn ensure_storage(&mut self, value: Rc<IRValue>) -> Rc<IRValue>
    {
        match &value.location
        {
            IRLocation::Null => panic!(),
            IRLocation::Field(_, _) => panic!(),
            IRLocation::Storage(_, _) => value,

            IRLocation::I32(i) =>
            {
                let storage = self.allocate(4);
                self.emit_ir(IR::SetI32(storage.clone(), *i));
                self.new_value(IRLocation::Storage(storage.clone(), 4))
            },

            IRLocation::I8(i) =>
            {
                let storage = self.allocate(1);
                self.emit_ir(IR::SetI8(storage.clone(), *i));
                self.new_value(IRLocation::Storage(storage.clone(), 1))
            },

            IRLocation::String(s) =>
            {
                let storage = self.allocate(s.len());
                self.emit_ir(IR::SetString(storage.clone(), s.clone()));
                self.new_value(IRLocation::Storage(storage.clone(), s.len()))
            },
        }
    }

    fn push(&mut self, value: Rc<IRValue>)
    {
        match &value.location
        {
            IRLocation::Null => panic!(),
            IRLocation::Field(_, _) => panic!(),
            IRLocation::I32(i) => self.emit_ir(IR::PushI32(*i)),
            IRLocation::I8(i) => self.emit_ir(IR::PushI8(*i)),
            IRLocation::String(s) => self.emit_ir(IR::PushString(s.clone())),

            IRLocation::Storage(storage, size) =>
                self.emit_ir(IR::Push(storage.clone(), *size)),
        }
    }

    pub fn create_label(&mut self, name: &str) -> String
    {
        let label = format!("{}{}", name, self.last_label_id);
        self.last_label_id += 1;
        label
    }

    pub fn emit_null(&mut self) -> Rc<IRValue>
    {
        self.new_value(IRLocation::Null)
    }

    pub fn emit_int(&mut self, i: i32) -> Rc<IRValue>
    {
        self.new_value(IRLocation::I32(i))
    }

    pub fn emit_string(&mut self, s: &str) -> Rc<IRValue>
    {
        self.new_value(IRLocation::String(s.to_owned()))
    }

    pub fn emit_char(&mut self, c: char) -> Rc<IRValue>
    {
        self.new_value(IRLocation::I8(c as i8))
    }

    pub fn emit_extern(&mut self, name: &str)
    {
        let mut output = self.output.borrow_mut();
        output.add_extern(name.to_owned());
    }

    pub fn emit_struct_offset(&mut self, offset: i32, size: usize) -> Rc<IRValue>
    {
        self.new_value(IRLocation::Field(offset as usize, size))
    }

    pub fn emit_label(&mut self, label: &str)
    {
        self.emit_ir(IR::Label(label.to_owned()));
    }

    pub fn emit_struct_data<F>(&mut self, struct_size: usize, field_count: usize, mut field_item: F)
            -> Result<Rc<IRValue>, Box<dyn Error>>
        where F: FnMut(&mut Self, usize) -> Result<(Rc<IRValue>, Rc<IRValue>), Box<dyn Error>>
    {
        let result = self.allocate(struct_size);
        for i in 0..field_count
        {
            let (field, item_value) = field_item(self, i)?;
            let (offset, size) =
                match &field.location
                {
                    IRLocation::Field(offset, size) => (*offset, *size),
                    _ => panic!(),
                };

            let stored_item = self.ensure_storage(item_value);
            self.emit_ir(IR::MoveToOffset(
                offset, result.clone(), stored_item.storage(), size))
        }

        Ok(self.new_value(IRLocation::Storage(result, struct_size)))
    }

    pub fn emit_array_literal<F>(&mut self, item_count: usize, mut compile_item: F, item_size: usize)
            -> Result<Rc<IRValue>, Box<dyn Error>>
        where F: FnMut(&mut Self, usize) -> Result<Rc<IRValue>, Box<dyn Error>>
    {
        let size = item_size * item_count;
        let result = self.allocate(size);

        for i in 0..item_count
        {
            let item_value = compile_item(self, i)?;
            let stored_item = self.ensure_storage(item_value);

            let offset = i * item_size;
            self.emit_ir(IR::MoveToOffset(
                offset, result.clone(), stored_item.storage(), item_size))
        }
        
        Ok(self.new_value(IRLocation::Storage(result, size)))
    }

    pub fn mov(&mut self, to: Rc<IRValue>, from: Rc<IRValue>)
    {
        match &to.location
        {
            IRLocation::Null => panic!(),
            IRLocation::Field(_, _) => panic!(),
            IRLocation::I32(_) => panic!(),
            IRLocation::I8(_) => panic!(),
            IRLocation::String(_) => panic!(),

            IRLocation::Storage(to_storage, _) =>
            {
                match &from.location
                {
                    IRLocation::Null => panic!(),
                    IRLocation::Field(_, _) => panic!(),

                    IRLocation::I32(i) =>
                        self.emit_ir(IR::SetI32(to_storage.clone(), *i)),

                    IRLocation::I8(i) =>
                        self.emit_ir(IR::SetI8(to_storage.clone(), *i)),

                    IRLocation::String(s) =>
                        self.emit_ir(IR::SetString(to_storage.clone(), s.clone())),

                    IRLocation::Storage(from_storage, size) =>
                    {
                        self.emit_ir(IR::Move(
                            to_storage.clone(), from_storage.clone(), *size))
                    },
                }
            },
        }
    }

    fn arithmatic_operation(&mut self,
                            lhs: Rc<IRValue>,
                            rhs: Rc<IRValue>,
                            operation: IROperation)
        -> Rc<IRValue>
    {
        // NOTE: For now we only have i32 operations, so 
        //       size will always be 4.
        let size = 4;
        let result = self.allocate(size);

        let lhs_value = self.ensure_storage(lhs);
        match &rhs.location
        {
            IRLocation::Null => panic!(),
            IRLocation::Field(_, _) => panic!(),
            IRLocation::String(_) => panic!(),
            IRLocation::I8(_) => panic!(),

            IRLocation::I32(i) =>
            {
                self.emit_ir(IR::I32ConstantOperation(
                    operation, result.clone(), lhs_value.storage(), *i))
            },

            IRLocation::Storage(rhs_storage, _) =>
            {
                self.emit_ir(IR::I32Operation(
                    operation, result.clone(), lhs_value.storage(), rhs_storage.clone()))
            },
        }

        self.new_value(IRLocation::Storage(result, size))
    }

    pub fn add(&mut self, lhs: Rc<IRValue>, rhs: Rc<IRValue>) -> Rc<IRValue>
    {
        self.arithmatic_operation(lhs, rhs, IROperation::Add)
    }

    pub fn mul(&mut self, lhs: Rc<IRValue>, rhs: Rc<IRValue>) -> Rc<IRValue>
    {
        self.arithmatic_operation(lhs, rhs, IROperation::Multiply)
    }

    pub fn subtract(&mut self, lhs: Rc<IRValue>, rhs: Rc<IRValue>) -> Rc<IRValue>
    {
        self.arithmatic_operation(lhs, rhs, IROperation::Subtract)
    }

    fn comparison_operation(&mut self,
                            lhs: Rc<IRValue>,
                            rhs: Rc<IRValue>,
                            operation: IROperation)
        -> Rc<IRValue>
    {
        let result = self.allocate(1);
        let lhs_value = self.ensure_storage(lhs);
        match &rhs.location
        {
            IRLocation::Null => panic!(),
            IRLocation::Field(_, _) => panic!(),
            IRLocation::String(_) => panic!(),
            IRLocation::I8(_) => panic!(),

            IRLocation::I32(i) =>
            {
                self.emit_ir(IR::I32ConstantOperation(
                    operation, result.clone(), lhs_value.storage(), *i))
            },

            IRLocation::Storage(rhs_storage, _) =>
            {
                self.emit_ir(IR::I32Operation(
                    operation, result.clone(), lhs_value.storage(), rhs_storage.clone()))
            },
        }

        self.new_value(IRLocation::Storage(result, 1))
    }

    pub fn greater_than(&mut self, lhs: Rc<IRValue>, rhs: Rc<IRValue>) -> Rc<IRValue>
    {
        self.comparison_operation(lhs, rhs, IROperation::GreaterThan)
    }

    pub fn less_than(&mut self, lhs: Rc<IRValue>, rhs: Rc<IRValue>) -> Rc<IRValue>
    {
        self.comparison_operation(lhs, rhs, IROperation::LessThan)
    }

    pub fn ref_of(&mut self, value: Rc<IRValue>) -> Rc<IRValue>
    {
        let result = self.allocate(4);
        self.emit_ir(IR::SetRef(result.clone(), value.storage()));
        self.new_value(IRLocation::Storage(result, 4))
    }

    pub fn deref(&mut self, value: Rc<IRValue>, size: usize) -> Rc<IRValue>
    {
        let result = self.allocate(size);
        self.emit_ir(IR::Deref(result.clone(), value.storage(), size));
        self.new_value(IRLocation::Storage(result, size))
    }

    pub fn access(&mut self, ref_value: Rc<IRValue>, field: Rc<IRValue>) -> Rc<IRValue>
    {
        let (field_offset, field_size) = match &field.location
        {
            IRLocation::Field(offset, size) => (*offset, *size),
            _ => panic!(),
        };

        self.emit_ir(IR::I32ConstantOperation(IROperation::Add,
            ref_value.storage(), ref_value.storage(), field_offset as i32));

        let result = self.allocate(field_size);
        self.emit_ir(IR::Deref(result.clone(), ref_value.storage(), field_size));
        self.new_value(IRLocation::Storage(result, field_size))
    }

    pub fn ret(&mut self, value: Rc<IRValue>, size: usize)
    {
        let stored_value = self.ensure_storage(value);
        self.emit_ir(IR::Return(stored_value.storage(), size));
    }

    pub fn goto(&mut self, label: &str)
    {
        self.emit_ir(IR::Goto(label.to_owned()));
    }

    pub fn goto_if_not(&mut self, label: &str, condition: Rc<IRValue>)
    {
        self.emit_ir(IR::GotoIfNot(label.to_owned(), condition.storage()));
    }

    pub fn call<F>(&mut self,
               function_name: &str,
               argument_count: usize,
               mut compile_argument: F,
               return_size: usize) -> Result<Rc<IRValue>, Box<dyn Error>>
        where F: FnMut(&mut Self, usize) -> Result<(Rc<IRValue>, usize), Box<dyn Error>>
    {
        let big_return_storage = 
            if return_size > 4 { Some(self.allocate(return_size)) }
            else { None };

        let mut total_argument_size = 0;
        for i in (0..argument_count).rev()
        {
            let (argument, size) = compile_argument(self, i)?;
            self.push(argument);
            total_argument_size += size;
        }

        let return_storage = big_return_storage.unwrap_or_else(|| self.allocate(return_size));
        self.emit_ir(IR::Call(
            function_name.to_owned(), return_storage.clone(), return_size));

        self.emit_ir(IR::Pop(total_argument_size));
        Ok(self.new_value(IRLocation::Storage(return_storage, return_size)))
    }

    pub fn start_function(&mut self,
                          function_name: &str,
                          params: impl Iterator<Item = usize>)
        -> Vec<Rc<IRValue>>
    {
        {
            let mut output = self.output.borrow_mut();
            if output.current_function.is_some()
            {
                let function = output.current_function.take().unwrap();
                output.add_function(function);
            }

            output.current_function = Some(IRFunction
            {
                name: function_name.to_owned(),
                code: Vec::new(),
                stack_frame_size: 0,
            });
        }

        let mut param_values = Vec::new();
        let mut last_offset = 0;
        for size in params
        {
            param_values.push(self.new_value(IRLocation::Storage(
                IRStorage::Param(last_offset), size)));
            last_offset += size;
        }
        param_values
    }

    pub fn allocate_local(&mut self, size: usize)
        -> Rc<IRValue>
    {
        let (local, size) =
        {
            let mut output = self.output.borrow_mut();
            match &mut output.current_function
            {
                Some(function) =>
                {
                    function.stack_frame_size += size;
                    (function.stack_frame_size, size)
                },

                None => panic!(),
            }
        };

        self.new_value(IRLocation::Storage(IRStorage::Local(local), size))
    }

}

