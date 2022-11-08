use crate::types::{expr, token};

pub struct Parser {
    tokens: Vec<token::Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<token::Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<expr::Statement>, expr::ExpError> {
        let mut statements = vec![];
        while !self.at_end() {
            let statement = self.declaration()?;
            statements.push(statement)
        }

        return Ok(statements);
    }

    pub fn declaration(&mut self) -> Result<expr::Statement, expr::ExpError> {
        if self.match_token(vec![token::TokenType::Var]) {
            return self.var_declaration();
        }
        return self.statement();
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
        return self.expression_statement();
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
        let expr = self.equality()?;
        if self.match_token(vec![token::TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            return match expr {
                expr::Expression::Variable(token) => {
                    Ok(expr::Expression::Assign(token, Box::new(value)))
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
        return self.primary();
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
