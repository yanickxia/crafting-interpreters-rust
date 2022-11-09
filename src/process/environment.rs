use std::collections::HashMap;

use crate::types::{env, val};

#[derive(Default, Clone)]
pub struct Environment {
    values: HashMap<String, val::Value>,
    enclosing: Option<Box<Environment>>,
}


impl Environment {
    pub fn with_enclosing(env: &Environment) -> Self {
        return Self {
            values: Default::default(),
            enclosing: Some(Box::new(env.clone())),
        };
    }

    pub fn define(&mut self, name: String, var: &val::Value) {
        self.values.insert(name.clone(), var.clone());
    }

    pub fn get(&self, name: &str) -> Option<&val::Value> {
        return match self.values.get(name) {
            None => {
                match &self.enclosing {
                    None => {
                        None
                    }
                    Some(parent) => {
                        parent.get(name)
                    }
                }
            }
            Some(val) => {
                Some(val)
            }
        };
    }

    pub fn assign(&mut self, name: String, var: &val::Value) -> Result<(), env::EnvError> {
        if self.values.contains_key(name.as_str()) {
            self.values.insert(name.clone(), var.clone());
            return Ok(());
        }

        return match &mut self.enclosing {
            None => {
                Err(env::EnvError::UnknownParam(name.clone()))
            }
            Some(parent) => {
                parent.assign(name, var)
            }
        };
    }
}