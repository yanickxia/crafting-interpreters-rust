use std::ops::Add;

use crate::types::expr;
use crate::types::expr::{Expression, Literal};

pub trait Accept {
    fn accept(&self, printer: &dyn Printer) -> String;
}


pub trait Printer {
    fn visit_expr(&self, group: &expr::Expression) -> String;
}

pub struct AstPrinter {}

impl AstPrinter {
    pub fn new() -> Self {
        return AstPrinter {};
    }

    fn parenthesize(&self, name: &str, expressions: Vec<&Expression>) -> String {
        let mut result = "".to_string();
        result = result.add("(");
        result = result.add(name);
        for expression in expressions {
            result = result.add(" ");
            result = result.add(expression.accept(self).as_str());
        }
        result = result.add(")");
        return result;
    }
}

impl Printer for AstPrinter {
    fn visit_expr(&self, group: &Expression) -> String {
        match group {
            Expression::Grouping(g) => {
                return self.parenthesize("group", vec![g.as_ref()]);
            }

            Expression::Binary(l, op, r) => {
                return self.parenthesize(op.token_type.to_string().as_str(), vec![l, r]);
            }
            Expression::Unary(op, exp) => {
                return self.parenthesize(op.token_type.to_string().as_str(), vec![exp]);
            }
            Expression::Literal(l) => {
                return match l {
                    Literal::String(s) => {
                        s.to_string()
                    }
                    Literal::Number(n) => {
                        n.to_string()
                    }
                    Literal::Nil => {
                        "nil".to_string()
                    }
                    Literal::True => {
                        "true".to_string()
                    }
                    Literal::False => {
                        "false".to_string()
                    }
                };
            }

            _ => {
                return "".to_string();
            }
        }
    }
}