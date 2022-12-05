use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

use crate::{cast, types};
use crate::types::val::{InterpreterError, Value};
use crate::vm::chunk::{Chunk, Constant, Function, OpCode};

#[derive(Default, Clone)]
pub struct CallFrame {
    function: Function,
    ip: usize,
    slots_offset: usize,
}

pub enum FunctionType {
    Function,
    Script,
}

#[derive(Default)]
pub struct VirtualMachine {
    pub call_frames: Vec<CallFrame>,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
}

impl VirtualMachine {
    pub fn init() {}
    pub fn destroy() {}

    fn prepare_interpret(&mut self, func: Function) {
        self.call_frames.push(CallFrame {
            function: func,
            ip: 0,
            slots_offset: 1,
        });
    }

    pub fn interpret(&mut self, function: Function) -> Result<(), InterpreterError> {
        self.prepare_interpret(function);
        self.run()?;
        self.call_frames.pop();
        Ok(())
    }

    fn current_frame(&self) -> CallFrame {
        return (*self.call_frames.last().expect("should exist")).clone();
    }

    fn current_chuck(&self) -> Chunk {
        return self.current_frame().function.chunk;
    }

    fn run(&mut self) -> Result<(), InterpreterError> {
        loop {
            if self.is_done() {
                return Ok(());
            }
            self.step()?;
        }
    }

    fn is_done(&self) -> bool {
        self.call_frames.is_empty() || self.current_frame().ip >= self.current_frame().function.chunk.code.len()
    }

    fn step(&mut self) -> Result<(), InterpreterError> {
        let mut frame = self.current_frame();
        let chuck = self.current_chuck();
        let opt = chuck.code.get(frame.ip).expect("never here").clone();

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
                let val: Value = chuck.constants.get(index).expect("should be exit").clone().into();
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
                self.push(Value::Bool(b > a));
            }
            (OpCode::OpLess, _) => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push(Value::Bool(b < a));
            }
            (OpCode::OpPrint, _) => {
                println!("{:?}", self.pop_stack());
            }
            (OpCode::OpPop, _) => {
                self.pop_stack();
            }
            (OpCode::OpDefineGlobal(index), _) => {
                let value = self.pop_stack();
                let key = cast!(chuck.get_constant(index), Constant::String);

                self.globals.insert(key, value);
            }
            (OpCode::OpGetGlobal(index), _) => {
                let key = cast!(chuck.get_constant(index), Constant::String);
                let val = self.globals.get(key.as_str()).expect("not found in globals").clone();
                self.push(val);
            }
            (OpCode::OpSetGlobal(index), _) => {
                let key = cast!(chuck.get_constant(index), Constant::String);
                let val = self.stack.last().expect("expect last").clone();
                self.globals.insert(key, val);
            }
            (OpCode::OpGetLocal(index), _) => {
                let slots_offset = frame.slots_offset;
                let val = self.stack[slots_offset + index - 1].clone();
                self.push(val)
            }
            (OpCode::OpSetLocal(index), _) => {
                let slots_offset = frame.slots_offset;
                let val = self.stack.last().expect("expect last").clone();
                self.stack[slots_offset + index - 1] = val;
            }
            (OpCode::JumpIfFalse(jump_location), _) => {
                let condition = cast!(self.stack.last().expect("expect last").clone(), Value::Bool);
                if !condition {
                    frame.ip += jump_location;
                }
            }
            (OpCode::Jump(jump_location), _) => {
                frame.ip += jump_location;
            }
            (OpCode::Loop(offset), _) => {
                frame.ip -= offset
            }
            (OpCode::Call(args_count), _) => {
                self.call(self.stack.get(self.stack.len() - args_count).expect("should exit").clone(), args_count)?;
            }
        }

        frame.ip += 1;
        let last = self.call_frames.len() - 1;
        self.call_frames[last] = frame;
        Ok(())
    }
    fn call(&mut self, callee: Value, arg_count: usize) -> Result<(), InterpreterError> {
        match callee {
            Value::LoxFunc(name, _) => {
                match self.current_chuck().find_function(name) {
                    None => panic!("Cannot call not function type"),
                    Some(fx) => {
                        self.call_frames.push(CallFrame {
                            function: fx,
                            ip: 0,
                            slots_offset: self.stack.len() - arg_count,
                        })
                    }
                }
            }
            _ => panic!("can't call")
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
                        match opt {
                            OpCode::OpAdd => {
                                Value::Number(x + y)
                            }
                            OpCode::OpSubtract => {
                                Value::Number(y - x)
                            }
                            OpCode::OpMultiply => {
                                Value::Number(y * x)
                            }
                            OpCode::OpDivide => {
                                Value::Number(y / x)
                            }
                            _ => panic!("type not equal")
                        }
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