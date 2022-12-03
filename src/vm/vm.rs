use std::collections::HashMap;

use crate::{cast, types};
use crate::types::val::{InterpreterError, Value};
use crate::vm::chunk::{Chunk, Constant, OpCode};

#[derive(Default)]
pub struct VirtualMachine {
    pub current: Chunk,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
}

impl VirtualMachine {
    pub fn init() {}
    pub fn destroy() {}

    pub fn interpret(&mut self, chuck: &Chunk) -> Result<(), InterpreterError> {
        self.current = chuck.clone();
        self.run()
    }

    fn run(&mut self) -> Result<(), InterpreterError> {
        self.step()
    }

    fn step(&mut self) -> Result<(), InterpreterError> {
        for i in 0..self.current.code.len() {
            let opt = self.current.code.get(i).expect("never here").clone();
            match opt {
                (OpCode::OpReturn, _) => {
                    println!("{:?}", self.pop_stack());
                }
                (OpCode::OpNegate, _) => {
                    let new_value = match self.pop_stack() {
                        Value::Number(val) => {
                            Value::Number(-val)
                        }
                        _ => {
                            panic!("can't negate")
                        }
                    };
                    self.push(new_value);
                }
                (OpCode::OpConstant(index), _) => {
                    let val: Value = self.current.constants.get(index).expect("should be exit").clone().into();
                    self.push(val);
                }
                (OpCode::OpAdd, _) | (OpCode::OpSubtract, _) | (OpCode::OpMultiply, _) | (OpCode::OpDivide, _) => {
                    self.binary_opt(opt.0.clone())
                }
                (OpCode::OpNil, _) => {
                    self.push(Value::Nil)
                }
                (OpCode::OpTrue, _) => {
                    self.push(Value::Bool(true))
                }
                (OpCode::OpFalse, _) => {
                    self.push(Value::Bool(false))
                }
                (OpCode::OpNot, _) => {
                    match self.pop_stack() {
                        Value::Bool(b) => {
                            self.push(Value::Bool(!b))
                        }
                        Value::Nil => {
                            self.push(Value::Bool(true))
                        }
                        _ => panic!("not execute opt not")
                    }
                }
                (OpCode::OpEqual, _) => {
                    let a = self.pop_stack();
                    let b = self.pop_stack();
                    self.push(Value::Bool(a.eq(&b)));
                }
                (OpCode::OpGreater, _) => {
                    let a = self.pop_stack();
                    let b = self.pop_stack();
                    self.push(Value::Bool(a > b));
                }
                (OpCode::OpLess, _) => {
                    let a = self.pop_stack();
                    let b = self.pop_stack();
                    self.push(Value::Bool(a < b));
                }
                (OpCode::OpPrint, _) => {
                    println!("{:?}", self.pop_stack());
                }
                (OpCode::OpPop, _) => {
                    self.pop_stack();
                }
                (OpCode::OpDefineGlobal(index), _) => {
                    let value = self.pop_stack();
                    let key = cast!(self.current.get_constant(index), Constant::String);

                    self.globals.insert(key, value);
                }
                (OpCode::OpGetGlobal(index), _) => {
                    let key = cast!(self.current.get_constant(index), Constant::String);
                    let val = self.globals.get(key.as_str()).expect("not found in globals").clone();
                    self.push(val);
                }
                (OpCode::OpSetGlobal(index), _) => {
                    let key = cast!(self.current.get_constant(index), Constant::String);
                    let val = self.stack.last().expect("expect last").clone();
                    self.globals.insert(key, val);
                }
                (OpCode::OpGetLocal(index), _) => {
                    self.push(self.stack[index].clone())
                }
                (OpCode::OpSetLocal(index), _) => {
                    let val = self.stack.last().expect("expect last").clone();
                    self.stack[index] = val;
                }
            }
        }
        Ok(())
    }

    pub fn pop_stack(&mut self) -> Value {
        match self.stack.pop() {
            Some(val) => val,
            None => panic!("attempted to pop empty stack!"),
        }
    }
    pub fn push(&mut self, var: Value) {
        self.stack.push(var);
    }

    fn binary_opt(&mut self, opt: OpCode) {
        let x = self.pop_stack();
        let y = self.pop_stack();

        let new_value = match x {
            Value::Number(x) => {
                match y {
                    Value::Number(y) => {
                        Value::Number(x + y)
                    }
                    _ => panic!("type not equal")
                }
            }
            Value::String(x) => {
                match y {
                    Value::String(y) => {
                        Value::String(y + x.as_str())
                    }
                    _ => panic!("type not equal")
                }
            }
            _ => panic!("not support binary opt")
        };

        self.push(new_value)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::val::Value;
    use crate::vm::chunk::{Chunk, Constant, OpCode};
    use crate::vm::vm::VirtualMachine;

    #[test]
    fn it_works() {
        let mut machine = VirtualMachine::default();
        let mut chuck = Chunk::default();
        let i = chuck.add_constant(Constant::Number(12.0));
        let j = chuck.add_constant(Constant::Number(24.0));
        chuck.code.push((OpCode::OpConstant(i), 1));
        chuck.code.push((OpCode::OpConstant(j), 2));
        chuck.code.push((OpCode::OpAdd, 3));

        machine.current = chuck;
        machine.step().expect("TODO: panic message");
        assert_eq!(machine.stack.get(0).unwrap().clone(), Value::Number(36.0));
    }
}