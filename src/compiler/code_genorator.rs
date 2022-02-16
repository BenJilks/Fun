use std::rc::Rc;
use std::error::Error;

pub trait CodeGenortator<Value>
{

    fn create_label(&mut self, name: &str) -> String;

    fn emit_null(&mut self) -> Rc<Value>;
    fn emit_int(&mut self, i: i32) -> Result<Rc<Value>, Box<dyn Error>>;
    fn emit_string(&mut self, s: &str) -> Result<Rc<Value>, Box<dyn Error>>;
    fn emit_char(&mut self, c: char) -> Result<Rc<Value>, Box<dyn Error>>;
    fn emit_extern(&mut self, name: &str);
    fn emit_struct_offset(&mut self, offset: i32, size: usize) -> Rc<Value>;
    fn emit_label(&mut self, label: &str) -> Result<(), Box<dyn Error>>;

    fn emit_struct_data<F>(&mut self, struct_size: usize, field_count: usize, field_item: F)
            -> Result<Rc<Value>, Box<dyn Error>>
        where F: FnMut(&mut Self, usize) -> Result<(Rc<Value>, Rc<Value>), Box<dyn Error>>;

    fn emit_array_literal<F>(&mut self, item_count: usize, compile_item: F, item_size: usize)
            -> Result<Rc<Value>, Box<dyn Error>>
        where F: FnMut(&mut Self, usize) -> Result<Rc<Value>, Box<dyn Error>>;

    fn mov(&mut self, to: Rc<Value>, from: Rc<Value>, size: usize) -> Result<(), Box<dyn Error>>;
    fn add(&mut self, lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>, Box<dyn Error>>;
    fn mul(&mut self, lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>, Box<dyn Error>>;
    fn subtract(&mut self, lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>, Box<dyn Error>>;
    fn greater_than(&mut self, lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>, Box<dyn Error>>;
    fn less_than(&mut self, lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>, Box<dyn Error>>;

    fn ref_of(&mut self, value: Rc<Value>) -> Result<Rc<Value>, Box<dyn Error>>;
    fn deref(&mut self, value: Rc<Value>, size: usize) -> Result<Rc<Value>, Box<dyn Error>>;
    fn access(&mut self, value: Rc<Value>, field: Rc<Value>) -> Result<Rc<Value>, Box<dyn Error>>;
    fn ret(&mut self, value: Rc<Value>, size: usize, to: Option<Rc<Value>>) -> Result<(), Box<dyn Error>>;
    fn goto(&mut self, label: &str) -> Result<(), Box<dyn Error>>;
    fn goto_if_not(&mut self, label: &str, condition: Rc<Value>) -> Result<(), Box<dyn Error>>;

    fn call<F>(&mut self, function_name: &str,
               argument_count: usize,
               compile_argument: F,
               return_size: usize) -> Result<Rc<Value>, Box<dyn Error>>
        where F: FnMut(&mut Self, usize) -> Result<(Rc<Value>, usize), Box<dyn Error>>;

    fn start_function(&mut self, function_name: &str,
                      params: impl Iterator<Item = usize>)
        -> Result<Vec<Rc<Value>>, Box<dyn Error>>;

    fn allocate_local(&mut self, size: usize)
        -> Result<Rc<Value>, Box<dyn Error>>;

}

