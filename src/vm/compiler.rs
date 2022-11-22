use std::any::Any;

use crate::process::parser::Parser;
use crate::types::{expr, token, val};
use crate::types::expr::{ExpError, Literal};
use crate::types::token::{Token, TokenType};
use crate::vm::chunk;
use crate::vm::chunk::{Chunk, OpCode};

#[derive(Debug, Copy, Clone)]
enum ParseFn {
    Grouping,
    Unary,
    Binary,
    Number,
    Literal,
    String,
    Variable,
    And,
    Or,
    Call,
    Dot,
    This,
    Super,
    List,
    Subscript,
}


#[derive(Eq, PartialEq, PartialOrd, Copy, Clone, Debug)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => panic!("primary has no next precedence!"),
        }
    }
}

struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

pub struct Compiler {
    tokens: Vec<token::Token>,
    current: usize,
    compiling: Chunk,
}

impl Compiler {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0, compiling: Chunk::default() }
    }

    pub fn compile(&mut self) -> Result<Chunk, ExpError> {
        while !self.at_end() {
            self.expression()?;
        }

        Ok(self.compiling.clone())
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), ExpError> {
        let token = self.advance();
        let rule = Self::get_rule(token.token_type);

        match rule.prefix {
            None => {
                return Err(ExpError::UnexpectedToken(token.clone()));
            }
            Some(parse_fn) => {
                self.apply_parse_fn(parse_fn)?;
            }
        }

        while precedence <= Compiler::get_rule(self.peek().token_type).precedence {
            self.advance();
            match Self::get_rule(self.previous().token_type).infix {
                Some(parse_fn) => self.apply_parse_fn(parse_fn)?,
                None => panic!("could not find infix rule to apply tok = {:?}", self.peek()),
            }
        }

        Ok(())
    }


    fn apply_parse_fn(&mut self, parse_fn: ParseFn) -> Result<(), ExpError> {
        match parse_fn {
            ParseFn::Grouping => self.grouping(),
            ParseFn::Unary => self.unary(),
            ParseFn::Binary => self.binary(),
            ParseFn::Number => self.number(),
            ParseFn::Literal => self.literal(),
            _ => panic!("not here"),
            // ParseFn::String => self.string(can_assign),
            // ParseFn::Variable => self.variable(can_assign),
            // ParseFn::And => self.and(can_assign),
            // ParseFn::Or => self.or(can_assign),
            // ParseFn::Call => self.call(can_assign),
            // ParseFn::Dot => self.dot(can_assign),
            // ParseFn::This => self.this(can_assign),
            // ParseFn::Super => self.super_(can_assign),
            // ParseFn::List => self.list(can_assign),
            // ParseFn::Subscript => self.subscr(can_assign),
        }
    }

    fn literal(&mut self) -> Result<(), ExpError> {
        match self.previous().token_type {
            TokenType::False => {
                self.emit_opt(OpCode::OpFalse)
            }
            TokenType::Nil => {
                self.emit_opt(OpCode::OpNil)
            }
            TokenType::True => {
                self.emit_opt(OpCode::OpTrue)
            }
            _ => {
                panic!("not literal")
            }
        }
        Ok(())
    }


    fn expression(&mut self) -> Result<(), ExpError> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn grouping(&mut self) -> Result<(), ExpError> {
        self.expression()?;
        to_empty_result(self.consume(TokenType::RightParen, "Expect ')' after expression."))
    }

    fn binary(&mut self) -> Result<(), ExpError> {
        let token_type = self.previous().token_type;
        let rule = Self::get_rule(token_type);
        self.parse_precedence(rule.precedence.next())?;

        match token_type {
            TokenType::Minus => {
                self.emit_opt(OpCode::OpSubtract)
            }
            TokenType::Plus => {
                self.emit_opt(OpCode::OpAdd)
            }
            TokenType::Slash => {
                self.emit_opt(OpCode::OpDivide)
            }
            TokenType::Star => {
                self.emit_opt(OpCode::OpMultiply)
            }
            _ => panic!("not binary opt")
        }
        Ok(())
    }

    fn unary(&mut self) -> Result<(), ExpError> {
        self.parse_precedence(Precedence::Unary)?;
        let token_type = self.previous().token_type;
        self.expression()?;
        match token_type {
            TokenType::Minus => {
                self.emit_opt(OpCode::OpNegate);
            }
            TokenType::Bang => {
                self.emit_opt(OpCode::OpNot);
            }
            TokenType::BangEqual => {
                self.emit_opt(OpCode::OpEqual);
                self.emit_opt(OpCode::OpNot);
            }
            TokenType::EqualEqual => {
                self.emit_opt(OpCode::OpEqual);
            }
            TokenType::Greater => {
                self.emit_opt(OpCode::OpGreater);
            }
            TokenType::GreaterEqual => {
                self.emit_opt(OpCode::OpLess);
                self.emit_opt(OpCode::OpNot);
            }
            TokenType::Less => {
                self.emit_opt(OpCode::OpLess);
            }
            TokenType::LessEqual => {
                self.emit_opt(OpCode::OpGreater);
                self.emit_opt(OpCode::OpNot);
            }
            _ => {
                Err(ExpError::TokenMismatch {
                    expected: token_type.clone(),
                    found: self.previous().clone(),
                    err_string: None,
                })?;
            }
        }
        Ok(())
    }


    fn number(&mut self) -> Result<(), ExpError> {
        match self.previous().literal {
            Some(token::Literal::Number(n)) => {
                self.emit_constant(chunk::Constant::Number(n))
            }
            _ => panic!("not number")
        }
        Ok(())
    }


    fn consume(&mut self, ty: TokenType, message: &str) -> Result<&Token, ExpError> {
        if self.check(ty) {
            return Ok(self.advance());
        }
        return Err(ExpError::TokenMismatch {
            expected: ty.clone(),
            found: self.previous().clone(),
            err_string: Some(message.to_string()),
        });
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.at_end() {
            return false;
        }
        return self.peek().token_type.eq(&token_type);
    }

    fn peek(&self) -> &token::Token {
        return &self.tokens[self.current];
    }


    fn emit_constant(&mut self, val: chunk::Constant) {
        let index = self.compiling.add_constant(val);
        self.compiling.code.push((OpCode::OpConstant(index), self.current))
    }


    fn emit_opt(&mut self, opt: OpCode) {
        self.compiling.code.push((opt, self.current))
    }

    fn end(&mut self) {
        self.compiling.code.push((OpCode::OpReturn, self.current))
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1
        }
        return self.previous();
    }

    fn previous(&mut self) -> &Token {
        return &self.tokens[self.current - 1];
    }

    fn at_end(&mut self) -> bool {
        return self.peek().token_type == TokenType::Eof;
    }

    fn get_rule(operator: TokenType) -> ParseRule {
        match operator {
            TokenType::LeftParen => ParseRule {
                prefix: Some(ParseFn::Grouping),
                infix: Some(ParseFn::Call),
                precedence: Precedence::Call,
            },
            TokenType::RightParen => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::LeftBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::RightBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            // TokenType::LeftParen => ParseRule {
            //     prefix: Some(ParseFn::List),
            //     infix: Some(ParseFn::Subscript),
            //     precedence: Precedence::Call,
            // },
            // TokenType::RightParen => ParseRule {
            //     prefix: None,
            //     infix: None,
            //     precedence: Precedence::None,
            // },
            TokenType::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Dot => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Dot),
                precedence: Precedence::Call,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(ParseFn::Unary),
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Term,
            },
            TokenType::Semicolon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(ParseFn::Unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Equality,
            },
            TokenType::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Comparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Comparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(ParseFn::Variable),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::String => ParseRule {
                prefix: Some(ParseFn::String),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Number => ParseRule {
                prefix: Some(ParseFn::Number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::And => ParseRule {
                prefix: None,
                infix: Some(ParseFn::And),
                precedence: Precedence::And,
            },
            TokenType::Class => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::False => ParseRule {
                prefix: Some(ParseFn::Literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::For => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Fun => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::If => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Nil => ParseRule {
                prefix: Some(ParseFn::Literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Or => ParseRule {
                prefix: None,
                infix: Some(ParseFn::Or),
                precedence: Precedence::Or,
            },
            TokenType::Print => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Return => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Super => ParseRule {
                prefix: Some(ParseFn::Super),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::This => ParseRule {
                prefix: Some(ParseFn::This),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::True => ParseRule {
                prefix: Some(ParseFn::Literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Var => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

fn to_empty_result(input: Result<&Token, ExpError>) -> Result<(), ExpError> {
    match input {
        Ok(_) => {
            Ok(())
        }
        Err(err) => {
            Err(err)
        }
    }
}