use std::collections::HashMap;

use crate::process::{environment, interpreter};
use crate::types::{class, expr, val};

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
    pub bind: Option<usize>,
}

impl LoxFunction {
    pub fn bind_instance(&mut self, instance: val::Value) -> Result<(), val::InterpreterError> {
        return match instance {
            val::Value::LoxInstance(id) => {
                self.bind = Some(id);
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

        match self.bind {
            None => {}
            Some(instance_id) => {
                new_env.values.insert("this".to_string(), val::Value::LoxInstance(instance_id));
            }
        }

        interpreter.environment = new_env;
        interpreter.execute(&self.body)?;
        interpreter.environment = saved_env;

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