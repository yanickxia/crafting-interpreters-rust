use crate::types::{expr, token};

pub struct Parser {
    tokens: Vec<token::Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<token::Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // http://www.craftinginterpreters.com/appendix-i.html
    pub fn parse(&mut self) -> Result<Vec<expr::Statement>, expr::ExpError> {
        let mut statements = vec![];
        while !self.at_end() {
            let statement = self.declaration()?;
            statements.push(statement)
        }

        return Ok(statements);
    }

    pub fn declaration(&mut self) -> Result<expr::Statement, expr::ExpError> {
        if self.match_token(vec![token::TokenType::Class]) {
            return self.class();
        }
        if self.match_token(vec![token::TokenType::Fun]) {
            return self.function("function");
        }
        if self.match_token(vec![token::TokenType::Var]) {
            return self.var_declaration();
        }
        return self.statement();
    }

    pub fn class(&mut self) -> Result<expr::Statement, expr::ExpError> {
        let name = self.consume(token::TokenType::Identifier, "Expect class name.")?.clone();

        let mut super_class = None;
        if self.match_token(vec![token::TokenType::Less]) {
            self.consume(token::TokenType::Identifier, "Expect superclass name.")?;
            super_class = Some(self.previous().clone().lexeme)
        }

        self.consume(token::TokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods = vec![];
        while !self.check(token::TokenType::RightBrace) && !self.at_end() {
            methods.push(self.function("method")?);
        }
        self.consume(token::TokenType::RightBrace, "Expect '}' before class body.")?;
        return Ok(expr::Statement::Class {
            name: name.lexeme,
            methods,
            super_class,
        });
    }

    pub fn function(&mut self, kind: &str) -> Result<expr::Statement, expr::ExpError> {
        let name = self.consume(token::TokenType::Identifier, format!("{} {} {}", "Expect", kind, "name").as_str())?.clone();
        self.consume(token::TokenType::LeftParen, format!("{} {} {}", "Expect '{' before", kind, "name").as_str())?;
        let mut parameters = vec![];

        if !self.check(token::TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(expr::ExpError::TooManyArgs);
                }
                parameters.push(self.consume(token::TokenType::Identifier, "Expect parameter name.")?.clone().lexeme);
                if !self.match_token(vec![token::TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(token::TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(token::TokenType::LeftBrace, format!("{} {} {}", "Expect '{' before", kind, "name").as_str())?;

        let body = self.block()?;
        return Ok(expr::Statement::Function(name.lexeme.clone(), parameters, Box::new(body)));
    }

    pub fn var_declaration(&mut self) -> Result<expr::Statement, expr::ExpError> {
        let name = self.consume(token::TokenType::Identifier, "Expect variable name.")?.clone();
        let mut initializer = expr::Expression::Literal(expr::Literal::Nil);
        if self.match_token(vec![token::TokenType::Equal]) {
            initializer = self.expression()?;
        }
        self.consume(token::TokenType::Semicolon, "Expect ';' after expression.")?;
        return Ok(expr::Statement::Var(name.lexeme.to_string(), initializer));
    }


    pub fn statement(&mut self) -> Result<expr::Statement, expr::ExpError> {
        if self.match_token(vec![token::TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_token(vec![token::TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_token(vec![token::TokenType::While]) {
            return self.while_statement();
        }
        if self.match_token(vec![token::TokenType::For]) {
            return self.for_statement();
        }
        if self.match_token(vec![token::TokenType::LeftBrace]) {
            return self.block();
        }
        if self.match_token(vec![token::TokenType::If]) {
            return self.if_statement();
        }
        return self.expression_statement();
    }

    pub fn return_statement(&mut self) -> Result<expr::Statement, expr::ExpError> {
        let token = self.previous().clone();
        let mut expr = None;

        if !self.check(token::TokenType::Semicolon) {
            expr = Some(self.expression()?)
        }

        self.consume(token::TokenType::Semicolon, "Expect ';' after return expression.")?;

        Ok(expr::Statement::Return(token.lexeme.to_string(), expr))
    }

    pub fn for_statement(&mut self) -> Result<expr::Statement, expr::ExpError> {
        self.consume(token::TokenType::LeftParen, "Expect '(' after for expression.")?;

        // initializer
        let mut initializer = None;
        if self.match_token(vec![token::TokenType::Semicolon])
        {} else if self.match_token(vec![token::TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        // condition
        let mut condition = expr::Expression::Literal(expr::Literal::True);
        if !self.check(token::TokenType::Semicolon) {
            condition = self.expression()?
        }
        self.consume(token::TokenType::Semicolon, "Expect ';' after loop expression.")?;

        let mut increment = None;
        if !self.check(token::TokenType::RightParen) {
            increment = Some(self.expression()?)
        }
        self.consume(token::TokenType::RightParen, "Expect ')' after for expression.")?;

        let mut body = self.statement()?;
        match increment {
            None => {}
            Some(inc) => {
                body = expr::Statement::Block(vec![body, expr::Statement::Expression(inc)])
            }
        }

        body = expr::Statement::While(condition, Box::new(body));

        match initializer {
            None => {}
            Some(init) => {
                body = expr::Statement::Block(vec![init, body])
            }
        }

        return Ok(body);
    }

    pub fn while_statement(&mut self) -> Result<expr::Statement, expr::ExpError> {
        self.consume(token::TokenType::LeftParen, "Expect '(' after while expression.")?;
        let condition = self.expression()?;
        self.consume(token::TokenType::RightParen, "Expect ')' after while expression.")?;
        let body = self.statement()?;
        Ok(expr::Statement::While(condition, Box::new(body)))
    }

    pub fn if_statement(&mut self) -> Result<expr::Statement, expr::ExpError> {
        self.consume(token::TokenType::LeftParen, "Expect '(' after if expression.")?;
        let condition = self.expression()?;
        self.consume(token::TokenType::RightParen, "Expect ')' after if expression.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.match_token(vec![token::TokenType::Else]) {
            let else_statement = self.statement()?;
            else_branch = Some(Box::new(else_statement))
        }

        return Ok(expr::Statement::If(condition, Box::new(then_branch), else_branch));
    }

    pub fn block(&mut self) -> Result<expr::Statement, expr::ExpError> {
        let mut statements = vec![];
        while !self.check(token::TokenType::RightBrace) && !self.at_end() {
            let statement = self.declaration()?;
            statements.push(statement)
        }
        self.consume(token::TokenType::RightBrace, "Expect '}' after expression.")?;
        return Ok(expr::Statement::Block(statements));
    }

    pub fn print_statement(&mut self) -> Result<expr::Statement, expr::ExpError> {
        let expr = self.expression()?;
        self.consume(token::TokenType::Semicolon, "Expect ';' after expression.")?;

        return Ok(expr::Statement::Print(expr));
    }

    pub fn expression_statement(&mut self) -> Result<expr::Statement, expr::ExpError> {
        let expr = self.expression()?;
        self.consume(token::TokenType::Semicolon, "Expect ';' after expression.")?;
        return Ok(expr::Statement::Expression(expr));
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.at_end() {
            if self.previous().token_type == token::TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                token::TokenType::Class | token::TokenType::Fun | token::TokenType::Var | token::TokenType::For |
                token::TokenType::If | token::TokenType::While | token::TokenType::Print | token::TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }


    fn expression(&mut self) -> Result<expr::Expression, expr::ExpError> {
        return self.assignment();
    }

    fn assignment(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let expr = self.or()?;
        if self.match_token(vec![token::TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            return match expr {
                expr::Expression::Variable(token) => {
                    Ok(expr::Expression::Assign(token, Box::new(value)))
                }
                expr::Expression::Get {
                    object, variable
                } => {
                    Ok(expr::Expression::Set {
                        object,
                        variable,
                        value: Box::new(value),
                    })
                }
                _ => {
                    Err(expr::ExpError::AssignmentFailed {
                        name: equals.lexeme.to_string()
                    })
                }
            };
        }

        return Ok(expr);
    }

    fn or(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let mut expr = self.and()?;
        while self.match_token(vec![token::TokenType::Or]) {
            let right = self.and()?;
            expr = expr::Expression::Logical(Box::new(expr), expr::LogicalOperatorType::Or, Box::new(right))
        }
        return Ok(expr);
    }

    fn and(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let mut expr = self.equality()?;
        while self.match_token(vec![token::TokenType::And]) {
            let right = self.equality()?;
            expr = expr::Expression::Logical(Box::new(expr), expr::LogicalOperatorType::And, Box::new(right))
        }
        return Ok(expr);
    }


    fn equality(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let mut expr = self.comparison()?;
        while self.match_token(vec![token::TokenType::BangEqual, token::TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = expr::Expression::Binary(Box::new(expr), expr::BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let mut expr = self.term()?;
        while self.match_token(vec![token::TokenType::Greater, token::TokenType::GreaterEqual,
                                    token::TokenType::Less, token::TokenType::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expr = expr::Expression::Binary(Box::new(expr), expr::BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn term(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let mut expr = self.factor()?;
        while self.match_token(vec![token::TokenType::Minus, token::TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = expr::Expression::Binary(Box::new(expr), expr::BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn factor(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let mut expr = self.unary()?;
        while self.match_token(vec![token::TokenType::Slash, token::TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = expr::Expression::Binary(Box::new(expr), expr::BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn unary(&mut self) -> Result<expr::Expression, expr::ExpError> {
        while self.match_token(vec![token::TokenType::Bang, token::TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(expr::Expression::Unary(expr::UnaryOp {
                token_type: Self::token_to_unary_token_type(&operator)?
            }, Box::new(right)));
        }
        return self.call();
    }


    fn call(&mut self) -> Result<expr::Expression, expr::ExpError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(vec![token::TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(vec![token::TokenType::Dot]) {
                let variable = self.consume(token::TokenType::Identifier, "Expect property name after '.'.")?.clone();
                expr = expr::Expression::Get {
                    object: Box::new(expr),
                    variable: variable.lexeme.to_string(),
                }
            } else {
                break;
            }
        }

        return Ok(expr);
    }

    fn finish_call(&mut self, callee: expr::Expression) -> Result<expr::Expression, expr::ExpError> {
        let mut arguments = vec![];
        if !self.check(token::TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(expr::ExpError::TooManyArgs);
                }

                arguments.push(self.expression()?);
                if !self.match_token(vec![token::TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(token::TokenType::RightParen, "Expect ')' after arguments.")?;
        return Ok(expr::Expression::Call(Box::new(callee), paren.lexeme.to_string(), arguments));
    }


    fn primary(&mut self) -> Result<expr::Expression, expr::ExpError> {
        if self.match_token(vec![token::TokenType::False]) {
            return Ok(expr::Expression::Literal(expr::Literal::False));
        }

        if self.match_token(vec![token::TokenType::True]) {
            return Ok(expr::Expression::Literal(expr::Literal::True));
        }

        if self.match_token(vec![token::TokenType::Nil]) {
            return Ok(expr::Expression::Literal(expr::Literal::Nil));
        }

        if self.match_token(vec![token::TokenType::Number]) {
            match &self.previous().literal {
                Some(token::Literal::Number(n)) => {
                    return Ok(expr::Expression::Literal(expr::Literal::Number(*n)));
                }
                Some(l) => panic!(
                    "internal error in parser: when parsing number, found  expr::Literal {:?}",
                    l
                ),
                None => panic!("internal error in parser: when parsing number, found no  expr::Literal"),
            }
        }

        if self.match_token(vec![token::TokenType::String]) {
            match &self.previous().literal {
                Some(token::Literal::Str(str)) => {
                    return Ok(expr::Expression::Literal(expr::Literal::String(str.to_string())));
                }
                Some(l) => panic!(
                    "internal error in parser: when parsing string, found  expr::Literal {:?}",
                    l
                ),
                None => panic!("internal error in parser: when parsing string, found no  expr::Literal"),
            }
        }

        if self.match_token(vec![token::TokenType::Identifier]) {
            return Ok(expr::Expression::Variable(self.previous().lexeme.to_string()));
        }

        if self.match_token(vec![token::TokenType::Super]) {
            let keyword = self.previous().lexeme.to_string();
            self.consume(token::TokenType::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(token::TokenType::Identifier, "Expect 'method' after 'super.'.")?.lexeme.clone();
            return Ok(expr::Expression::Super { keyword, method });
        }

        if self.match_token(vec![token::TokenType::This]) {
            return Ok(expr::Expression::This(self.previous().lexeme.to_string()));
        }

        if self.match_token(vec![token::TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(token::TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(expr::Expression::Grouping(Box::new(expr)));
        }

        return Err(expr::ExpError::ExpectedExpression {
            token_type: self.peek().token_type,
            line: self.peek().line,
        });
    }

    fn consume(&mut self, ty: token::TokenType, message: &str) -> Result<&token::Token, expr::ExpError> {
        if self.check(ty) {
            return Ok(self.advance());
        }
        return Err(expr::ExpError::TokenMismatch {
            expected: ty.clone(),
            found: self.previous().clone(),
            err_string: Some(message.to_string()),
        });
    }


    fn match_token(&mut self, token_types: Vec<token::TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, token_type: token::TokenType) -> bool {
        if self.at_end() {
            return false;
        }
        return self.peek().token_type.eq(&token_type);
    }

    fn advance(&mut self) -> &token::Token {
        if !self.at_end() {
            self.current += 1
        }
        return self.previous();
    }

    fn at_end(&mut self) -> bool {
        return self.peek().token_type == token::TokenType::Eof;
    }

    fn peek(&self) -> &token::Token {
        return &self.tokens[self.current];
    }

    fn previous(&mut self) -> &token::Token {
        return &self.tokens[self.current - 1];
    }

    fn token_to_binary_token_type(token: &token::Token) -> Result<expr::BinaryOperatorType, expr::ExpError> {
        match token.token_type {
            token::TokenType::BangEqual => Ok(expr::BinaryOperatorType::NotEqual),
            token::TokenType::EqualEqual => Ok(expr::BinaryOperatorType::EqualEqual),
            token::TokenType::Less => Ok(expr::BinaryOperatorType::Less),
            token::TokenType::LessEqual => Ok(expr::BinaryOperatorType::LessEqual),
            token::TokenType::Greater => Ok(expr::BinaryOperatorType::Greater),
            token::TokenType::GreaterEqual => Ok(expr::BinaryOperatorType::GreaterEqual),
            token::TokenType::Plus => Ok(expr::BinaryOperatorType::Plus),
            token::TokenType::Minus => Ok(expr::BinaryOperatorType::Minus),
            token::TokenType::Star => Ok(expr::BinaryOperatorType::Star),
            token::TokenType::Slash => Ok(expr::BinaryOperatorType::Slash),
            _ => Err(expr::ExpError::ConvertFailed {
                expected: vec![token::TokenType::BangEqual, token::TokenType::EqualEqual, token::TokenType::Less,
                               token::TokenType::LessEqual, token::TokenType::Greater, token::TokenType::GreaterEqual,
                               token::TokenType::Plus, token::TokenType::Minus, token::TokenType::Star, token::TokenType::Slash],
                found: token.clone(),
            }),
        }
    }

    fn token_to_unary_token_type(token: &token::Token) -> Result<expr::UnaryOperatorType, expr::ExpError> {
        match token.token_type {
            token::TokenType::Minus => Ok(expr::UnaryOperatorType::Minus),
            token::TokenType::Bang => Ok(expr::UnaryOperatorType::Bang),
            _ => Err(expr::ExpError::ConvertFailed {
                expected: vec![token::TokenType::Minus, token::TokenType::Bang],
                found: token.clone(),
            }),
        }
    }
}
