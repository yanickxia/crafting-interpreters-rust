use std::collections::HashMap;

use crate::types::{env, val};

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, val::Value>,
}


impl Environment {
    pub fn define(&mut self, name: String, var: &val::Value) {
        self.values.insert(name.clone(), var.clone());
    }

    pub fn get(&self, name: &str) -> Option<&val::Value> {
        return self.values.get(name);
    }

    pub fn assign(&mut self, name: String, var: &val::Value) -> Result<(), env::EnvError> {
        if !self.values.contains_key(name.as_str()) {
            return Err(env::EnvError::UnknownParam(name.clone()));
        }
        self.values.insert(name.clone(), var.clone());
        return Ok(());
    }
}