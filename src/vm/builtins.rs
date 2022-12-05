use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cast;
use crate::types::expr::ExpError;
use crate::types::val::{InterpreterError, Value};
use crate::vm::vm::VirtualMachine;

pub fn clock(
    _vm: &mut VirtualMachine,
    _args: &[Value],
) -> Result<Value, InterpreterError> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    Ok(Value::Number(since_the_epoch.as_millis() as f64))
}

pub fn sleep(
    _vm: &mut VirtualMachine,
    _args: &[Value],
) -> Result<Value, InterpreterError> {
    let secs = cast!(_args[0], Value::Number);

    thread::sleep(Duration::from_secs(secs as u64));
    Ok(Value::Nil)
}