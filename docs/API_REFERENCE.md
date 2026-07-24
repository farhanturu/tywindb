# Tywindb API Reference

This document describes the Rust API for Tywindb.

---

## Table of Contents

- [Database](#database)
- [Query Result](#query-result)
- [Value](#value)
- [Error Handling](#error-handling)
- [Examples](#examples)

---

## Database

The main entry point for using Tywindb.

### Opening a Database

```rust
use tywindb::Database;

// Open or create a database
let mut db = Database::open("mydb.tdb")?;
```

### Executing Queries

```rust
use tywindb::{Database, QueryResult};

let mut db = Database::open("mydb.tdb")?;

// Execute a query
let result = db.query("SELECT * FROM users")?;

// Handle different result types
match result {
    QueryResult::Rows(rows) => {
        for row in rows {
            println!("{:?}", row);
        }
    }
    QueryResult::RowsAffected(n) => {
        println!("{} rows affected", n);
    }
    QueryResult::TableCreated => {
        println!("Table created");
    }
    _ => {}
}
```

### Batch Execution

```rust
let results = db.execute_batch("
    CREATE TABLE users (id INTEGER, name TEXT);
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    SELECT * FROM users;
")?;

for result in results {
    println!("{:?}", result);
}
```

### Transactions

```rust
// Begin transaction
db.begin()?;

// Execute operations
db.query("INSERT INTO users (id, name) VALUES (2, 'Bob')")?;

// Commit or rollback
db.commit()?;
// or
db.rollback()?;
```

### Closing the Database

```rust
db.close()?;
```

---

## Query Result

Represents the result of a SQL query.

### Variants

```rust
pub enum QueryResult {
    /// No rows returned
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
```

### Usage

```rust
let result = db.query("SELECT * FROM users")?;

if let QueryResult::Rows(rows) = result {
    println!("Found {} rows", rows.len());
    
    for row in rows {
        // Access values by column name
        if let Some(name) = row.get("name") {
            println!("Name: {}", name);
        }
    }
}
```

---

## Value

Represents a SQL value.

### Variants

```rust
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
```

### Conversion Methods

```rust
let value = Value::Text("hello".to_string());

// Check type
assert!(value.as_str().is_some());

// Get value
assert_eq!(value.as_str(), Some("hello"));

// Convert to string
assert_eq!(value.to_string(), "hello");
```

### Working with JSON

```rust
use serde_json;

// Parse JSON string to Value
let json_str = r#"{"name": "Alice", "age": 30}"#;
let value: Value = serde_json::from_str(json_str)?;

// Convert Value to JSON string
let json_string = serde_json::to_string(&value)?;
```

---

## Error Handling

Tywindb uses a custom error type `TywindbError`.

### Error Variants

```rust
pub enum TywindbError {
    Io(std::io::Error),
    Serialization(bincode::Error),
    Json(serde_json::Error),
    SqlParse(String),
    TableNotFound(String),
    ColumnNotFound(String),
    TypeMismatch { expected: String, actual: String },
    ConstraintViolation(String),
    Transaction(String),
    Corrupted(String),
    NotImplemented(String),
}
```

### Handling Errors

```rust
use tywindb::{Database, TywindbError};

match db.query("SELECT * FROM nonexistent") {
    Ok(result) => {
        println!("Query successful: {:?}", result);
    }
    Err(TywindbError::TableNotFound(name)) => {
        eprintln!("Table '{}' not found", name);
    }
    Err(TywindbError::SqlParse(msg)) => {
        eprintln!("SQL syntax error: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

---

## Examples

### Complete Example

```rust
use tywindb::{Database, QueryResult, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open database
    let mut db = Database::open("example.tdb")?;
    
    // Create table
    db.query("CREATE TABLE users (id INTEGER, name TEXT, email TEXT)")?;
    
    // Insert data
    db.query("INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com')")?;
    db.query("INSERT INTO users (id, name, email) VALUES (2, 'Bob', 'bob@example.com')")?;
    
    // Query data
    let result = db.query("SELECT * FROM users")?;
    
    if let QueryResult::Rows(rows) = result {
        println!("Found {} users:", rows.len());
        
        for row in rows {
            let id = row.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let email = row.get("email").and_then(|v| v.as_str()).unwrap_or("unknown");
            
            println!("  {}: {} ({})", id, name, email);
        }
    }
    
    // Update data
    db.query("UPDATE users SET email = 'alice.new@example.com' WHERE id = 1")?;
    
    // Delete data
    db.query("DELETE FROM users WHERE id = 2")?;
    
    // Transaction example
    db.begin()?;
    db.query("INSERT INTO users (id, name, email) VALUES (3, 'Charlie', 'charlie@example.com')")?;
    db.commit()?;
    
    // Close database
    db.close()?;
    
    Ok(())
}
```

### JSON Example

```rust
use tywindb::{Database, QueryResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::open("json_example.tdb")?;
    
    // Create table with JSON column
    db.query("CREATE TABLE users (id INTEGER, name TEXT, metadata TEXT)")?;
    
    // Insert with JSON data
    let metadata = r#"{"age": 30, "role": "admin", "skills": ["rust", "python"]}"#;
    db.query(&format!(
        "INSERT INTO users (id, name, metadata) VALUES (1, 'Alice', '{}')",
        metadata
    ))?;
    
    // Query and parse JSON
    let result = db.query("SELECT * FROM users")?;
    
    if let QueryResult::Rows(rows) = result {
        for row in rows {
            if let Some(metadata) = row.get("metadata").and_then(|v| v.as_str()) {
                let json: serde_json::Value = serde_json::from_str(metadata)?;
                println!("Name: {}", row.get("name").unwrap());
                println!("Age: {}", json["age"]);
                println!("Role: {}", json["role"]);
            }
        }
    }
    
    Ok(())
}
```

### Vector Search Example

```rust
use tywindb::vector::VectorIndex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create vector index
    let mut index = VectorIndex::new(3);
    
    // Insert vectors
    index.insert("doc1".to_string(), vec![1.0, 0.0, 0.0])?;
    index.insert("doc2".to_string(), vec![0.0, 1.0, 0.0])?;
    index.insert("doc3".to_string(), vec![0.0, 0.0, 1.0])?;
    
    // Search for similar vectors
    let results = index.search(&[1.0, 0.0, 0.0], 2)?;
    
    for (id, score) in results {
        println!("{}: {}", id, score);
    }
    
    Ok(())
}
```

---

## Building with Tywindb

### Cargo.toml

```toml
[dependencies]
tywindb = "0.1.0"
```

### Features

Tywindb includes the following features by default:
- SQL Parser
- ACID Transactions
- Persistent Storage
- MVCC
- Document Operations (JSON)
- Vector Search
- Full-Text Search

---

*Last updated: 2026-07-24*
