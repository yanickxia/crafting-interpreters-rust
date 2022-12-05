use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::types::{class, expr};
use crate::vm::chunk::Constant;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    LoxFunc(String, usize),
    LoxClass(class::LoxClass),
    LoxInstance {
        id: usize,
        parent: Option<usize>,
    },

    // for fast return
    Ret(Box<Value>),
}

impl From<Constant> for Value {
    fn from(c: Constant) -> Self {
        return match c {
            Constant::Number(f) => {
                Value::Number(f)
            }
            Constant::Bool(b) => {
                Value::Bool(b)
            }
            Constant::Nil => {
                Value::Nil
            }
            Constant::String(s) => {
                Value::String(s)
            }
            Constant::Function(func) => {
                Value::LoxFunc(func.name, func.arity)
            }
        };
    }
}


impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return match self {
            Value::Number(x) => {
                match other {
                    Value::Number(y) => {
                        x.partial_cmp(y)
                    }
                    _ => { None }
                }
            }
            Value::String(x) => {
                match other {
                    Value::String(y) => {
                        x.partial_cmp(y)
                    }
                    _ => { None }
                }
            }
            Value::Bool(_) => {
                match other {
                    _ => { None }
                }
            }
            Value::Nil => {
                match other {
                    _ => { None }
                }
            }
            _ => {
                None
            }
        };
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        return match self {
            Value::Number(x) => {
                match other {
                    Value::Number(y) => {
                        x == y
                    }
                    _ => { false }
                }
            }
            Value::String(x) => {
                match other {
                    Value::String(y) => {
                        x.as_str() == y.as_str()
                    }
                    _ => { false }
                }
            }
            Value::Bool(x) => {
                match other {
                    Value::Bool(y) => {
                        x == y
                    }
                    _ => { false }
                }
            }
            Value::Nil => {
                match other {
                    Value::Nil => {
                        true
                    }
                    _ => { false }
                }
            }
            _ => {
                false
            }
        };
    }
}


#[derive(Debug)]
pub enum InterpreterError {
    TypeNotMatch {
        expected: String,
        found: Value,
    },
    OperatorNotMatch {
        left: Value,
        right: Value,
        opt: expr::BinaryOperatorType,
    },
    MissVariable {
        name: String
    },
    ExecuteError(Box<InterpreterError>),
    SimpleError(String),
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::SimpleError(message) => write!(
                f,
                "Simple Error: {}",
                message
            ),
            InterpreterError::TypeNotMatch { expected, found } => write!(
                f,
                "Expected {:?} but found {:?}",
                expected, found
            ),
            InterpreterError::OperatorNotMatch { left, right, opt } => write!(
                f,
                "Left {:?} Right {:?} Operator {:?}, not match",
                left, right, opt
            ),
            InterpreterError::MissVariable { name } => write!(
                f,
                "miss param name {}",
                name),
            InterpreterError::ExecuteError(inner) => write!(
                f,
                "ExecuteError, case {:?}",
                inner
            )
        }
    }
}

impl Error for InterpreterError {}