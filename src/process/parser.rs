use crate::types::{expr, token};
use crate::types::expr::{BinaryOp, BinaryOperatorType, Expression, Literal, UnaryOp, UnaryOperatorType};
use crate::types::token::{Token, TokenType};
use crate::types::token::TokenType::{Bang, BangEqual, Eof, EqualEqual, False, Greater, GreaterEqual, LeftParen, Less, LessEqual, Minus, Nil, Number, Plus, RightParen, Semicolon, Slash, Star, String, True};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expression, expr::ExpError> {
        return self.expression();
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.at_end() {
            if self.previous().token_type == Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For |
                TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }


    fn expression(&mut self) -> Result<Expression, expr::ExpError> {
        return self.equality();
    }

    /**
    expression     → equality ;
    equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    term           → factor ( ( "-" | "+" ) factor )* ;
    factor         → unary ( ( "/" | "*" ) unary )* ;
    unary          → ( "!" | "-" ) unary
                   | primary ;
    primary        → NUMBER | STRING | "true" | "false" | "nil"
                   | "(" expression ")" ;
    **/
    fn equality(&mut self) -> Result<Expression, expr::ExpError> {
        let mut expr = self.comparison()?;
        while self.match_token(vec![BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expression::Binary(Box::new(expr), BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expression, expr::ExpError> {
        let mut expr = self.term()?;
        while self.match_token(vec![Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expr = Expression::Binary(Box::new(expr), BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expression, expr::ExpError> {
        let mut expr = self.factor()?;
        while self.match_token(vec![Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expression::Binary(Box::new(expr), BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expression, expr::ExpError> {
        let mut expr = self.unary()?;
        while self.match_token(vec![Slash, Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expression::Binary(Box::new(expr), BinaryOp {
                token_type: Self::token_to_binary_token_type(&operator)?,
            }, Box::new(right))
        }
        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expression, expr::ExpError> {
        while self.match_token(vec![Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expression::Unary(UnaryOp {
                token_type: Self::token_to_unary_token_type(&operator)?
            }, Box::new(right)));
        }
        return self.primary();
    }

    fn primary(&mut self) -> Result<Expression, expr::ExpError> {
        if self.match_token(vec![False]) {
            return Ok(Expression::Literal(Literal::False));
        }

        if self.match_token(vec![True]) {
            return Ok(Expression::Literal(Literal::True));
        }

        if self.match_token(vec![Nil]) {
            return Ok(Expression::Literal(Literal::Nil));
        }

        if self.match_token(vec![Number]) {
            match &self.previous().literal {
                Some(token::Literal::Number(n)) => {
                    return Ok(Expression::Literal(Literal::Number(*n)));
                }
                Some(l) => panic!(
                    "internal error in parser: when parsing number, found literal {:?}",
                    l
                ),
                None => panic!("internal error in parser: when parsing number, found no literal"),
            }
        }

        if self.match_token(vec![String]) {
            match &self.previous().literal {
                Some(token::Literal::Str(str)) => {
                    return Ok(Expression::Literal(Literal::String(str.to_string())));
                }
                Some(l) => panic!(
                    "internal error in parser: when parsing string, found literal {:?}",
                    l
                ),
                None => panic!("internal error in parser: when parsing string, found no literal"),
            }
        }

        if self.match_token(vec![LeftParen]) {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ')' after expression.")?;
            return Ok(Expression::Grouping(Box::new(expr)));
        }

        return Err(expr::ExpError::ExpectedExpression {
            token_type: self.peek().token_type,
            line: self.peek().line,
        });
    }

    fn consume(&mut self, ty: TokenType, message: &str) -> Result<&Token, expr::ExpError> {
        if self.check(ty) {
            return Ok(self.advance());
        }
        return Err(expr::ExpError::TokenMismatch {
            expected: ty.clone(),
            found: self.previous().clone(),
            err_string: Some(message.to_string()),
        });
    }


    fn match_token(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.at_end() {
            return false;
        }
        return self.peek().token_type.eq(&token_type);
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1
        }
        return self.previous();
    }

    fn at_end(&mut self) -> bool {
        return self.peek().token_type == Eof;
    }

    fn peek(&self) -> &Token {
        return &self.tokens[self.current];
    }

    fn previous(&mut self) -> &Token {
        return &self.tokens[self.current - 1];
    }

    fn token_to_binary_token_type(token: &Token) -> Result<BinaryOperatorType, expr::ExpError> {
        match token.token_type {
            BangEqual => Ok(BinaryOperatorType::NotEqual),
            EqualEqual => Ok(BinaryOperatorType::EqualEqual),
            Less => Ok(BinaryOperatorType::Less),
            LessEqual => Ok(BinaryOperatorType::LessEqual),
            Greater => Ok(BinaryOperatorType::Greater),
            GreaterEqual => Ok(BinaryOperatorType::GreaterEqual),
            Plus => Ok(BinaryOperatorType::Plus),
            Minus => Ok(BinaryOperatorType::Minus),
            Star => Ok(BinaryOperatorType::Star),
            Slash => Ok(BinaryOperatorType::Slash),
            _ => Err(expr::ExpError::ConvertFailed {
                expected: vec![BangEqual, EqualEqual, Less, LessEqual, Greater, GreaterEqual, Plus, Minus, Star, Slash],
                found: token.clone(),
            }),
        }
    }

    fn token_to_unary_token_type(token: &Token) -> Result<UnaryOperatorType, expr::ExpError> {
        match token.token_type {
            Minus => Ok(UnaryOperatorType::Minus),
            Bang => Ok(UnaryOperatorType::Bang),
            _ => Err(expr::ExpError::ConvertFailed {
                expected: vec![Minus, Bang],
                found: token.clone(),
            }),
        }
    }
}
