use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum EnvError {
    UnknownParam(String)
}

impl Display for EnvError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            EnvError::UnknownParam(param) => write!(f, "unknown param {}", param)
        }
    }
}

impl Error for EnvError {}
