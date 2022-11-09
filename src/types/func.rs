use std::collections::HashMap;

use crate::process::{environment, interpreter};
use crate::types::{expr, val};

pub trait Callable {
    fn arity(&self, interpreter: &interpreter::AstInterpreter) -> usize;
    fn call(&self, interpreter: &mut interpreter::AstInterpreter, args: Vec<val::Value>) -> Result<val::Value, val::InterpreterError>;
}


#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub id: usize,
    pub name: String,
    pub parameters: Vec<String>,
    pub body: expr::Statement,
    // lambda 闭包
    // pub closure: environment::Environment,
}

impl Callable for LoxFunction {
    fn arity(&self, interpreter: &interpreter::AstInterpreter) -> usize {
        return self.parameters.len();
    }

    fn call(&self, interpreter: &mut interpreter::AstInterpreter, args: Vec<val::Value>) -> Result<val::Value, val::InterpreterError> {
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
        let mut new_env = environment::Environment::with_enclosing(interpreter.environment.clone());
        new_env.values.extend(args_env);

        interpreter.environment = new_env;
        println!("excute env {:?}", &interpreter.environment);
        let ret = interpreter.execute(&self.body)?;
        interpreter.environment = saved_env;
        Ok(ret)
    }
}