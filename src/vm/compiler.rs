use std::any::Any;

use crate::cast;
use crate::process::parser::Parser;
use crate::types::{expr, token, val};
use crate::types::expr::{ExpError, Literal};
use crate::types::token::{Token, TokenType};
use crate::types::val::Value;
use crate::vm::chunk;
use crate::vm::chunk::{Chunk, Class, Constant, Function, OpCode};
use crate::vm::chunk::OpCode::OpPop;
use crate::vm::vm::FunctionType;

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

pub struct Compiler {
    tokens: Vec<Token>,
    current: usize,
    scope_depth: usize,
    locals: Vec<Local>,
    function: Function,
    function_type: FunctionType,
}

impl Compiler {
    pub fn new(tokens: Vec<Token>, function_type: FunctionType) -> Self {
        let mut compiler = Self {
            tokens,
            current: 0,
            scope_depth: 0,
            locals: vec![],
            function: Default::default(),
            function_type,
        };
        return compiler;
    }

    pub fn current_chunk(&mut self) -> &mut Chunk {
        return &mut self.function.chunk;
    }

    pub fn current_line(&self) -> usize {
        return self.current;
    }

    pub fn current_function_mut(&mut self) -> &mut Function {
        return &mut self.function;
    }

    pub fn compile(&mut self) -> Result<Function, ExpError> {
        while !self.at_end() {
            self.declaration()?;
        }
        self.end();
        Ok(self.function.clone())
    }

    fn declaration(&mut self) -> Result<(), ExpError> {
        if self._match(TokenType::Class) {
            self.class_declaration()?;
        } else if self._match(TokenType::Fun) {
            self.fun_declaration()?;
        } else if self._match(TokenType::Var) {
            self.var_declaration()?;
        } else {
            self.statement()?;
        }

        Ok(())
    }

    fn class_declaration(&mut self) -> Result<(), ExpError> {
        self.consume(TokenType::Identifier, "Expect class name.")?;
        let class_name = self.previous().lexeme.clone();

        let constant_index = self.identifier_constant(class_name.clone());
        self.declare_variable()?;

        self.emit_opt(OpCode::OpClass(Class {
            name: class_name.clone(),
        }));
        self.define_variable(constant_index)?;

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;
        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;
        Ok(())
    }

    fn identifier_constant(&mut self, name: String) -> usize {
        self.current_chunk().add_constant(Constant::String(name))
    }

    fn fun_declaration(&mut self) -> Result<(), ExpError> {
        let function_name = self.parse_variable("expect function name")?;
        self.mark_initialized()?;
        self.function(FunctionType::Function)?;
        self.define_variable(function_name)
    }

    fn function(&mut self, fun_type: FunctionType) -> Result<(), ExpError> {
        let mut compiler = Self {
            tokens: self.tokens.clone(),
            current: self.current,
            scope_depth: 0,
            locals: vec![],
            function: Default::default(),
            function_type: fun_type,
        };
        compiler.function.name = self.previous().lexeme.clone();
        compiler.begin_scope()?;

        compiler.consume(TokenType::LeftParen, "Expect '(' after function name.")?;

        if !compiler.check(TokenType::RightParen) {
            loop {
                let func = compiler.current_function_mut();
                func.arity += 1;
                let parameter_name = compiler.parse_variable("Expected parameter name")?;
                compiler.define_variable(parameter_name)?;
                if !compiler._match(TokenType::Comma) {
                    break;
                }
            }
        }

        compiler.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        compiler.consume(TokenType::LeftBrace, "Expect '{' before function body.")?;
        compiler.block()?;

        compiler.emit_return();
        let func = compiler.function;
        self.emit_constant(Constant::Function(func));
        self.current = compiler.current;

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
        if self.scope_depth == 0 {
            return Ok(());
        }

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

    fn call(&mut self, _: bool) -> Result<(), ExpError> {
        let args = self.argument_list()?;
        self.emit_opt(OpCode::Call(args));
        Ok(())
    }

    fn argument_list(&mut self) -> Result<usize, ExpError> {
        let mut count = 0 as usize;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression()?;
                count += 1;
                if !self._match(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        return Ok(count);
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
        let i = self.current_chunk().add_constant(Constant::String(previous.lexeme));
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
        } else if self._match(TokenType::Return) {
            self.return_statement()?;
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

    fn return_statement(&mut self) -> Result<(), ExpError> {
        if self._match(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
            self.emit_opt(OpCode::OpReturn)
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

        let mut loop_start = self.current_chunk().code.len();
        let mut exit_jump = None;
        if !self._match(TokenType::Semicolon) {
            self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;
            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse(0)));
            self.emit_opt(OpCode::OpPop);
        }

        if !self._match(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump(0));
            let increment_start = self.current_chunk().code.len();
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
        let loop_start = self.current_chunk().code.len();
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
        let i = self.current_chunk().code.len() - loop_start + 1;
        self.emit_opt(OpCode::Loop(i))
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
        self.current_chunk().code.len() - 1
    }

    fn patch_jump(&mut self, jump_location: usize) {
        let true_jump = self.current_chunk().code.len() - jump_location - 1;
        let (jump, line) = &self.current_chunk().code[jump_location];
        match jump {
            OpCode::JumpIfFalse(_) => {
                self.current_chunk().code[jump_location] = (OpCode::JumpIfFalse(true_jump), *line)
            }
            OpCode::Jump(_) => {
                self.current_chunk().code[jump_location] = (OpCode::Jump(true_jump), *line)
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

    fn dot(&mut self, can_assign: bool) -> Result<(), ExpError> {
        self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
        let property_name = self.previous().lexeme.clone();
        if can_assign && self._match(TokenType::Equal) {
            self.expression()?;
            self.emit_opt(OpCode::OpSetProperty(property_name))
        } else {
            self.emit_opt(OpCode::OpGetProperty(property_name))
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
            ParseFn::Call => self.call(can_assign),
            ParseFn::Dot => self.dot(can_assign),
            _ => panic!("not here"),
            // ParseFn::Dot => self.dot(can_assign),
            // ParseFn::This => self.this(can_assign),
            // ParseFn::Super => self.super_(can_assign),
            // ParseFn::List => self.list(can_assign),
            // ParseFn::Subscript => self.subscr(can_assign),
        }
    }

    fn string(&mut self) -> Result<(), ExpError> {
        let string = self.prev_string()?;
        let index = self.identifier_constant(string);
        self.emit_opt(OpCode::OpConstant(index));
        Ok(())
    }

    fn prev_string(&mut self) -> Result<String, ExpError> {
        let prev = self.previous().clone();
        match prev.token_type {
            TokenType::String => {
                match prev.literal {
                    None => panic!("not here"),
                    Some(s) => {
                        match s {
                            token::Literal::Str(s) => {
                                return Ok(s);
                            }
                            _ => panic!("not here")
                        }
                    }
                }
            }
            _ => {}
        }
        Err(ExpError::Common("not string".to_string()))
    }

    fn variable(&mut self, can_assign: bool) -> Result<(), ExpError> {
        let name = self.previous().lexeme.clone();
        self.named_variable(name, can_assign)
    }

    fn named_variable(&mut self, name: String, can_assign: bool) -> Result<(), ExpError> {
        match self.resolve_local(name.clone())? {
            None => {
                let index = self.current_chunk().add_constant(Constant::String(name.clone()));
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
        let line = self.current;
        let compiling = self.current_chunk();
        let index = compiling.add_constant(val);
        compiling.code.push((OpCode::OpConstant(index), line))
    }


    fn emit_opt(&mut self, opt: OpCode) {
        let line = self.current;
        self.current_chunk().code.push((opt, line))
    }

    fn end(&mut self) {
        // self.emit_opt(OpCode::OpReturn);
    }

    fn emit_return(&mut self) {
        self.emit_opt(OpCode::OpNil);
        self.emit_opt(OpCode::OpReturn);
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