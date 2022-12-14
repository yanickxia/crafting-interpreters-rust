use std::collections::HashMap;
use crate::process::environment;
use crate::process::interpreter::Interpreter;
use crate::types::{expr, func, val};

#[derive(Clone, Debug, Default)]
pub struct LoxClass {
    pub name: String,
    pub methods: Vec<val::Value>,
    pub super_class: Option<Box<LoxClass>>,
}

impl LoxClass {
    pub fn find_method(&self, name: String) -> Option<val::Value> {
        for method in &self.methods {
            match method {
                val::Value::LoxFunc(func_name, _) => {
                    if func_name.as_str() == name.as_str() {
                        return Some(method.clone());
                    }
                }
                _ => {}
            }
        }

        match &self.super_class {
            None => {}
            Some(sc) => {
                return sc.find_method(name);
            }
        }

        None
    }
}

impl func::Callable for LoxClass {
    fn arity(&self, inter: &Interpreter) -> usize {
        match self.find_method("init".to_string()) {
            None => {
                return 0;
            }
            Some(init) => {
                match init {
                    val::Value::LoxFunc(_, ref func_id) => {
                        let func = inter.lox_functions.get(func_id).unwrap();
                        return func.parameters.len();
                    }
                    _ => {
                        panic!("should be lox")
                    }
                }
            }
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<val::Value>) -> Result<val::Value, val::InterpreterError> {
        let lox_instance = LoxInstance::new(&self);
        let id = interpreter.next_id();
        interpreter.lox_instances.insert(id, lox_instance);


        let mut parent_id = None;
        if self.super_class.is_some() {
            let parent = self.super_class.as_ref().unwrap().call(interpreter, arguments.clone())?;
            match parent {
                val::Value::LoxInstance {
                    id, ..
                } => {
                    parent_id = Some(id)
                }
                _ => {}
            }
        }

        let func = self.find_method("init".to_string());
        match func {
            None => {}
            Some(func) => {
                match func {
                    val::Value::LoxFunc(_, ref func_id) => {
                        let mut func = interpreter.lox_functions.get_mut(func_id).unwrap().clone();
                        func.bind = Some(val::Value::LoxInstance {
                            id,
                            parent: parent_id,
                        });
                        func.call(interpreter, arguments)?;
                        func.is_initializer = true;
                    }
                    _ => {}
                }
            }
        }


        return Ok(val::Value::LoxInstance {
            id,
            parent: parent_id,
        });
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
        return lox_class.find_method(name.to_string());
    }

    pub fn set(&mut self, name: &str, val: val::Value) {
        self.fields.insert(name.to_string(), val);
    }
}


