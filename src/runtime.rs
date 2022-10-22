use std::{fs, io};
use std::error::Error;
use std::io::BufRead;
use crate::process::scanner;

pub struct Runtime {
    had_error: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        return Runtime {
            had_error: false
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
        let vec = scanner::scan_tokens(file);
        for x in vec {
            println!("token: {:?}", x)
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