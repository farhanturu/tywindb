#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::Path;
use crate::error::Result;
use crate::types::Value;

pub struct ExportManager;

impl ExportManager {
    pub fn to_csv(rows: &[std::collections::HashMap<String, Value>], path: &Path) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        let cols: Vec<&String> = rows[0].keys().collect();
        let mut file = fs::File::create(path)?;

        writeln!(file, "{}", cols.iter().map(|c| escape_csv(c)).collect::<Vec<_>>().join(","))?;

        for row in rows {
            let values: Vec<String> = cols.iter()
                .map(|c| {
                    match row.get(*c) {
                        Some(val) => {
                            let s = match val {
                                Value::Null => String::new(),
                                Value::Boolean(b) => b.to_string(),
                                Value::Integer(i) => i.to_string(),
                                Value::Float(f) => f.to_string(),
                                Value::Text(s) => s.clone(),
                                _ => val.to_string(),
                            };
                            escape_csv(&s)
                        }
                        None => String::new(),
                    }
                })
                .collect();
            writeln!(file, "{}", values.join(","))?;
        }

        Ok(())
    }

    pub fn from_csv(path: &Path) -> Result<Vec<std::collections::HashMap<String, Value>>> {
        let content = fs::read_to_string(path)?;
        let mut rows = Vec::new();
        let mut headers = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let values = parse_csv_line(line);
            if i == 0 {
                headers = values;
            } else {
                let mut row = std::collections::HashMap::new();
                for (j, header) in headers.iter().enumerate() {
                    let value = values.get(j).cloned().unwrap_or_default();
                    row.insert(header.clone(), parse_value(&value));
                }
                rows.push(row);
            }
        }

        Ok(rows)
    }

    pub fn to_json(rows: &[std::collections::HashMap<String, Value>], path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(rows)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn from_json(path: &Path) -> Result<Vec<std::collections::HashMap<String, Value>>> {
        let content = fs::read_to_string(path)?;
        let rows: Vec<std::collections::HashMap<String, Value>> = serde_json::from_str(&content)?;
        Ok(rows)
    }

    pub fn to_sql(rows: &[std::collections::HashMap<String, Value>], table: &str) -> String {
        let mut sql = String::new();

        if let Some(first) = rows.first() {
            let cols: Vec<&String> = first.keys().collect();
            sql.push_str(&format!("INSERT INTO {} ({}) VALUES\n", table, cols.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));

            let value_sets: Vec<String> = rows.iter().map(|row| {
                let values: Vec<String> = cols.iter().map(|c| {
                    match row.get(*c) {
                        Some(val) => value_to_sql(val),
                        None => "NULL".to_string(),
                    }
                }).collect();
                format!("({})", values.join(", "))
            }).collect();

            sql.push_str(&value_sets.join(",\n"));
            sql.push(';');
        }

        sql
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                values.push(current.clone());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    values.push(current);
    values
}

fn parse_value(s: &str) -> Value {
    if s.is_empty() || s == "NULL" {
        Value::Null
    } else if let Ok(i) = s.parse::<i64>() {
        Value::Integer(i)
    } else if let Ok(f) = s.parse::<f64>() {
        Value::Float(f)
    } else if s == "true" {
        Value::Boolean(true)
    } else if s == "false" {
        Value::Boolean(false)
    } else {
        Value::Text(s.to_string())
    }
}

fn value_to_sql(val: &Value) -> String {
    match val {
        Value::Null => "NULL".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Text(s) => format!("'{}'", s.replace('\'', "''")),
        _ => format!("'{}'", val.to_string().replace('\'', "''")),
    }
}
