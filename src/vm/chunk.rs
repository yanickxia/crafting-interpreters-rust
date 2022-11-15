use crate::types::val::Value;

#[derive(Debug, Clone)]
pub enum OpCode {
    OpReturn,
    OpConstant(usize),
    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
}

#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64)
}

impl Into<Value> for Constant {
    fn into(self) -> Value {
        return match self {
            Constant::Number(val) => {
                Value::Number(val)
            }
        };
    }
}

#[derive(Clone, Default)]
pub struct Chunk {
    pub code: Vec<(OpCode, usize)>,
    pub constants: Vec<Constant>,
}

impl Chunk {
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
            _ => format!("Unknown opcode {:?}", opt)
        };
        println!("{0: <04}   {1: <50} line {2: <50}", index, formatted_op, lineno)
    }
}