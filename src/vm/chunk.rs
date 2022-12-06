use std::fmt::{Debug, Formatter};

use crate::types::val::{InterpreterError, UpValue, Value};
use crate::vm::compiler;
use crate::vm::vm::VirtualMachine;

#[derive(Default, Clone, Debug)]
pub struct Closure {
    pub function: Function,
    pub up_values: Vec<UpValue>,
}

#[derive(Default, Clone, Debug)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
}

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub name: String,
    pub func: fn(&mut VirtualMachine, &[Value]) -> Result<Value, InterpreterError>,
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeFunction({})", self.name)
    }
}

#[derive(Debug, Clone)]
pub enum OpCode {
    OpReturn,
    OpConstant(usize),
    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNil,
    OpTrue,
    OpFalse,
    OpNot,
    OpEqual,
    OpGreater,
    OpLess,
    OpPrint,
    OpPop,
    OpDefineGlobal(usize),
    OpGetGlobal(usize),
    OpSetGlobal(usize),
    OpGetLocal(usize),
    OpSetLocal(usize),
    OpJumpIfFalse(usize),
    OpJump(usize),
    OpLoop(usize),
    OpCall(usize),
    OpClosure(Vec<compiler::UpValue>),
    OpGetUpValue(usize),
    OpSetUpValue(usize),
}


#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    Bool(bool),
    String(String),
    Function(Function),
    Nil,
}


#[derive(Clone, Default, Debug)]
pub struct Chunk {
    pub code: Vec<(OpCode, usize)>,
    pub constants: Vec<Constant>,
}

impl Chunk {
    pub fn get_constant(&self, index: usize) -> Constant {
        let constant = self.constants[index].clone();
        return constant;
    }

    pub fn add_constant(&mut self, val: Constant) -> usize {
        let constants_index = self.constants.len();
        self.constants.push(val);
        return constants_index;
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for i in 0..self.code.len() {
            self.disassemble_instruction(i)
        }
    }

    pub fn disassemble_instruction(&self, index: usize) {
        let (opt, lineno) = self.code.get(index).expect("want instruction");
        let formatted_op = match opt {
            OpCode::OpReturn => "OP_RETURN".to_string(),
            OpCode::OpConstant(const_idx) => {
                let constant = self.constants[*const_idx].clone();

                return match constant {
                    Constant::Function(func) => {
                        func.chunk.disassemble(func.name.as_str());
                        println!("== {} ==", func.name.as_str());
                        "".to_string();
                    }
                    _ => {
                        format!(
                            "OP_CONSTANT {:?} (idx={})",
                            constant.clone(), *const_idx
                        );
                    }
                };
            }
            OpCode::OpNil => "OP_NIL".to_string(),
            OpCode::OpTrue => "OP_TRUE".to_string(),
            OpCode::OpFalse => "OP_FALSE".to_string(),
            OpCode::OpNot => "OP_NOT".to_string(),
            OpCode::OpNegate => "OP_NEGATE".to_string(),
            OpCode::OpAdd => "OP_ADD".to_string(),
            OpCode::OpSubtract => "OP_SUB".to_string(),
            OpCode::OpMultiply => "OP_MUL".to_string(),
            OpCode::OpDivide => "OP_DIV".to_string(),
            OpCode::OpEqual => "OP_EQUAL".to_string(),
            OpCode::OpGreater => "OP_GREATER".to_string(),
            OpCode::OpLess => "OP_LESS".to_string(),
            OpCode::OpPrint => "OP_PRINT".to_string(),
            OpCode::OpPop => "OP_POP".to_string(),
            OpCode::OpDefineGlobal(index) => format!("OP_DEF_GLOBAL: {}", index),
            OpCode::OpGetGlobal(index) => format!("OP_GET_GLOBAL: {:?}", self.constants[*index]),
            OpCode::OpSetGlobal(index) => format!("OP_SET_GLOBAL: {:?}", self.constants[*index]),
            OpCode::OpGetLocal(index) => format!("OP_GET_LOCAL: {}", index),
            OpCode::OpSetLocal(index) => format!("OP_SET_LOCAL: {}", index),
            OpCode::OpJumpIfFalse(offset) => format!("OP_JUMP_IF_FALSE: {}", offset),
            OpCode::OpJump(offset) => format!("OP_JUMP: {}", offset),
            OpCode::OpLoop(offset) => format!("OP_LOOP: {}", offset),
            OpCode::OpCall(count) => format!("OP_CALL: ARGS_SIZE {}", count),
            OpCode::OpClosure(cls) => format!("OP_CLOSURE "),
            OpCode::OpGetUpValue(index) => format!("OP_GET_UP_VALUE: {}", index),
            OpCode::OpSetUpValue(index) => format!("OP_SET_UP_VALUE: {}", index),
        };
        println!("{0: <04}   {1: <50} line {2: <50}", index, formatted_op, lineno)
    }
}