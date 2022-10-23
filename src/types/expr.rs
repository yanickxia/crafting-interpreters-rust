use crate::types::token::TokenType;

pub trait Expr {}

pub struct Binary {
    pub left: Box<dyn Expr>,
    pub right: Box<dyn Expr>,
    pub operator: TokenType,
}

impl Expr for Binary {}

impl Binary {
    pub fn new(left: Box<dyn Expr>, operator: TokenType, right: Box<dyn Expr>) -> Self {
        return Self {
            left,
            right,
            operator,
        };
    }
}

pub struct Unary {}

impl Expr for Unary {

}
