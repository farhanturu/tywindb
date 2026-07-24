#![allow(dead_code)]

use std::collections::HashMap;

use crate::error::{Result, TywindbError};
use crate::sql::ast::{Expression, Statement};
use crate::storage::engine::StorageEngine;
use crate::storage::wal::WalOp;
use crate::transaction::TransactionManager;
use crate::types::{Row, Value};

/// Query result
#[derive(Debug)]
pub enum QueryResult {
    /// No rows returned (INSERT, UPDATE, DELETE)
    Empty,
    /// Rows returned (SELECT)
    Rows(Vec<Row>),
    /// Table created
    TableCreated,
    /// Transaction started
    TransactionStarted { tx_id: u64 },
    /// Transaction committed
    TransactionCommitted,
    /// Transaction rolled back
    TransactionRolledBack,
    /// Number of rows affected
    RowsAffected(usize),
}

/// Query executor
pub struct Executor {
    storage: StorageEngine,
    tx_manager: TransactionManager,
    tables: HashMap<String, Vec<Row>>,
    current_tx: Option<u64>,
}

impl Executor {
    pub fn new(storage: StorageEngine) -> Self {
        Self {
            storage,
            tx_manager: TransactionManager::new(),
            tables: HashMap::new(),
            current_tx: None,
        }
    }

    pub fn tables(&self) -> &HashMap<String, Vec<Row>> {
        &self.tables
    }

    pub fn tables_mut(&mut self) -> &mut HashMap<String, Vec<Row>> {
        &mut self.tables
    }

    pub fn execute(&mut self, stmt: Statement) -> Result<QueryResult> {
        match stmt {
            Statement::Select {
                columns,
                from,
                where_clause,
                order_by,
                limit,
            } => self.execute_select(&from, &columns, where_clause.as_ref(), order_by.as_ref(), limit),

            Statement::Insert {
                table,
                columns,
                values,
            } => self.execute_insert(&table, columns.as_ref(), &values),

            Statement::Update {
                table,
                set,
                where_clause,
            } => self.execute_update(&table, &set, where_clause.as_ref()),

            Statement::Delete {
                table,
                where_clause,
            } => self.execute_delete(&table, where_clause.as_ref()),

            Statement::CreateTable { name, columns } => self.execute_create_table(&name, &columns),

            Statement::Begin => {
                let tx_id = self.tx_manager.begin();
                self.current_tx = Some(tx_id);
                self.storage.write_wal(tx_id, WalOp::Begin)?;
                Ok(QueryResult::TransactionStarted { tx_id })
            }

            Statement::Commit => {
                if let Some(tx_id) = self.current_tx {
                    self.tx_manager.commit(tx_id)?;
                    self.storage.write_wal(tx_id, WalOp::Commit)?;
                    self.storage.sync_wal()?;
                    self.current_tx = None;
                    Ok(QueryResult::TransactionCommitted)
                } else {
                    Err(TywindbError::Transaction("No active transaction".to_string()))
                }
            }

            Statement::Rollback => {
                if let Some(tx_id) = self.current_tx {
                    self.tx_manager.abort(tx_id)?;
                    self.storage.write_wal(tx_id, WalOp::Abort)?;
                    self.current_tx = None;
                    Ok(QueryResult::TransactionRolledBack)
                } else {
                    Err(TywindbError::Transaction("No active transaction".to_string()))
                }
            }
        }
    }

    fn execute_select(
        &self,
        table: &str,
        _columns: &[Expression],
        where_clause: Option<&Expression>,
        order_by: Option<&Vec<crate::sql::ast::OrderByExpr>>,
        limit: Option<u64>,
    ) -> Result<QueryResult> {
        let rows = self.tables.get(table).cloned().unwrap_or_default();

        // Filter rows
        let filtered: Vec<Row> = if let Some(where_expr) = where_clause {
            rows.into_iter()
                .filter(|row| self.evaluate_where(row, where_expr).unwrap_or(false))
                .collect()
        } else {
            rows
        };

        // Apply ORDER BY
        let sorted = if let Some(order) = order_by {
            let mut sorted = filtered;
            if let Some(first_order) = order.first() {
                sorted.sort_by(|a, b| {
                    let val_a = self.evaluate_expression(a, &first_order.expr);
                    let val_b = self.evaluate_expression(b, &first_order.expr);
                    match (val_a, val_b) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if first_order.ascending { a.cmp(&b) } else { b.cmp(&a) }
                        }
                        (Value::Float(a), Value::Float(b)) => {
                            if first_order.ascending { a.partial_cmp(&b).unwrap() } else { b.partial_cmp(&a).unwrap() }
                        }
                        (Value::Text(a), Value::Text(b)) => {
                            if first_order.ascending { a.cmp(&b) } else { b.cmp(&a) }
                        }
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            }
            sorted
        } else {
            filtered
        };

        // Apply LIMIT
        let result = if let Some(limit) = limit {
            sorted.into_iter().take(limit as usize).collect()
        } else {
            sorted
        };

        Ok(QueryResult::Rows(result))
    }

    fn execute_insert(
        &mut self,
        table: &str,
        columns: Option<&Vec<String>>,
        values: &Vec<Vec<Expression>>,
    ) -> Result<QueryResult> {
        // Build all rows first, then insert them
        let mut new_rows = Vec::new();

        for row_values in values {
            let mut row = Row::new();

            if let Some(cols) = columns {
                for (i, col_name) in cols.iter().enumerate() {
                    let value = if let Some(val) = row_values.get(i) {
                        self.evaluate_expression(&Row::new(), val)
                    } else {
                        Value::Null
                    };
                    row.insert(col_name.clone(), value);
                }
            } else {
                for (i, val) in row_values.iter().enumerate() {
                    let value = self.evaluate_expression(&Row::new(), val);
                    row.insert(format!("col_{}", i), value);
                }
            }

            new_rows.push(row);
        }

        let rows_affected = new_rows.len();
        
        // Now insert into the table
        let table_rows = self.tables.entry(table.to_string()).or_default();
        table_rows.extend(new_rows);

        Ok(QueryResult::RowsAffected(rows_affected))
    }

    fn execute_update(
        &mut self,
        table: &str,
        set: &Vec<(String, Expression)>,
        where_clause: Option<&Expression>,
    ) -> Result<QueryResult> {
        // Clone the table to avoid borrow issues
        let table_data = self.tables.get(table).cloned().unwrap_or_default();
        
        let mut rows_affected = 0;
        let mut updated_rows = Vec::new();

        for row in table_data {
            let should_update = if let Some(where_expr) = where_clause {
                self.evaluate_where(&row, where_expr).unwrap_or(false)
            } else {
                true
            };

            if should_update {
                let mut new_row = row.clone();
                for (col_name, val_expr) in set {
                    let value = self.evaluate_expression(&row, val_expr);
                    new_row.insert(col_name.clone(), value);
                }
                updated_rows.push(new_row);
                rows_affected += 1;
            } else {
                updated_rows.push(row);
            }
        }

        self.tables.insert(table.to_string(), updated_rows);
        Ok(QueryResult::RowsAffected(rows_affected))
    }

    fn execute_delete(
        &mut self,
        table: &str,
        where_clause: Option<&Expression>,
    ) -> Result<QueryResult> {
        // Clone the table to avoid borrow issues
        let table_data = self.tables.get(table).cloned().unwrap_or_default();
        
        let original_len = table_data.len();

        let filtered = if let Some(where_expr) = where_clause {
            let where_clone = where_expr.clone();
            table_data.into_iter()
                .filter(|row| !self.evaluate_where(row, &where_clone).unwrap_or(false))
                .collect()
        } else {
            Vec::new()
        };

        let rows_affected = original_len - filtered.len();
        self.tables.insert(table.to_string(), filtered);
        
        Ok(QueryResult::RowsAffected(rows_affected))
    }

    fn execute_create_table(&mut self, name: &str, _columns: &[crate::sql::ast::ColumnDef]) -> Result<QueryResult> {
        self.tables.entry(name.to_string()).or_default();
        Ok(QueryResult::TableCreated)
    }

    fn evaluate_where(&self, row: &Row, expr: &Expression) -> Result<bool> {
        let value = self.evaluate_expression(row, expr);
        match value {
            Value::Boolean(b) => Ok(b),
            Value::Integer(i) => Ok(i != 0),
            Value::Float(f) => Ok(f != 0.0),
            Value::Text(s) => Ok(!s.is_empty()),
            Value::Null => Ok(false),
            _ => Ok(true),
        }
    }

    fn evaluate_expression(&self, row: &Row, expr: &Expression) -> Value {
        match expr {
            Expression::Literal(val) => val.clone(),
            Expression::Column(name) => row.get(name).cloned().unwrap_or(Value::Null),
            Expression::QualifiedColumn { table: _, column } => {
                row.get(column).cloned().unwrap_or(Value::Null)
            }
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(row, left);
                let right_val = self.evaluate_expression(row, right);
                self.evaluate_binary_op(&left_val, op, &right_val)
            }
            Expression::UnaryOp { op, expr } => {
                let val = self.evaluate_expression(row, expr);
                self.evaluate_unary_op(op, &val)
            }
            Expression::Star => Value::Null,
            _ => Value::Null,
        }
    }

    fn evaluate_binary_op(&self, left: &Value, op: &crate::sql::ast::BinaryOperator, right: &Value) -> Value {
        use crate::sql::ast::BinaryOperator;

        match op {
            BinaryOperator::Eq => Value::Boolean(self.values_equal(left, right)),
            BinaryOperator::Ne => Value::Boolean(!self.values_equal(left, right)),
            BinaryOperator::Lt => Value::Boolean(self.values_compare(left, right) == std::cmp::Ordering::Less),
            BinaryOperator::Le => Value::Boolean(self.values_compare(left, right) != std::cmp::Ordering::Greater),
            BinaryOperator::Gt => Value::Boolean(self.values_compare(left, right) == std::cmp::Ordering::Greater),
            BinaryOperator::Ge => Value::Boolean(self.values_compare(left, right) != std::cmp::Ordering::Less),
            BinaryOperator::And => {
                let l = self.value_to_bool(left);
                let r = self.value_to_bool(right);
                Value::Boolean(l && r)
            }
            BinaryOperator::Or => {
                let l = self.value_to_bool(left);
                let r = self.value_to_bool(right);
                Value::Boolean(l || r)
            }
            BinaryOperator::Add => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a + b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 + b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a + *b as f64),
                    _ => Value::Null,
                }
            }
            BinaryOperator::Subtract => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 - b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a - *b as f64),
                    _ => Value::Null,
                }
            }
            BinaryOperator::Multiply => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a * b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 * b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a * *b as f64),
                    _ => Value::Null,
                }
            }
            BinaryOperator::Divide => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b == 0 { Value::Null } else { Value::Integer(a / b) }
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if *b == 0.0 { Value::Null } else { Value::Float(a / b) }
                    }
                    _ => Value::Null,
                }
            }
            BinaryOperator::Modulo => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b == 0 { Value::Null } else { Value::Integer(a % b) }
                    }
                    _ => Value::Null,
                }
            }
        }
    }

    fn evaluate_unary_op(&self, op: &crate::sql::ast::UnaryOperator, val: &Value) -> Value {
        match op {
            crate::sql::ast::UnaryOperator::Not => Value::Boolean(!self.value_to_bool(val)),
            crate::sql::ast::UnaryOperator::Negate => {
                match val {
                    Value::Integer(i) => Value::Integer(-i),
                    Value::Float(f) => Value::Float(-f),
                    _ => Value::Null,
                }
            }
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            _ => false,
        }
    }

    fn values_compare(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }

    fn value_to_bool(&self, val: &Value) -> bool {
        match val {
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Text(s) => !s.is_empty(),
            _ => true,
        }
    }
}
