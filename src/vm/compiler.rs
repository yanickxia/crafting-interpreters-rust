use std::any::Any;

use crate::cast;
use crate::process::parser::Parser;
use crate::types::{expr, token, val};
use crate::types::expr::{ExpError, Literal};
use crate::types::token::{Token, TokenType};
use crate::types::val::Value;
use crate::vm::chunk;
use crate::vm::chunk::{Chunk, Constant, OpCode};

type ConstantIndex = usize;
type LocalIndex = usize;

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

#[derive(Clone)]
pub struct Local {
    name: String,
    depth: i32,
}

#[derive(Default)]
pub struct Compiler {
    tokens: Vec<Token>,
    current: usize,
    compiling: Chunk,
    scope_depth: usize,
    locals: Vec<Local>,
}

impl Compiler {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut compiler = Self::default();
        compiler.tokens = tokens;
        return compiler;
    }

    pub fn compile(&mut self) -> Result<Chunk, ExpError> {
        while !self.at_end() {
            self.declaration()?;
        }

        Ok(self.compiling.clone())
    }

    fn declaration(&mut self) -> Result<(), ExpError> {
        if self._match(TokenType::Var) {
            self.var_declaration()?;
        } else {
            self.statement()?;
        }

        Ok(())
    }

    fn var_declaration(&mut self) -> Result<(), ExpError> {
        let global = self.parse_variable("Expect variable name.")?;
        if self._match(TokenType::Equal) {
            self.expression()?;
        } else {
            self.emit_opt(OpCode::OpNil)
        }

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;
        self.define_variable(global)?;

        Ok(())
    }

    fn mark_initialized(&mut self) -> Result<(), ExpError> {
        let mut last = (*self.locals.last().expect("should exist")).clone();
        last.depth = self.scope_depth as i32;
        let last_index = self.locals.len() - 1;
        self.locals[last_index] = last;
        Ok(())
    }

    fn define_variable(&mut self, val: ConstantIndex) -> Result<(), ExpError> {
        if self.scope_depth > 0 {
            self.mark_initialized()?;
            return Ok(());
        }

        self.emit_opt(OpCode::OpDefineGlobal(val));
        Ok(())
    }

    fn and(&mut self, _: bool) -> Result<(), ExpError> {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        self.emit_opt(OpCode::OpPop);
        self.parse_precedence(Precedence::And)?;
        self.patch_jump(end_jump);
        Ok(())
    }

    fn or(&mut self, _: bool) -> Result<(), ExpError> {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        let end_jump = self.emit_jump(OpCode::Jump(0));

        self.patch_jump(else_jump);
        self.emit_opt(OpCode::OpPop);
        self.parse_precedence(Precedence::Or)?;
        self.patch_jump(end_jump);
        Ok(())
    }

    fn parse_variable(&mut self, err_msg: &str) -> Result<ConstantIndex, ExpError> {
        self.consume(TokenType::Identifier, err_msg)?;

        self.declare_variable()?;
        if self.scope_depth > 0 {
            return Ok(0);
        }


        let previous = self.previous().clone();
        let i = self.compiling.add_constant(Constant::String(previous.lexeme));
        return Ok(i);
    }

    fn declare_variable(&mut self) -> Result<(), ExpError> {
        if self.scope_depth == 0 {
            return Ok(());
        }

        let name = self.previous().lexeme.clone();
        for l in &self.locals {
            if l.depth != -1 && l.depth < self.scope_depth as i32 {
                break;
            }
            if l.name.eq(name.as_str()) {
                return Err(ExpError::VariableRepeatDef(name.clone()));
            }
        }

        self.add_local(name)?;

        Ok(())
    }

    fn add_local(&mut self, name: String) -> Result<(), ExpError> {
        self.locals.push(Local {
            name,
            depth: -1,
        });
        Ok(())
    }

    fn statement(&mut self) -> Result<(), ExpError> {
        if self._match(TokenType::Print) {
            self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
            self.emit_opt(OpCode::OpPrint)
        } else if self._match(TokenType::For) {
            self.for_statement()?;
        } else if self._match(TokenType::If) {
            self.if_statement()?;
        } else if self._match(TokenType::While) {
            self.while_statement()?;
        } else if self._match(TokenType::LeftBrace) {
            self.begin_scope()?;
            self.block()?;
            self.end_scope()?;
        } else {
            self.expression_statement()?;
        }
        Ok(())
    }

    fn for_statement(&mut self) -> Result<(), ExpError> {
        self.begin_scope()?;
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        if self._match(TokenType::Semicolon) {} else if self._match(TokenType::Var) {
            self.var_declaration()?;
        } else {
            self.expression_statement()?;
        }

        let mut loop_start = self.compiling.code.len();
        let mut exit_jump = None;
        if !self._match(TokenType::Semicolon) {
            self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;
            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse(0)));
            self.emit_opt(OpCode::OpPop);
        }
        
        if !self._match(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump(0));
            let increment_start = self.compiling.code.len();
            self.expression()?;
            self.emit_opt(OpCode::OpPop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }


        self.statement()?;
        self.emit_loop(loop_start);

        match exit_jump {
            None => {}
            Some(index) => {
                self.patch_jump(index);
                self.emit_opt(OpCode::OpPop);
            }
        }
        self.end_scope()?;
        Ok(())
    }

    fn while_statement(&mut self) -> Result<(), ExpError> {
        let loop_start = self.compiling.code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        self.emit_opt(OpCode::OpPop);
        self.statement()?;

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_opt(OpCode::OpPop);
        Ok(())
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_opt(OpCode::Loop(self.compiling.code.len() - loop_start + 1))
    }

    fn if_statement(&mut self) -> Result<(), ExpError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let then_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        self.emit_opt(OpCode::OpPop);
        self.statement()?;
        let else_jump = self.emit_jump(OpCode::Jump(0));
        self.patch_jump(then_jump);
        self.emit_opt(OpCode::OpPop);
        if self._match(TokenType::Else) {
            self.statement()?;
        }
        self.patch_jump(else_jump);

        Ok(())
    }

    fn emit_jump(&mut self, opt: OpCode) -> usize {
        self.emit_opt(opt);
        self.compiling.code.len() - 1
    }

    fn patch_jump(&mut self, jump_location: usize) {
        let true_jump = self.compiling.code.len() - jump_location - 1;
        let (jump, line) = &self.compiling.code[jump_location];
        match jump {
            OpCode::JumpIfFalse(_) => {
                self.compiling.code[jump_location] = (OpCode::JumpIfFalse(true_jump), *line)
            }
            OpCode::Jump(_) => {
                self.compiling.code[jump_location] = (OpCode::Jump(true_jump), *line)
            }
            _ => panic!("not here")
        }
    }

    fn block(&mut self) -> Result<(), ExpError> {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration()?;
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(())
    }

    fn expression_statement(&mut self) -> Result<(), ExpError> {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        self.emit_opt(OpCode::OpPop);

        Ok(())
    }

    fn begin_scope(&mut self) -> Result<(), ExpError> {
        self.scope_depth += 1;
        Ok(())
    }

    fn end_scope(&mut self) -> Result<(), ExpError> {
        self.scope_depth -= 1;
        while self.locals.len() > 0 && self.locals.last().expect("exist").depth > self.scope_depth as i32 {
            self.emit_opt(OpCode::OpPop);
            self.locals.pop();
        }
        Ok(())
    }

    fn apply_parse_fn(&mut self, parse_fn: ParseFn, can_assign: bool) -> Result<(), ExpError> {
        match parse_fn {
            ParseFn::Grouping => self.grouping(),
            ParseFn::Unary => self.unary(),
            ParseFn::Binary => self.binary(),
            ParseFn::Number => self.number(),
            ParseFn::Literal => self.literal(),
            ParseFn::String => self.string(),
            ParseFn::Variable => self.variable(can_assign),
            ParseFn::And => self.and(can_assign),
            ParseFn::Or => self.or(can_assign),
            _ => panic!("not here"),
            // ParseFn::Call => self.call(can_assign),
            // ParseFn::Dot => self.dot(can_assign),
            // ParseFn::This => self.this(can_assign),
            // ParseFn::Super => self.super_(can_assign),
            // ParseFn::List => self.list(can_assign),
            // ParseFn::Subscript => self.subscr(can_assign),
        }
    }

    fn string(&mut self) -> Result<(), ExpError> {
        let prev = self.previous().clone();
        match prev.token_type {
            TokenType::String => {
                match prev.literal {
                    None => panic!("not here"),
                    Some(s) => {
                        match s {
                            token::Literal::Str(s) => {
                                let index = self.compiling.add_constant(Constant::String(s));
                                self.emit_opt(OpCode::OpConstant(index))
                            }
                            _ => panic!("not here")
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> Result<(), ExpError> {
        let name = self.previous().lexeme.clone();
        self.named_variable(name, can_assign)
    }

    fn named_variable(&mut self, name: String, can_assign: bool) -> Result<(), ExpError> {
        match self.resolve_local(name.clone())? {
            None => {
                let index = self.compiling.add_constant(Constant::String(name.clone()));
                if can_assign && self._match(TokenType::Equal) {
                    self.expression()?;
                    self.emit_opt(OpCode::OpSetGlobal(index));
                } else {
                    self.emit_opt(OpCode::OpGetGlobal(index));
                }
            }
            Some(index) => {
                if can_assign && self._match(TokenType::Equal) {
                    self.expression()?;
                    self.emit_opt(OpCode::OpSetLocal(index));
                } else {
                    self.emit_opt(OpCode::OpGetLocal(index));
                }
            }
        }

        Ok(())
    }

    fn resolve_local(&mut self, name: String) -> Result<Option<LocalIndex>, ExpError> {
        for i in (0..self.locals.len()).rev() {
            let local = &self.locals[i];
            if local.name.eq(name.as_str()) {
                if local.depth == -1 {
                    return Err(ExpError::Common("Can't read local variable in its own initializer.".to_string()));
                }
                return Ok(Some(i));
            }
        }
        return Ok(None);
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), ExpError> {
        let token = self.advance();
        let can_assign = precedence <= Precedence::Assignment;
        let rule = Self::get_rule(token.token_type);

        match rule.prefix {
            None => {
                return Err(ExpError::UnexpectedToken(token.clone()));
            }
            Some(parse_fn) => {
                self.apply_parse_fn(parse_fn, can_assign)?;
            }
        }

        while precedence <= Compiler::get_rule(self.peek().token_type).precedence {
            self.advance();
            match Self::get_rule(self.previous().token_type).infix {
                Some(parse_fn) => self.apply_parse_fn(parse_fn, can_assign)?,
                None => panic!("could not find infix rule to apply tok = {:?}", self.peek()),
            }
        }

        if can_assign && self._match(TokenType::Equal) {
            panic!("Invalid assignment target")
        }

        Ok(())
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
            TokenType::Slash => {
                self.emit_opt(OpCode::OpDivide)
            }
            TokenType::Star => {
                self.emit_opt(OpCode::OpMultiply)
            }
            TokenType::Minus => {
                self.emit_opt(OpCode::OpSubtract)
            }
            TokenType::Plus => {
                self.emit_opt(OpCode::OpAdd)
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
                panic!("not binary opt")
            }
        }
        Ok(())
    }

    fn unary(&mut self) -> Result<(), ExpError> {
        // self.parse_precedence(Precedence::Unary)?;
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
                // Err(ExpError::TokenMismatch {
                //     expected: token_type.clone(),
                //     found: self.previous().clone(),
                //     err_string: None,
                // })?;
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

    fn _match(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        true
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


    fn emit_constant(&mut self, val: Constant) {
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