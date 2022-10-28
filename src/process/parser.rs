use crate::types::expr::{Binary, Expression};
use crate::types::token::TokenType::{BangEqual, Eof, EqualEqual};
use crate::types::token::{Token, TokenType};
use core::panicking::panic;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn expression(&mut self) -> Box<dyn Expression> {
        return self.equality();
    }

    fn equality(&mut self) -> Box<dyn Expression> {
        let expr = self.comparison();
        while self.match_token(vec![BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            return Box::new(Binary::new(expr, operator, right));
        }
        return expr;
    }

    fn comparison(&mut self) -> Box<dyn Expression> {
        panic!("xxx")
    }

    fn match_token(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.at_end() {
            return false;
        }
        return self.peek().token_type.eq(&token_type);
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1
        }
        return self.previous();
    }

    fn at_end(&mut self) -> bool {
        return self.peek().token_type == Eof;
    }

    fn peek(&mut self) -> &Token {
        return &self.tokens[self.current];
    }

    fn previous(&mut self) -> &Token {
        return &self.tokens[self.current - 1];
    }
}
