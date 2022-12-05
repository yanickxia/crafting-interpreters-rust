use crate::types::val::Value;

#[derive(Default, Clone, Debug)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
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
    JumpIfFalse(usize),
    Jump(usize),
    Loop(usize),
    Call(usize),
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

    pub fn find_function(&self, name: String) -> Option<Function> {
        for constant in &self.constants {
            match constant {
                Constant::Function(f) => {
                    if f.name.eq(&name) {
                        return Some(f.clone());
                    }
                }
                _ => {}
            }
        }

        return None;
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
            OpCode::OpConstant(const_idx) => format!(
                "OP_CONSTANT {:?} (idx={})",
                self.constants[*const_idx], *const_idx
            ),
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
            OpCode::JumpIfFalse(offset) => format!("JUMP_IF_FALSE: {}", offset),
            OpCode::Jump(offset) => format!("JUMP: {}", offset),
            OpCode::Loop(offset) => format!("LOOP: {}", offset),
            OpCode::Call(count) => format!("CALL: {}", count),
        };
        println!("{0: <04}   {1: <50} line {2: <50}", index, formatted_op, lineno)
    }
}