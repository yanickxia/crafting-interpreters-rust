use std::error::Error;
use std::fmt::{Display, Formatter};

pub type RunResult<T> = Result<T, Box<dyn Error>>;


#[derive(Debug)]
pub struct RunError {
    line: usize,
    message: String,
}

impl Display for RunError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "at: {}, case: {}", &self.line, &self.message)
    }
}

impl Error for RunError {}

pub fn new_error(line: usize, message: String) -> Box<dyn Error> {
    return Box::new(RunError { line, message });
}
