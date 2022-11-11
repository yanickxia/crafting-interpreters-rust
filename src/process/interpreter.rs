use std::cmp::Ordering;
use std::collections::HashMap;

use crate::process::environment;
use crate::types::{class, expr, func, val};

#[derive(Default)]
pub struct Interpreter {
    pub environment: environment::Environment,
    pub global: environment::Environment,
    pub lox_functions: HashMap<usize, func::LoxFunction>,
    pub lox_instances: HashMap<usize, class::LoxInstance>,
    counter: usize,
    pub ret: Option<val::Value>,
}

impl Interpreter {
    pub fn execute(&mut self, expr: &expr::Statement) -> Result<(), val::InterpreterError> {
        self.interpret_statement(expr)?;
        Ok(())
    }

    pub fn execute_block(&mut self, sts: &Vec<expr::Statement>) -> Result<(), val::InterpreterError> {
        // everytime execute, should be new env for block
        self.environment = environment::Environment::with_enclosing(self.environment.clone());
        for st in sts {
            match self.execute(st) {
                Ok(_) => {
                    match self.ret {
                        None => {}
                        Some(_) => {
                            // fast return
                            break;
                        }
                    }
                }
                Err(e) => {
                    return Err(val::InterpreterError::ExecuteError(Box::new(e)));
                }
            }
        }

        match &self.environment.enclosing {
            None => {
                panic!("impossible, always has previous");
            }
            Some(previous) => {
                self.environment = *previous.clone()
            }
        }
        Ok(())
    }

    fn cast_callable(interpreter: &mut Self, value: &val::Value) -> Option<Box<dyn func::Callable>> {
        match value {
            val::Value::LoxFunc(_, id) => {
                let f = interpreter.get_lox_function(*id);
                Some(Box::new(f.clone()))
            }
            val::Value::LoxClass(class) => {
                Some(Box::new(class.clone()))
            }
            _ => None,
        }
    }

    pub fn next_id(&mut self) -> usize {
        let res = self.counter;
        self.counter += 1;
        res
    }


    pub fn get_lox_function(&self, id: usize) -> &func::LoxFunction {
        match self.lox_functions.get(&id) {
            Some(func) => func,
            None => panic!(
                "Internal interpreter error! couldn't find an function with id {}.",
                id
            ),
        }
    }

    pub fn interpret_statement(&mut self, expr: &expr::Statement) -> Result<(), val::InterpreterError> {
        return match expr {
            expr::Statement::Class {
                name, methods, super_class
            } => {
                let mut super_lox_class = None;

                match super_class {
                    None => {}
                    Some(super_class) => {
                        match self.environment.get(super_class).unwrap() {
                            val::Value::LoxClass(clazz) => {
                                super_lox_class = Some(Box::new(clazz.clone()))
                            }
                            _ => {}
                        }
                    }
                }


                self.environment.define(name.to_string(), &val::Value::Nil);
                let mut lox_class = class::LoxClass::default();
                lox_class.name = name.to_string();
                lox_class.super_class = super_lox_class;
                let mut lox_class_methods = vec![];
                // init methods
                for method in methods {
                    match method {
                        expr::Statement::Function(name, params, body) => {
                            let func_id = self.next_id();
                            let lox_function = func::LoxFunction {
                                id: func_id,
                                name: name.to_string(),
                                parameters: params.clone(),
                                body: *body.clone(),
                                closure: self.environment.clone(),
                                bind: None,
                                is_initializer: name.as_str().eq("init"),
                            };
                            self.lox_functions.insert(func_id, lox_function);
                            lox_class_methods.push(val::Value::LoxFunc(name.to_string(), func_id))
                        }
                        _ => panic!("not method")
                    }
                }
                lox_class.methods = lox_class_methods;
                self.environment.assign(name.to_string(), &val::Value::LoxClass(lox_class)).expect("failed");
                Ok(())
            }
            expr::Statement::Return(_, expr) => {
                match expr {
                    Some(expr) => {
                        self.ret = Some(self.interpret_expression(expr)?);
                    }
                    _ => {}
                }
                Ok(())
            }
            expr::Statement::Function(name, params, body) => {
                let func_id = self.next_id();

                // env 里面要放入这个函数，不然后面找不到
                self.environment.define(name.to_string(), &val::Value::LoxFunc(name.to_string(), func_id));

                let lox_function = func::LoxFunction {
                    id: func_id,
                    name: name.to_string(),
                    parameters: params.clone(),
                    body: *body.clone(),
                    closure: self.environment.clone(),
                    bind: None,
                    is_initializer: false,
                };

                self.lox_functions.insert(func_id, lox_function);

                Ok(())
            }
            expr::Statement::Expression(exp) => {
                self.interpret_expression(exp)?;
                Ok(())
            }
            expr::Statement::Print(exp) => {
                println!("{:?}", self.interpret_expression(exp));
                Ok(())
            }
            expr::Statement::Var(name, var) => {
                let value = self.interpret_expression(var)?;
                self.environment.define(name.to_string(), &value);
                Ok(())
            }
            expr::Statement::Block(sts) => {
                self.execute_block(sts)?;
                Ok(())
            }
            expr::Statement::If(condition, then, els) => {
                let condition = self.interpret_expression(condition)?;
                return match condition {
                    val::Value::Bool(b) => {
                        if b {
                            return self.interpret_statement(then);
                        }
                        match els {
                            None => {
                                Ok(())
                            }
                            Some(sts) => {
                                self.interpret_statement(sts)
                            }
                        }
                    }
                    _ => {
                        panic!("should be bool")
                    }
                };
            }
            expr::Statement::While(condition, sts) => {
                loop {
                    let condition = self.interpret_expression(condition)?;
                    match condition {
                        val::Value::Bool(b) => {
                            if b {
                                self.interpret_statement(sts)?;
                            } else {
                                return Ok(());
                            }
                        }
                        _ => {
                            panic!("should be bool")
                        }
                    }
                }
            }
        };
    }

    fn lookup(&self, name: String) -> Result<val::Value, val::InterpreterError> {
        return match self.environment.get(name.as_str()) {
            None => {
                match self.global.get(name.as_str()) {
                    None => {
                        Err(val::InterpreterError::MissVariable {
                            name
                        })
                    }
                    Some(val) => {
                        Ok(val.clone())
                    }
                }
            }
            Some(val) => {
                Ok(val.clone())
            }
        };
    }

    fn interpret_expression(&mut self, expr: &expr::Expression) -> Result<val::Value, val::InterpreterError> {
        match expr {
            expr::Expression::This(this) => {
                self.lookup(this.to_string())
            }
            expr::Expression::Set { object, variable, value } => {
                let obj = self.interpret_expression(object)?;
                let val = self.interpret_expression(value)?;
                return match obj {
                    val::Value::LoxInstance(ref id) => {
                        return match self.lox_instances.get_mut(&id) {
                            None => {
                                Err(val::InterpreterError::SimpleError(format!("miss instance: {:?}", id)))
                            }
                            Some(mut instance) => {
                                instance.set(variable, val);
                                Ok(val::Value::Nil)
                            }
                        };
                    }
                    _ => {
                        Err(val::InterpreterError::SimpleError("should be call in instance".to_string()))
                    }
                };
            }
            expr::Expression::Get { object, variable } => {
                let obj = self.interpret_expression(object)?;
                return match obj {
                    val::Value::LoxInstance(instance_id) => {
                        return match self.lox_instances.get(&instance_id) {
                            None => {
                                Err(val::InterpreterError::SimpleError(format!("miss instance: {:?}", instance_id)))
                            }
                            Some(instance) => {
                                match instance.get(variable.as_str()) {
                                    None => {
                                        Err(val::InterpreterError::SimpleError(format!("miss variable: {} in {:?}", variable.as_str(), instance)))
                                    }
                                    Some(val) => {
                                        return match val {
                                            // bind instance
                                            val::Value::LoxFunc(_, ref func_id) => {
                                                let lox_func = self.lox_functions.get_mut(func_id).unwrap();
                                                lox_func.bind = Some(instance_id);
                                                Ok(val.clone())
                                            }
                                            _ => {
                                                Ok(val.clone())
                                            }
                                        };
                                    }
                                }
                            }
                        };
                    }
                    _ => {
                        Err(val::InterpreterError::SimpleError("should be call in instance".to_string()))
                    }
                };
            }
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
                self.interpret_expression(expr)
            }

            expr::Expression::Binary(left, op, right) => {
                let left = self.interpret_expression(left)?;
                let right = self.interpret_expression(right)?;
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
                        return match left.partial_cmp(&right) {
                            None => {
                                Err(val::InterpreterError::OperatorNotMatch {
                                    left,
                                    right,
                                    opt: expr::BinaryOperatorType::GreaterEqual,
                                })
                            }
                            Some(ord) => {
                                Ok(val::Value::Bool(ord == Ordering::Greater || left.eq(&right)))
                            }
                        };
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
                let value = self.interpret_expression(expr)?;
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
                let val = self.interpret_expression(expr)?;
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
            expr::Expression::Logical(left, opt, right) => {
                let l = self.interpret_expression(left)?;
                match opt {
                    expr::LogicalOperatorType::And => {
                        match l {
                            val::Value::Bool(b) => {
                                if !b {
                                    return Ok(l);
                                }
                            }
                            _ => {}
                        }
                    }
                    expr::LogicalOperatorType::Or => {
                        match l {
                            val::Value::Bool(b) => {
                                if b {
                                    return Ok(l);
                                }
                            }
                            _ => {}
                        }
                    }
                };

                return self.interpret_expression(right);
            }
            expr::Expression::Call(callee, name, args) => {
                let callee = self.interpret_expression(callee)?;
                let mut arguments = vec![];
                for a in args {
                    arguments.push(self.interpret_expression(a)?);
                }

                return match Self::cast_callable(self, &callee) {
                    None => {
                        panic!("should be callable")
                    }
                    Some(callable) => {
                        callable.call(self, arguments)
                    }
                };
            }
        }
    }
}
