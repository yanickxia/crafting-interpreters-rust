use std::collections::HashMap;

use crate::process::{environment, interpreter};
use crate::types::{expr, val};
use crate::types::class::LoxClass;

pub trait Callable {
    fn arity(&self, interpreter: &interpreter::Interpreter) -> usize;
    fn call(&self, interpreter: &mut interpreter::Interpreter, args: Vec<val::Value>) -> Result<val::Value, val::InterpreterError>;
}


#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub id: usize,
    pub name: String,
    pub parameters: Vec<String>,
    pub body: expr::Statement,
    pub closure: environment::Environment,
    pub bind: Option<val::Value>,
    pub is_initializer: bool,
}

impl LoxFunction {
    pub fn bind_instance(&mut self, instance: val::Value) -> Result<(), val::InterpreterError> {
        return match instance {
            val::Value::LoxInstance {
                ..
            } => {
                self.bind = Some(instance.clone());
                Ok(())
            }
            _ => {
                Err(val::InterpreterError::SimpleError("shoud bind instance".to_string()))
            }
        };
    }
}

impl Callable for LoxFunction {
    fn arity(&self, interpreter: &interpreter::Interpreter) -> usize {
        return self.parameters.len();
    }

    fn call(&self, interpreter: &mut interpreter::Interpreter, args: Vec<val::Value>) -> Result<val::Value, val::InterpreterError> {
        let args_env: HashMap<_, _> = self
            .parameters
            .iter()
            .zip(args.iter())
            .map(|(param, arg)| {
                (
                    param.clone(),
                    (
                        arg.clone()
                    ),
                )
            })
            .collect();

        let saved_env = interpreter.environment.clone();
        let mut new_env = environment::Environment::with_enclosing(self.closure.clone());
        new_env.values.extend(args_env);

        match &self.bind {
            None => {}
            Some(bind) => {
                match bind {
                    val::Value::LoxInstance {
                        id, parent
                    } => {
                        new_env.values.insert("this".to_string(), val::Value::LoxInstance {
                            id: *id,
                            parent: parent.clone(),
                        });

                        match parent {
                            Some(p) => {
                                new_env.values.insert("super".to_string(), val::Value::LoxInstance {
                                    id: *p,
                                    parent: None,
                                });
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        interpreter.environment = new_env;
        interpreter.execute(&self.body)?;
        interpreter.environment = saved_env;

        if self.is_initializer {
            return Ok(self.bind.as_ref().unwrap().clone());
        }

        return match interpreter.ret.clone() {
            None => {
                Ok(val::Value::Nil)
            }
            Some(ret) => {
                interpreter.ret = None;
                Ok(ret)
            }
        };
    }
}