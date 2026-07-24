#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a SQL value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Text(s) => write!(f, "'{}'", s),
            Value::Blob(b) => write!(f, "<blob {} bytes>", b.len()),
            Value::Array(a) => write!(f, "[{}]", a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            Value::Object(o) => {
                let pairs: Vec<String> = o.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}

/// Represents a row of data
pub type Row = HashMap<String, Value>;

/// Table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Integer,
    Float,
    Text,
    Blob,
    Array,
    Object,
}

impl DataType {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "BOOL" | "BOOLEAN" => Some(DataType::Boolean),
            "INT" | "INTEGER" | "BIGINT" => Some(DataType::Integer),
            "FLOAT" | "DOUBLE" | "REAL" => Some(DataType::Float),
            "TEXT" | "VARCHAR" | "STRING" => Some(DataType::Text),
            "BLOB" | "BYTES" => Some(DataType::Blob),
            "ARRAY" | "LIST" => Some(DataType::Array),
            "OBJECT" | "MAP" | "JSON" => Some(DataType::Object),
            _ => None,
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Blob => write!(f, "BLOB"),
            DataType::Array => write!(f, "ARRAY"),
            DataType::Object => write!(f, "OBJECT"),
        }
    }
}

/// Transaction ID
pub type TransactionId = u64;

/// Row ID
pub type RowId = u64;

/// Page number
pub type PageNum = u64;
