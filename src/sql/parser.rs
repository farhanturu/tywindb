use crate::sql::ast::{ColumnDef, Expression, OrderByExpr, Statement};
use crate::sql::tokenizer::{Token, Tokenizer};
use crate::types::Value;

pub struct SqlParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl SqlParser {
    pub fn new(sql: &str) -> Result<Self, String> {
        let mut tokenizer = Tokenizer::new(sql);
        let tokens = tokenizer.tokenize()?;

        Ok(Self { tokens, pos: 0 })
    }

    pub fn parse(&mut self) -> Result<Statement, String> {
        let token = self.peek()?;
        
        match token {
            Token::Select => self.parse_select(),
            Token::Insert => self.parse_insert(),
            Token::Update => self.parse_update(),
            Token::Delete => self.parse_delete(),
            Token::Create => self.parse_create_table(),
            Token::Begin => Ok(Statement::Begin),
            Token::Commit => Ok(Statement::Commit),
            Token::Rollback => Ok(Statement::Rollback),
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }

    fn peek(&self) -> Result<&Token, String> {
        self.tokens.get(self.pos).ok_or_else(|| "Unexpected end of input".to_string())
    }

    fn advance(&mut self) -> Result<Token, String> {
        if self.pos >= self.tokens.len() {
            return Err("Unexpected end of input".to_string());
        }
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        Ok(token)
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let token = self.advance()?;
        if &token == expected {
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, token))
        }
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.expect(&Token::Select)?;

        // Parse columns
        let columns = self.parse_expression_list()?;

        // Parse FROM
        self.expect(&Token::From)?;
        let from = self.parse_identifier()?;

        // Parse WHERE (optional)
        let where_clause = if self.peek()? == &Token::Where {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse ORDER BY (optional)
        let order_by = if self.peek()? == &Token::Order {
            self.advance()?;
            self.expect(&Token::By)?;
            Some(self.parse_order_by()?)
        } else {
            None
        };

        // Parse LIMIT (optional)
        let limit = if self.peek()? == &Token::Limit {
            self.advance()?;
            match self.advance()? {
                Token::Integer(n) => Some(n as u64),
                _ => return Err("Expected integer after LIMIT".to_string()),
            }
        } else {
            None
        };

        Ok(Statement::Select {
            columns,
            from,
            where_clause,
            order_by,
            limit,
        })
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        self.expect(&Token::Insert)?;
        self.expect(&Token::Into)?;
        let table = self.parse_identifier()?;

        // Parse column list (optional)
        let columns = if self.peek()? == &Token::LeftParen {
            self.advance()?;
            let cols = self.parse_identifier_list()?;
            self.expect(&Token::RightParen)?;
            Some(cols)
        } else {
            None
        };

        // Parse VALUES
        self.expect(&Token::Values)?;
        let mut values = Vec::new();

        loop {
            self.expect(&Token::LeftParen)?;
            let row = self.parse_expression_list()?;
            values.push(row);
            self.expect(&Token::RightParen)?;

            if self.peek()? == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        Ok(Statement::Insert {
            table,
            columns,
            values,
        })
    }

    fn parse_update(&mut self) -> Result<Statement, String> {
        self.expect(&Token::Update)?;
        let table = self.parse_identifier()?;
        self.expect(&Token::Set)?;

        let mut set = Vec::new();
        loop {
            let column = self.parse_identifier()?;
            self.expect(&Token::Eq)?;
            let value = self.parse_expression()?;
            set.push((column, value));

            if self.peek()? == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        let where_clause = if self.peek()? == &Token::Where {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Update {
            table,
            set,
            where_clause,
        })
    }

    fn parse_delete(&mut self) -> Result<Statement, String> {
        self.expect(&Token::Delete)?;
        self.expect(&Token::From)?;
        let table = self.parse_identifier()?;

        let where_clause = if self.peek()? == &Token::Where {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Delete {
            table,
            where_clause,
        })
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        self.expect(&Token::Create)?;
        self.expect(&Token::Table)?;
        let name = self.parse_identifier()?;
        self.expect(&Token::LeftParen)?;

        let mut columns = Vec::new();
        loop {
            let col = self.parse_column_def()?;
            columns.push(col);

            if self.peek()? == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        self.expect(&Token::RightParen)?;

        Ok(Statement::CreateTable { name, columns })
    }

    fn parse_column_def(&mut self) -> Result<ColumnDef, String> {
        let name = self.parse_identifier()?;
        let data_type = self.parse_identifier()?;

        let mut nullable = true;
        let mut default = None;

        // Parse constraints
        loop {
            match self.peek()? {
                Token::Not => {
                    self.advance()?;
                    if self.peek()? == &Token::Null {
                        self.advance()?;
                        nullable = false;
                    }
                }
                Token::Default => {
                    self.advance()?;
                    default = Some(self.parse_expression()?);
                }
                _ => break,
            }
        }

        Ok(ColumnDef {
            name,
            data_type,
            nullable,
            default,
        })
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expression>, String> {
        let mut exprs = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            exprs.push(expr);

            if self.peek()? == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        Ok(exprs)
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and()?;

        while self.peek()? == &Token::Or {
            self.advance()?;
            let right = self.parse_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: crate::sql::ast::BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;

        while self.peek()? == &Token::And {
            self.advance()?;
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: crate::sql::ast::BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_addition()?;

        loop {
            let op = match self.peek()? {
                Token::Eq => crate::sql::ast::BinaryOperator::Eq,
                Token::Ne => crate::sql::ast::BinaryOperator::Ne,
                Token::Lt => crate::sql::ast::BinaryOperator::Lt,
                Token::Le => crate::sql::ast::BinaryOperator::Le,
                Token::Gt => crate::sql::ast::BinaryOperator::Gt,
                Token::Ge => crate::sql::ast::BinaryOperator::Ge,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_addition()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;

        loop {
            let op = match self.peek()? {
                Token::Plus => crate::sql::ast::BinaryOperator::Add,
                Token::Minus => crate::sql::ast::BinaryOperator::Subtract,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_multiplication()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek()? {
                Token::Star => crate::sql::ast::BinaryOperator::Multiply,
                Token::Slash => crate::sql::ast::BinaryOperator::Divide,
                Token::Percent => crate::sql::ast::BinaryOperator::Modulo,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        match self.peek()? {
            Token::Not => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: crate::sql::ast::UnaryOperator::Not,
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: crate::sql::ast::UnaryOperator::Negate,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.peek()?.clone() {
            Token::Integer(n) => {
                self.advance()?;
                Ok(Expression::Literal(Value::Integer(n)))
            }
            Token::Float(f) => {
                self.advance()?;
                Ok(Expression::Literal(Value::Float(f)))
            }
            Token::String(s) => {
                self.advance()?;
                Ok(Expression::Literal(Value::Text(s)))
            }
            Token::Boolean(b) => {
                self.advance()?;
                Ok(Expression::Literal(Value::Boolean(b)))
            }
            Token::Null => {
                self.advance()?;
                Ok(Expression::Literal(Value::Null))
            }
            Token::Star => {
                self.advance()?;
                Ok(Expression::Star)
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                Ok(Expression::Parenthesized(Box::new(expr)))
            }
            Token::Identifier(name) => {
                self.advance()?;
                // Check if it's a table.column
                if self.peek()? == &Token::Dot {
                    self.advance()?;
                    let column = self.parse_identifier()?;
                    Ok(Expression::QualifiedColumn { table: name, column })
                } else {
                    Ok(Expression::Column(name))
                }
            }
            _ => Err(format!("Unexpected token: {:?}", self.peek()?)),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.advance()? {
            Token::Identifier(name) => Ok(name),
            token => Err(format!("Expected identifier, got {:?}", token)),
        }
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<String>, String> {
        let mut ids = Vec::new();

        loop {
            let id = self.parse_identifier()?;
            ids.push(id);

            if self.peek()? == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        Ok(ids)
    }

    fn parse_order_by(&mut self) -> Result<Vec<OrderByExpr>, String> {
        let mut exprs = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            let ascending = if self.peek()? == &Token::Asc {
                self.advance()?;
                true
            } else if self.peek()? == &Token::Desc {
                self.advance()?;
                false
            } else {
                true
            };

            exprs.push(OrderByExpr { expr, ascending });

            if self.peek()? == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        Ok(exprs)
    }
}
