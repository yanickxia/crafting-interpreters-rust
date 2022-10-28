use crate::types::token::Token;

pub trait Expression {}

pub trait Operator {}

pub struct Binary<'a> {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
    pub operator: &'a Token,
}

impl Expression for Binary<'_> {}

impl Binary<'_> {
    pub fn new(left: Box<dyn Expression>, operator: &Token, right: Box<dyn Expression>) -> Self {
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
