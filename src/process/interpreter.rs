use std::cmp::Ordering;

use crate::process::environment;
use crate::types::{expr, val};

pub trait Interpreter {
    fn visit_expression(&mut self, expr: &expr::Expression) -> Result<val::Value, val::InterpreterError>;
    fn visit_statement(&mut self, expr: &expr::Statement) -> Result<val::Value, val::InterpreterError>;
}

#[derive(Default)]
pub struct AstInterpreter {
    environment: environment::Environment,
}

impl Interpreter for AstInterpreter {
    fn visit_statement(&mut self, expr: &expr::Statement) -> Result<val::Value, val::InterpreterError> {
        return match expr {
            expr::Statement::Expression(exp) => {
                self.visit_expression(exp);
                Ok(val::Value::Nil)
            }
            expr::Statement::Print(exp) => {
                println!("{:?}", self.visit_expression(exp));
                Ok(val::Value::Nil)
            }
            expr::Statement::Var(name, var) => {
                let value = self.visit_expression(var)?;
                self.environment.define(name.to_string(), &value);
                Ok(val::Value::Nil)
            }
        };
    }

    fn visit_expression(&mut self, expr: &expr::Expression) -> Result<val::Value, val::InterpreterError> {
        match expr {
            expr::Expression::Literal(l) => {
                match l {
                    expr::Literal::String(s) => {
                        return Ok(val::Value::String(s.to_string()));
                    }
                    expr::Literal::Number(n) => {
                        return Ok(val::Value::Number(*n));
                    }
                    expr::Literal::Nil => {
                        return Ok(val::Value::Nil);
                    }
                    expr::Literal::True => {
                        return Ok(val::Value::Bool(true));
                    }
                    expr::Literal::False => {
                        return Ok(val::Value::Bool(false));
                    }
                }
            }
            expr::Expression::Grouping(expr) => {
                self.visit_expression(expr)
            }

            expr::Expression::Binary(left, op, right) => {
                let left = self.visit_expression(left)?;
                let right = self.visit_expression(right)?;
                return match op.token_type {
                    expr::BinaryOperatorType::EqualEqual => {
                        Ok(val::Value::Bool(left.eq(&right)))
                    }
                    expr::BinaryOperatorType::NotEqual => {
                        Ok(val::Value::Bool(!left.eq(&right)))
                    }
                    expr::BinaryOperatorType::Less => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::Less,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Less));
                            }
                        }
                    }
                    expr::BinaryOperatorType::LessEqual => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::LessEqual,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Less || left.eq(&right)));
                            }
                        }
                    }
                    expr::BinaryOperatorType::Greater => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::Greater,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Greater));
                            }
                        }
                    }
                    expr::BinaryOperatorType::GreaterEqual => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::GreaterEqual,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Greater || left.eq(&right)));
                            }
                        }
                    }
                    expr::BinaryOperatorType::Plus => {
                        match left {
                            val::Value::Number(x) => {
                                match right {
                                    val::Value::Number(y) => {
                                        Ok(val::Value::Number(x + y))
                                    }
                                    _ => {
                                        Err(val::InterpreterError::OperatorNotMatch {
                                            left,
                                            right,
                                            opt: expr::BinaryOperatorType::Plus,
                                        })
                                    }
                                }
                            }
                            val::Value::String(x) => {
                                match right {
                                    val::Value::String(y) => {
                                        Ok(val::Value::String((x.to_owned() + y.as_str()).to_string()))
                                    }
                                    _ => {
                                        Err(val::InterpreterError::OperatorNotMatch {
                                            left: val::Value::String(x),
                                            right,
                                            opt: expr::BinaryOperatorType::Plus,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::Plus,
                                })
                            }
                        }
                    }
                    expr::BinaryOperatorType::Minus => {
                        match left {
                            val::Value::Number(x) => {
                                match right {
                                    val::Value::Number(y) => {
                                        Ok(val::Value::Number(x - y))
                                    }
                                    _ => {
                                        Err(val::InterpreterError::OperatorNotMatch {
                                            left,
                                            right,
                                            opt: expr::BinaryOperatorType::Minus,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::Minus,
                                })
                            }
                        }
                    }
                    expr::BinaryOperatorType::Star => {
                        match left {
                            val::Value::Number(x) => {
                                match right {
                                    val::Value::Number(y) => {
                                        Ok(val::Value::Number(x * y))
                                    }
                                    _ => {
                                        Err(val::InterpreterError::OperatorNotMatch {
                                            left,
                                            right,
                                            opt: expr::BinaryOperatorType::Minus,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::Star,
                                })
                            }
                        }
                    }
                    expr::BinaryOperatorType::Slash => {
                        match left {
                            val::Value::Number(x) => {
                                match right {
                                    val::Value::Number(y) => {
                                        Ok(val::Value::Number(x / y))
                                    }
                                    _ => {
                                        Err(val::InterpreterError::OperatorNotMatch {
                                            left,
                                            right,
                                            opt: expr::BinaryOperatorType::Slash,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::Slash,
                                })
                            }
                        }
                    }
                };
            }

            expr::Expression::Unary(opt, expr) => {
                let value = self.visit_expression(expr)?;
                return match opt.token_type {
                    expr::UnaryOperatorType::Minus => {
                        match value {
                            val::Value::Number(n) => {
                                Ok(val::Value::Number(-n))
                            }
                            other => {
                                Err(val::InterpreterError::TypeNotMatch {
                                    expected: "want val::Value::Number".to_string(),
                                    found: other,
                                })
                            }
                        }
                    }
                    expr::UnaryOperatorType::Bang => {
                        match value {
                            val::Value::Bool(b) => {
                                Ok(val::Value::Bool(!b))
                            }
                            val::Value::Nil => {
                                Ok(val::Value::Bool(false))
                            }
                            other => {
                                Err(val::InterpreterError::TypeNotMatch {
                                    expected: "want val::Value::Bool".to_string(),
                                    found: other,
                                })
                            }
                        }
                    }
                };
            }

            expr::Expression::Variable(name) => {
                match self.environment.get(name) {
                    None => {
                        Err(val::InterpreterError::MissVariable {
                            name: name.to_string()
                        })
                    }
                    Some(val) => {
                        Ok(val.clone())
                    }
                }
            }

            expr::Expression::Assign(name, expr) => {
                let val = self.visit_expression(expr)?;
                return match self.environment.assign(name.to_string(), &val) {
                    Ok(_) => {
                        Ok(val)
                    }
                    Err(_) => {
                        Err(val::InterpreterError::MissVariable {
                            name: name.to_string()
                        })
                    }
                };
            }
        }
    }
}
