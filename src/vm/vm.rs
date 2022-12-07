use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::f32::consts::E;

use log::debug;

use crate::{cast, types};
use crate::types::class::LoxClass;
use crate::types::expr::ExpError;
use crate::types::val::{InterpreterError, Value};
use crate::vm::builtins;
use crate::vm::chunk::{Chunk, Constant, Function, Instance, NativeFunction, OpCode};

#[derive(Default, Clone)]
pub struct CallFrame {
    function: Function,
    ip: usize,
    slots_offset: usize,
}

impl CallFrame {
    fn read_constant(&self, idx: usize) -> Constant {
        self.function.chunk.constants[idx].clone()
    }
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
    id: usize,
}

impl VirtualMachine {
    pub fn init(&mut self) {
        self.globals.insert("clock".to_string(), Value::NativeFunc(NativeFunction {
            arity: 0,
            name: "clock".to_string(),
            func: builtins::clock,
        }));
        self.globals.insert("sleep".to_string(), Value::NativeFunc(NativeFunction {
            arity: 1,
            name: "sleep".to_string(),
            func: builtins::sleep,
        }));
    }
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

        Ok(())
    }

    fn pop_stack_n_times(&mut self, num_to_pop: usize) {
        for _ in 0..num_to_pop {
            self.pop();
        }
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        let last = self.call_frames.len() - 1;
        return &mut self.call_frames[last];
    }
    fn frame(&self) -> &CallFrame {
        return self.call_frames.last().expect("should exist");
    }

    fn current_chuck(&self) -> Chunk {
        return self.frame().clone().function.chunk;
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
        self.call_frames.is_empty() || self.frame().ip >= self.frame().function.chunk.code.len()
    }

    fn next_op_and_advance(&mut self) -> (OpCode, usize) {
        let frame = self.frame_mut();
        let chuck = frame.function.chunk.clone();
        let result = chuck.code.get(frame.ip).expect("never here").clone();
        frame.ip += 1;
        return result;
    }

    fn step(&mut self) -> Result<(), InterpreterError> {
        let opt = self.next_op_and_advance();
        match opt {
            (OpCode::OpReturn, _) => {
                let result = self.pop();

                if self.call_frames.len() <= 1 {
                    self.call_frames.pop();
                    return Ok(());
                }

                let num_to_pop = self.stack.len() - self.frame().slots_offset + self.frame().function.arity;
                self.call_frames.pop();
                self.pop_stack_n_times(num_to_pop);

                self.stack.push(result.clone());
                debug!("return value: {:?}", result.clone())
            }
            (OpCode::OpNegate, _) => {
                let new_value = match self.pop() {
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
                let val: Value = self.frame().read_constant(index).into();
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
                match self.pop() {
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
                let a = self.pop();
                let b = self.pop();
                self.push(Value::Bool(a.eq(&b)));
            }
            (OpCode::OpGreater, _) => {
                let a = self.pop();
                let b = self.pop();
                self.push(Value::Bool(b > a));
            }
            (OpCode::OpLess, _) => {
                let a = self.pop();
                let b = self.pop();
                self.push(Value::Bool(b < a));
            }
            (OpCode::OpPrint, _) => {
                println!("{:?}", self.pop());
            }
            (OpCode::OpPop, _) => {
                self.pop();
            }
            (OpCode::OpDefineGlobal(index), _) => {
                let value = self.pop();
                let key = cast!(self.frame().read_constant(index), Constant::String);

                self.globals.insert(key, value);
            }
            (OpCode::OpGetGlobal(index), _) => {
                let key = cast!(self.frame().read_constant(index), Constant::String);
                let val = self.globals.get(key.as_str()).expect("not found in globals").clone();
                self.push(val);
            }
            (OpCode::OpSetGlobal(index), _) => {
                let key = cast!(self.frame().read_constant(index), Constant::String);
                let val = self.stack.last().expect("expect last").clone();
                self.globals.insert(key, val);
            }
            (OpCode::OpGetLocal(index), _) => {
                let slots_offset = self.frame().slots_offset;
                let val = self.stack[slots_offset + index].clone();
                self.push(val)
            }
            (OpCode::OpSetLocal(index), _) => {
                let slots_offset = self.frame().slots_offset;
                let val = self.stack.last().expect("expect last").clone();
                self.stack[slots_offset + index] = val;
            }
            (OpCode::JumpIfFalse(jump_location), _) => {
                let last = self.stack.len() - 1;
                let condition = cast!(self.stack[last].clone(), Value::Bool);
                if !condition {
                    self.frame_mut().ip += jump_location;
                }
            }
            (OpCode::Jump(jump_location), _) => {
                self.frame_mut().ip += jump_location;
            }
            (OpCode::Loop(offset), _) => {
                self.frame_mut().ip -= offset
            }
            (OpCode::Call(args_count), _) => {
                self.call(self.stack.get(self.stack.len() - args_count - 1).expect("should exit").clone(), args_count)?;
                debug!("call function, increment call frame");
            }
            (OpCode::OpClass(clazz), _) => {
                self.push(Value::Class(clazz))
            }
            (OpCode::OpSetProperty(name), _) => {
                let mut instance = cast!(self.peek(1), Value::Instance);
                let val = self.peek(0);
                self.pop();
                self.pop();
                instance.fields.insert(name, val.clone());
                self.push(val);
                self.update_ref(Value::Instance(instance));
            }
            (OpCode::OpGetProperty(name), _) => {
                let instance = cast!(self.peek(0), Value::Instance);
                match instance.fields.get(name.as_str()) {
                    None => {}
                    Some(val) => {
                        self.pop();
                        self.push(val.clone());
                        return Ok(());
                    }
                }
                return Err(InterpreterError::SimpleError(format!("not found property {}", name)));
            }
        }
        Ok(())
    }

    fn peek(&self, n: usize) -> Value {
        self.stack[self.stack.len() - 1 - n].clone()
    }

    fn update_ref(&mut self, val: Value) {
        let instance = cast!(val.clone(), Value::Instance);

        for v in self.globals.values_mut() {
            match v {
                Value::Instance(ins) => {
                    if ins.id == instance.id {
                        ins.fields = instance.fields.clone()
                    }
                }
                _ => {}
            }
        }


        for i in 0..self.stack.len() {
            let item = &self.stack[i];
            match item {
                Value::Instance(ins) => {
                    if ins.id == instance.id {
                        self.stack[i] = val.clone();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn find_function(&self, name: String) -> Option<Function> {
        for i in (0..self.call_frames.len()).rev() {
            let call_frame = &self.call_frames[i];
            for constant in &call_frame.function.chunk.constants {
                match constant {
                    Constant::Function(f) => {
                        if f.name.eq(&name) {
                            return Some(f.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        return None;
    }

    fn next_id(&mut self) -> usize {
        self.id += 1;
        return self.id;
    }

    fn call(&mut self, callee: Value, arg_count: usize) -> Result<(), InterpreterError> {
        match callee {
            Value::Class(clazz) => {
                let new_instance = Instance {
                    id: self.next_id(),
                    class: clazz,
                    fields: Default::default(),
                };

                let index = self.stack.len() - 1 - arg_count;
                self.stack[index] = Value::Instance(new_instance);
            }
            Value::LoxFunc(name, _) => {
                match self.find_function(name) {
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
            Value::NativeFunc(native) => {
                let mut values = vec![];
                for _ in 0..native.arity {
                    values.push(self.pop());
                }
                values.reverse();
                // native function value
                self.pop();

                let result = (native.func)(self, values.as_slice())?;
                self.push(result);
            }
            _ => panic!("can't call")
        }

        Ok(())
    }


    pub fn pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(val) => val,
            None => panic!("attempted to pop empty stack!"),
        }
    }
    pub fn push(&mut self, var: Value) {
        self.stack.push(var);
    }

    fn binary_opt(&mut self, opt: OpCode) {
        let x = self.pop();
        let y = self.pop();

        debug!("call binary opt: {:?}, x: {:?} y: {:?}", opt,x, y);

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