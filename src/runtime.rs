use std::{fs, io};
use std::error::Error;
use std::io::BufRead;

use crate::process::{interpreter, parser, scanner};
use crate::process::interpreter::Interpreter;
use crate::types::expr::ExpError;
use crate::types::val::{InterpreterError, Value};
use crate::vm::{compiler, vm};
use crate::vm::chunk::Constant;
use crate::vm::vm::FunctionType;

pub struct VMRuntime {
    had_error: bool,
    vm: vm::VirtualMachine,
    pub disassemble: bool,
}


impl Default for VMRuntime {
    fn default() -> Self {
        let mut machine = vm::VirtualMachine::default();
        machine.init();
        return VMRuntime {
            had_error: false,
            vm: machine,
            disassemble: false,
        };
    }
}

impl VMRuntime {
    pub fn run_file(&mut self, file_name: String) {
        let all_file = fs::read_to_string(file_name).expect("read file error");
        self.run(all_file);
        if self.had_error {
            std::process::exit(65);
        }
    }

    fn run(&mut self, file: String) {
        let tokens = scanner::scan_tokens(file);
        let mut compiler = compiler::Compiler::new(tokens.unwrap(), FunctionType::Script);
        match compiler.compile() {
            Ok(func) => {
                if self.disassemble {
                    func.chunk.disassemble("main");
                    return;
                }
                match self.vm.interpret(func) {
                    Ok(_) => {
                        let mut i = self.vm.stack.len();
                        loop {
                            let v = self.vm.stack.pop();
                            if v.is_none() {
                                break;
                            }
                            i -= 1;
                            println!("stack #{}: value {:?}", i, v)
                        }
                    }
                    Err(e) => {
                        self.report(Box::new(e))
                    }
                }
            }
            Err(e) => {
                self.report(Box::new(e))
            }
        }
    }

    fn report(&mut self, err: Box<dyn Error>) {
        println!("{}", err);
        self.had_error = true;
    }
}


pub struct Runtime {
    had_error: bool,
    interpreter: Interpreter,

}

impl Default for Runtime {
    fn default() -> Self {
        return Runtime {
            had_error: false,
            interpreter: Interpreter::default(),
        };
    }
}

impl Runtime {
    fn error(line: usize, message: String) {}

    fn report(&mut self, err: Box<dyn Error>) {
        println!("{}", err);
        self.had_error = true;
    }

    pub fn run_file(&mut self, file_name: String) {
        let all_file = fs::read_to_string(file_name).expect("read file error");
        self.run(all_file);
        if self.had_error {
            std::process::exit(65);
        }
    }

    fn run(&mut self, file: String) {
        let tokens = scanner::scan_tokens(file);
        let expression = parser::Parser::new(tokens.unwrap()).parse();
        match expression {
            Ok(exp) => {
                for ex in exp {
                    match self.interpreter.interpret_statement(&ex) {
                        Ok(result) => {}
                        Err(e) => {
                            self.report(Box::new(e))
                        }
                    }
                }
            }
            Err(e) => {
                self.report(Box::new(e))
            }
        }
    }

    pub fn run_prompt(&mut self) {
        let stdin = io::stdin();
        println!("input: ");
        for line in stdin.lock().lines() {
            let readed = line.unwrap();
            if readed.len() == 0 {
                break;
            }
            self.run(readed);
            self.had_error = false;
        }
    }
}