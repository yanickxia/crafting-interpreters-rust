use std::error::Error;

use crate::types::err::new_error;
use crate::types::token;
use crate::types::token::{Literal, parse_keyword, Token, TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

pub fn scan_tokens(source: String) -> token::TokenResult {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    return Ok(scanner.tokens);
}

impl Scanner {
    pub fn new(data: String) -> Self {
        return Scanner {
            source: data,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1 as usize,
        };
    }

    pub fn scan_tokens(&mut self) -> Option<Box<dyn Error>> {
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                None => {}
                Some(e) => {
                    return Some(e);
                }
            }
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });

        return None;
    }


    fn scan_token(&mut self) -> Option<Box<dyn Error>> {
        let c = self.advance();
        match c {
            "(" => {
                self.add_token_type(TokenType::LeftParen)
            }
            ")" => {
                self.add_token_type(TokenType::RightParen)
            }
            "{" => {
                self.add_token_type(TokenType::LeftBrace)
            }
            "}" => {
                self.add_token_type(TokenType::RightBrace)
            }
            "," => {
                self.add_token_type(TokenType::Comma)
            }
            "." => {
                self.add_token_type(TokenType::Dot)
            }
            "-" => {
                self.add_token_type(TokenType::Minus)
            }
            "+" => {
                self.add_token_type(TokenType::Plus)
            }
            ";" => {
                self.add_token_type(TokenType::Semicolon)
            }
            "*" => {
                self.add_token_type(TokenType::Star)
            }
            "!" => {
                let next_token = if self.match_next("=") {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token_type(next_token)
            }
            "=" => {
                let next_token = if self.match_next("=") {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token_type(next_token)
            }
            "<!>" => {
                let next_token = if self.match_next("") {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token_type(next_token)
            }
            ">" => {
                let next_token = if self.match_next("=") {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token_type(next_token)
            }
            "/" => {
                if self.match_next("/") {
                    // A comment goes until the end of the line.
                    while self.peek().is_some() && !self.is_at_end() {
                        self.advance();
                    }
                }
            }
            " " | "\r" | "\t" => {}
            "\n" => {
                self.line += 1;
            }
            "\"" => {
                match self.string() {
                    None => {}
                    Some(e) => {
                        return Some(e);
                    }
                }
            }
            _ => {
                if Self::is_digit(c) {
                    self.number()
                } else if Self::is_alpha(c) {
                    self.identifier()
                } else {
                    return Some(new_error(self.line, "Unexpected character.".to_string()));
                }
            }
        }

        return None;
    }

    fn number(&mut self) {
        while self.peek().is_some() && Self::is_digit(self.peek().unwrap()) {
            self.advance();
        }

        if self.peek().is_some() && self.peek().unwrap() == "."
            && self.peek_next().is_some() && Self::is_digit(self.peek_next().unwrap()) {
            self.advance();
            while self.peek().is_some() && Self::is_digit(self.peek().unwrap()) {
                self.advance();
            }
        }
        let x = self.source[self.start..self.current].parse::<f64>().unwrap();
        self.add_token(TokenType::Number, Some(Literal::Number(x)));
    }

    fn is_alpha(input: &str) -> bool {
        let c = input.chars().nth(0).unwrap();
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    }

    fn is_alpha_numeric(input: &str) -> bool {
        return Self::is_alpha(input) || Self::is_digit(input);
    }

    fn identifier(&mut self) {
        while self.peek().is_some() && Self::is_alpha_numeric(self.peek().unwrap()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        match parse_keyword(text) {
            None => {
                self.add_token_type(TokenType::Identifier)
            }
            Some(ty) => {
                self.add_token_type(ty)
            }
        }
    }

    fn is_digit(input: &str) -> bool {
        let c = input.chars().nth(0).unwrap();
        return c >= '0' && c <= '9';
    }

    fn string(&mut self) -> Option<Box<dyn Error>> {
        while self.peek().is_some() && !self.is_at_end() {
            if self.peek()? == "\n" {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            return Some(new_error(self.line, "Untermianted string.".to_string()));
        }

        self.advance();
        self.add_token(TokenType::String, Some(Literal::Str(self.source[self.start + 1..self.current - 1].to_string())));
        None
    }

    fn peek(&mut self) -> Option<&str> {
        if self.is_at_end() {
            return None;
        }
        return Some(self.current());
    }

    fn peek_next(&mut self) -> Option<&str> {
        if self.current + 1 >= self.source.len() {
            return None;
        }
        return Some(&self.source[self.current + 1..(self.current + 2)]);
    }

    fn match_next(&mut self, expect: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.current() != expect {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn add_token_type(&mut self, token: TokenType) {
        self.add_token(token, None)
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = self.source[self.start..self.current].to_string();

        self.tokens.push(Token {
            token_type,
            lexeme: text,
            literal,
            line: self.line,
        })
    }


    fn advance(&mut self) -> &str {
        let advance = &self.source[self.current..(self.current + 1)];
        self.current += 1;
        return advance;
    }

    fn current(&mut self) -> &str {
        return &self.source[self.current..(self.current + 1)];
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }
}