use crate::types;
use crate::types::val::{InterpreterError, Value};
use crate::vm::chunk::{Chunk, OpCode};

#[derive(Default)]
pub struct VirtualMachine {
    pub current: Chunk,
    pub stack: Vec<Value>,
}

impl VirtualMachine {
    pub fn init() {}
    pub fn destroy() {}

    pub fn interpret(&mut self, chuck: &Chunk) -> Result<Value, InterpreterError> {
        self.current = chuck.clone();
        self.run()?;
        Ok(self.stack.pop().unwrap())
    }

    fn run(&mut self) -> Result<(), InterpreterError> {
        self.step()
    }

    fn step(&mut self) -> Result<(), InterpreterError> {
        for i in 0..self.current.code.len() {
            let opt = self.current.code.get(i).expect("xxx");
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
                    let val: Value = self.current.constants.get(*index).expect("should be exit").clone().into();
                    self.push(val);
                }
                (OpCode::OpAdd, _) | (OpCode::OpSubtract, _) | (OpCode::OpMultiply, _) | (OpCode::OpDivide, _) => {
                    self.binary_opt(opt.0.clone())
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
        let x = match self.pop_stack() {
            Value::Number(val) => {
                val
            }
            _ => {
                panic!("can't add")
            }
        };

        let y = match self.pop_stack() {
            Value::Number(val) => {
                val
            }
            _ => {
                panic!("can't add")
            }
        };


        let new_value = match opt {
            OpCode::OpAdd => {
                Value::Number(x + y)
            }
            OpCode::OpSubtract => {
                Value::Number(x - y)
            }
            OpCode::OpMultiply => {
                Value::Number(x * y)
            }
            OpCode::OpDivide => {
                Value::Number(x / y)
            }
            _ => {
                panic!("not binary opt")
            }
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