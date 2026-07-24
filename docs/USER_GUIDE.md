# Tywindb User Guide

Welcome to Tywindb! This guide will help you get started with the database.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Reference](#cli-reference)
- [GUI Interface](#gui-interface)
- [SQL Reference](#sql-reference)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/tywindb/tywindb.git
cd tywindb

# Build
cargo build --release

# Install (optional)
cargo install --path .
```

### Verify Installation

```bash
tywindb --version
```

---

## Quick Start

### 1. Create Your First Database

```bash
# Start interactive REPL
tywindb repl --db mydb.tdb
```

### 2. Create a Table

```sql
CREATE TABLE users (
    id INTEGER,
    name TEXT,
    email TEXT
);
```

### 3. Insert Data

```sql
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users (id, name, email) VALUES (2, 'Bob', 'bob@example.com');
```

### 4. Query Data

```sql
SELECT * FROM users;
SELECT * FROM users WHERE id = 1;
```

### 5. Exit

```sql
exit
```

---

## CLI Reference

### Commands

| Command | Description | Example |
|---------|-------------|---------|
| `repl` | Start interactive REPL | `tywindb repl --db mydb.tdb` |
| `exec` | Execute SQL file | `tywindb exec --db mydb.tdb script.sql` |
| `query` | Run single query | `tywindb query --db mydb.tdb --sql "SELECT * FROM users"` |
| `gui` | Start web GUI | `tywindb gui --port 8080` |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-d, --db` | Database file path | `tywindb.tdb` |
| `-p, --port` | Server port (GUI mode) | `8080` |
| `-s, --sql` | SQL query (query mode) | — |

### Examples

```bash
# Create database and run SQL file
tywindb exec --db mydb.tdb setup.sql

# Run query and get results
tywindb query --db mydb.tdb --sql "SELECT COUNT(*) FROM users"

# Start GUI on custom port
tywindb gui --db mydb.tdb --port 3000
```

---

## GUI Interface

### Starting the GUI

```bash
tywindb gui --db mydb.tdb
```

Then open http://127.0.0.1:8080 in your browser.

### Features

- **Query Editor** — Write and execute SQL queries
- **Results Table** — View query results in a table
- **Table Browser** — Browse database tables
- **Create Table** — Create new tables with a form
- **Export** — Export results as CSV
- **Keyboard Shortcuts** — Fast query execution

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + Enter` | Run query |
| `Ctrl + L` | Clear editor |
| `Ctrl + S` | Save query |

---

## SQL Reference

### Data Types

| Type | Description | Example |
|------|-------------|---------|
| `INTEGER` | Whole numbers | `42` |
| `TEXT` | Text strings | `'Hello'` |
| `FLOAT` | Decimal numbers | `3.14` |
| `BOOLEAN` | True/false | `true` |
| `BLOB` | Binary data | — |

### Commands

#### CREATE TABLE

```sql
CREATE TABLE table_name (
    column1 TYPE,
    column2 TYPE,
    ...
);
```

#### INSERT

```sql
INSERT INTO table_name (col1, col2) VALUES (val1, val2);
```

#### SELECT

```sql
SELECT * FROM table_name;
SELECT col1, col2 FROM table_name WHERE condition;
SELECT * FROM table_name ORDER BY col1;
SELECT * FROM table_name LIMIT 10;
```

#### UPDATE

```sql
UPDATE table_name SET col1 = val1 WHERE condition;
```

#### DELETE

```sql
DELETE FROM table_name WHERE condition;
```

#### Transactions

```sql
BEGIN;
-- Your operations here
COMMIT;

-- Or rollback
BEGIN;
-- Your operations here
ROLLBACK;
```

### Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Equal | `WHERE id = 1` |
| `!=` or `<>` | Not equal | `WHERE id != 1` |
| `>` | Greater than | `WHERE age > 25` |
| `<` | Less than | `WHERE age < 25` |
| `>=` | Greater or equal | `WHERE age >= 25` |
| `<=` | Less or equal | `WHERE age <= 25` |
| `AND` | Logical AND | `WHERE age > 25 AND active = true` |
| `OR` | Logical OR | `WHERE age < 25 OR age > 60` |
| `LIKE` | Pattern match | `WHERE name LIKE 'A%'` |
| `IN` | In list | `WHERE id IN (1, 2, 3)` |

---

## Examples

### Example 1: Blog Database

```sql
-- Create tables
CREATE TABLE posts (
    id INTEGER,
    title TEXT,
    content TEXT,
    author_id INTEGER,
    created_at TEXT
);

CREATE TABLE comments (
    id INTEGER,
    post_id INTEGER,
    author TEXT,
    content TEXT
);

-- Insert posts
INSERT INTO posts (id, title, content, author_id, created_at)
VALUES (1, 'Hello World', 'My first post', 1, '2026-01-01');

INSERT INTO posts (id, title, content, author_id, created_at)
VALUES (2, 'Tywindb Guide', 'How to use Tywindb', 1, '2026-01-02');

-- Insert comments
INSERT INTO comments (id, post_id, author, content)
VALUES (1, 1, 'Bob', 'Great post!');

INSERT INTO comments (id, post_id, author, content)
VALUES (2, 1, 'Charlie', 'Thanks for sharing!');

-- Query with conditions
SELECT * FROM posts WHERE author_id = 1;
SELECT * FROM comments WHERE post_id = 1;

-- Update
UPDATE posts SET title = 'Hello World!' WHERE id = 1;

-- Delete
DELETE FROM comments WHERE id = 2;
```

### Example 2: E-commerce Database

```sql
-- Create tables
CREATE TABLE products (
    id INTEGER,
    name TEXT,
    price FLOAT,
    stock INTEGER
);

CREATE TABLE orders (
    id INTEGER,
    product_id INTEGER,
    quantity INTEGER,
    total FLOAT
);

-- Insert products
INSERT INTO products (id, name, price, stock)
VALUES (1, 'Laptop', 999.99, 10);

INSERT INTO products (id, name, price, stock)
VALUES (2, 'Mouse', 29.99, 100);

-- Insert order
INSERT INTO orders (id, product_id, quantity, total)
VALUES (1, 1, 1, 999.99);

-- Query product with orders
SELECT p.name, o.quantity, o.total
FROM products p
JOIN orders o ON p.id = o.product_id;

-- Update stock
UPDATE products SET stock = stock - 1 WHERE id = 1;
```

### Example 3: JSON Data

```sql
-- Create table with JSON column
CREATE TABLE users (
    id INTEGER,
    name TEXT,
    metadata TEXT
);

-- Insert with JSON data
INSERT INTO users (id, name, metadata)
VALUES (1, 'Alice', '{"age": 30, "role": "admin", "skills": ["rust", "python"]}');

INSERT INTO users (id, name, metadata)
VALUES (2, 'Bob', '{"age": 25, "role": "user", "skills": ["go", "javascript"]}');

-- Query all users
SELECT * FROM users;

-- Query with condition
SELECT * FROM users WHERE id = 1;
```

---

## Troubleshooting

### Common Issues

#### "Database is locked"

This happens when another process is using the database. Close other Tywindb instances and try again.

#### "Table not found"

Make sure you've created the table before querying it:

```sql
CREATE TABLE my_table (id INTEGER, name TEXT);
```

#### "Syntax error"

Check your SQL syntax. Common mistakes:
- Missing semicolon at end of statement
- Missing quotes around strings
- Misspelled keywords

### Getting Help

- **CLI Help**: `tywindb --help`
- **GitHub Issues**: https://github.com/tywindb/tywindb/issues
- **Documentation**: https://github.com/tywindb/tywindb/tree/main/docs

---

*Last updated: 2026-07-24*
