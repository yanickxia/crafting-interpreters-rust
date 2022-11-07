use std::cmp::Ordering;
use crate::types::{expr, val};
use crate::types::expr::BinaryOperatorType;


pub trait Interpreter {
    fn evaluate(&self, group: &expr::Expression) -> Result<val::Value, val::InterpreterError>;
}

#[derive(Default)]
pub struct AstInterpreter {}

impl Interpreter for AstInterpreter {
    fn evaluate(&self, group: &expr::Expression) -> Result<val::Value, val::InterpreterError> {
        match group {
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
                self.evaluate(expr)
            }

            expr::Expression::Binary(left, op, right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                return match op.token_type {
                    BinaryOperatorType::EqualEqual => {
                        Ok(val::Value::Bool(left.eq(&right)))
                    }
                    BinaryOperatorType::NotEqual => {
                        Ok(val::Value::Bool(!left.eq(&right)))
                    }
                    BinaryOperatorType::Less => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::Less,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Less));
                            }
                        }
                    }
                    BinaryOperatorType::LessEqual => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::LessEqual,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Less || left.eq(&right)));
                            }
                        }
                    }
                    BinaryOperatorType::Greater => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::Greater,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Greater));
                            }
                        }
                    }
                    BinaryOperatorType::GreaterEqual => {
                        match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::GreaterEqual,
                                })
                            }
                            Some(ord) => {
                                return Ok(val::Value::Bool(ord == Ordering::Greater || left.eq(&right)));
                            }
                        }
                    }
                    BinaryOperatorType::Plus => {
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
                                            opt: BinaryOperatorType::Plus,
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
                                            opt: BinaryOperatorType::Plus,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::Plus,
                                })
                            }
                        }
                    }
                    BinaryOperatorType::Minus => {
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
                                            opt: BinaryOperatorType::Minus,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::Minus,
                                })
                            }
                        }
                    }
                    BinaryOperatorType::Star => {
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
                                            opt: BinaryOperatorType::Minus,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::Star,
                                })
                            }
                        }
                    }
                    BinaryOperatorType::Slash => {
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
                                            opt: BinaryOperatorType::Slash,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: BinaryOperatorType::Slash,
                                })
                            }
                        }
                    }
                };
            }

            expr::Expression::Unary(opt, expr) => {
                let value = self.evaluate(expr)?;
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
            _ => {
                unimplemented!()
            }
        }
    }
}
