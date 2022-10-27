use crate::types::token::{Token, TokenType};

pub trait Expression {}

pub trait Operator {}

pub struct Binary {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
    pub operator: Token,
}

impl Expression for Binary {}

impl Binary {
    pub fn new(left: Box<dyn Expression>, operator: Token, right: Box<dyn Expression>) -> Self {
        return Self {
            left,
            right,
            operator,
        };
    }
}

pub struct Unary {}

impl Expression for Unary {}

pub struct Literal {}

impl Expression for Literal {}

pub struct Grouping {}

impl Expression for Grouping {}

