use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::process::ast;
use crate::process::ast::Printer;
use crate::types::token;

#[derive(Debug)]
pub enum ExpError {
    Common(String),
    VariableRepeatDef(String),
    UnexpectedToken(token::Token),
    TokenMismatch {
        expected: token::TokenType,
        found: token::Token,
        err_string: Option<String>,
    },
    ConvertFailed {
        expected: Vec<token::TokenType>,
        found: token::Token,
    },
    ExpectedExpression {
        token_type: token::TokenType,
        line: usize,
    },
    AssignmentFailed {
        name: String
    },
    TooManyArgs,
}

impl Display for ExpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ExpError::TooManyArgs => write!(
                f,
                "too many args, can't more than 255"
            ),
            ExpError::UnexpectedToken(tok) => write!(
                f,
                "Unexpected token {:?} at line={}",
                tok.token_type, tok.line
            ),
            ExpError::TokenMismatch {
                expected,
                found,
                err_string,
            } => {
                write!(
                    f,
                    "Expected token {:?} but found {:?}",
                    expected, found.token_type
                )?;
                if let Some(on_err_string) = err_string {
                    write!(f, ": {}", on_err_string)?;
                }
                Ok(())
            }
            ExpError::ConvertFailed { expected, found } => write!(
                f,
                "Cannot ConvertFailed, expected {:?}, found {:?}", expected, found
            ),
            ExpError::ExpectedExpression { token_type, line } => write!(
                f,
                "ExpectedExpression line={},token_type={:?}",
                line, token_type
            ),
            ExpError::AssignmentFailed { name } => write!(f, "{}, Invalid assignment target.", name),

            ExpError::VariableRepeatDef(name) => write!(f, "{}, Variable repeat def.", name),
            ExpError::Common(str) => write!(f, "{}", str),
        }
    }
}

impl Error for ExpError {}


impl ast::Accept for Expression {
    fn accept(&self, printer: &dyn Printer) -> String {
        return printer.visit_expr(self);
    }
}


#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}


#[derive(Debug, Copy, Clone)]
pub struct UnaryOp {
    pub token_type: UnaryOperatorType,
    // pub line: usize,
    // pub col: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct BinaryOp {
    pub token_type: BinaryOperatorType,
    // pub line: usize,
    // pub col: i64,
}


#[derive(Debug, Copy, Clone)]
pub enum UnaryOperatorType {
    Minus,
    Bang,
}

impl Display for UnaryOperatorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LogicalOperatorType {
    And,
    Or,
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOperatorType {
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash,
}

impl Display for BinaryOperatorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(Literal),
    Unary(UnaryOp, Box<Expression>),
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    Call(Box<Expression>, String, Vec<Expression>),
    Get {
        object: Box<Expression>,
        variable: String,
    },
    Set {
        object: Box<Expression>,
        variable: String,
        value: Box<Expression>,
    },
    Super {
        keyword: String,
        method: String,
    },
    This(String),
    Grouping(Box<Expression>),
    Variable(String),
    Assign(String, Box<Expression>),
    Logical(Box<Expression>, LogicalOperatorType, Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum Statement {
    Expression(Expression),
    Function(String, Vec<String>, Box<Statement>),
    Print(Expression),
    Return(String, Option<Expression>),
    Var(String, Expression),
    Block(Vec<Statement>),
    Class {
        name: String,
        methods: Vec<Statement>,
        super_class: Option<String>,
    },
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
}
