use std::{fs, io};
use std::error::Error;
use std::io::BufRead;

use crate::process::{interpreter, parser, scanner};
use crate::process::ast::Printer;
use crate::process::interpreter::Interpreter;
use crate::types::val::{InterpreterError, Value};

pub struct Runtime {
    had_error: bool,
    interpreter: interpreter::AstInterpreter,
}

impl Default for Runtime {
    fn default() -> Self {
        return Runtime {
            had_error: false,
            interpreter: interpreter::AstInterpreter::default(),
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
                    match self.interpreter.visit_statement(&ex) {
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