use std::collections::HashMap;
use crate::process::environment;
use crate::process::interpreter::Interpreter;
use crate::types::{expr, func, val};

#[derive(Clone, Debug, Default)]
pub struct LoxClass {
    pub name: String,
    pub methods: Vec<val::Value>,
}

impl LoxClass {}

impl func::Callable for LoxClass {
    fn arity(&self, _: &Interpreter) -> usize {
        return 0;
    }

    fn call(&self, interpreter: &mut Interpreter, _: Vec<val::Value>) -> Result<val::Value, val::InterpreterError> {
        let lox_instance = LoxInstance::new(&self);
        let i = interpreter.next_id();
        interpreter.lox_instances.insert(i, lox_instance);
        return Ok(val::Value::LoxInstance(i));
    }
}

#[derive(Clone, Debug)]
pub struct LoxInstance {
    pub class: LoxClass,
    fields: HashMap<String, val::Value>,
}


impl LoxInstance {
    pub fn new(class: &LoxClass) -> Self {
        return Self {
            class: class.clone(),
            fields: HashMap::default(),
        };
    }

    pub fn get(&self, name: &str) -> Option<val::Value> {
        match self.fields.get(name) {
            None => {
                self.get_method(name)
            }
            Some(val) => {
                Some(val.clone())
            }
        }
    }

    fn get_method(&self, name: &str) -> Option<val::Value> {
        let lox_class = &self.class;
        let vec = &lox_class.methods;
        for method in vec {
            match method {
                val::Value::LoxFunc(func_name, id) => {
                    if func_name.as_str() == name {
                        return Some(val::Value::LoxFunc(func_name.to_string(), *id));
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn set(&mut self, name: &str, val: val::Value) {
        self.fields.insert(name.to_string(), val);
    }
}


