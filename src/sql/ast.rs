#![allow(dead_code)]

use crate::types::Value;

/// SQL Statement
#[derive(Debug, Clone)]
pub enum Statement {
    CreateTable {
        name: String,
        columns: Vec<ColumnDef>,
    },
    Insert {
        table: String,
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expression>>,
    },
    Select {
        columns: Vec<Expression>,
        from: String,
        where_clause: Option<Expression>,
        order_by: Option<Vec<OrderByExpr>>,
        limit: Option<u64>,
    },
    Update {
        table: String,
        set: Vec<(String, Expression)>,
        where_clause: Option<Expression>,
    },
    Delete {
        table: String,
        where_clause: Option<Expression>,
    },
    Begin,
    Commit,
    Rollback,
}

/// Column definition
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<Expression>,
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Value),
    Column(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Function {
        name: String,
        args: Vec<Expression>,
    },
    In {
        expr: Box<Expression>,
        values: Vec<Expression>,
    },
    Like {
        expr: Box<Expression>,
        pattern: String,
    },
    IsNull {
        expr: Box<Expression>,
        negated: bool,
    },
    Parenthesized(Box<Expression>),
    Star,
    /// Table-qualified column (e.g., users.name)
    QualifiedColumn {
        table: String,
        column: String,
    },
}

/// Binary operator
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

/// Unary operator
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Negate,
}

/// ORDER BY expression
#[derive(Debug, Clone)]
pub struct OrderByExpr {
    pub expr: Expression,
    pub ascending: bool,
}

impl Expression {
    pub fn literal(val: Value) -> Self {
        Expression::Literal(val)
    }

    pub fn column(name: impl Into<String>) -> Self {
        Expression::Column(name.into())
    }

    pub fn eq(self, other: Expression) -> Self {
        Expression::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Eq,
            right: Box::new(other),
        }
    }

    pub fn gt(self, other: Expression) -> Self {
        Expression::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Gt,
            right: Box::new(other),
        }
    }

    pub fn and(self, other: Expression) -> Self {
        Expression::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::And,
            right: Box::new(other),
        }
    }
}
