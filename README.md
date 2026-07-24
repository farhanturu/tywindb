<div align="center">

# 🗄️ Tywindb

**A Modern, Secure, and Feature-Rich Embedded Database**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/Version-0.97-green.svg)](https://github.com/paong-dev/tywindb/releases)

[Features](#features) • [Quick Start](#quick-start) • [Commands](#commands) • [Comparison](#comparison) • [Contributing](#contributing)

</div>

---

## ✨ What is Tywindb?

Tywindb is a modern embedded database built in Rust that combines the best features from SQLite, PostgreSQL, and MongoDB into a single, easy-to-use package. It's designed for developers who need:

- 🔒 **Security** — Password protection and AES-256 encryption built-in
- 🚀 **Performance** — Fast queries with MVCC support
- 📦 **Portability** — Single file database, no server required
- 🛠️ **Developer Experience** — Beautiful CLI with colored output

---

## 🎯 Features

| Feature | Description |
|---------|-------------|
| 🔐 **Password Protection** | Secure your database with Argon2 password hashing |
| 🔒 **AES-256 Encryption** | Military-grade encryption for data at rest |
| 📊 **SQL Support** | Full SQL syntax with SELECT, INSERT, UPDATE, DELETE |
| 🔄 **ACID Transactions** | BEGIN, COMMIT, ROLLBACK support |
| 📁 **Backup & Restore** | Compressed backups with easy restore |
| 📥 **CSV Import/Export** | Import and export data as CSV files |
| 🔄 **Migration System** | Version-controlled schema changes |
| 🎨 **Beautiful CLI** | Colored output with formatted tables |
| 💾 **Persistent Storage** | Data survives restarts |
| 🚫 **Zero Config** | No server, no setup, just run |

---

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/paong-dev/tywindb.git
cd tywindb

# Build
cargo build --release

# The binary will be at target/release/tywindb
```

### First Steps

```bash
# Start interactive REPL
./target/release/tywindb repl

# Create a table
tywindb> CREATE TABLE users (id INTEGER, name TEXT, email TEXT);

# Insert data
tywindb> INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

# Query data
tywindb> SELECT * FROM users;

# Set password
tywindb> passwd
New password: ****
Confirm password: ****
Password set.
```

---

## 📋 Commands

### Database Operations

| Command | Description | Example |
|---------|-------------|---------|
| `repl` | Start interactive REPL | `tywindb repl --db mydb.tdb` |
| `exec` | Execute SQL file | `tywindb exec --db mydb.tdb script.sql` |
| `query` | Run single query | `tywindb query --db mydb.tdb --sql "SELECT * FROM users"` |

### Security

| Command | Description | Example |
|---------|-------------|---------|
| `passwd` | Set database password | `tywindb passwd --db mydb.tdb` |

### Backup & Restore

| Command | Description | Example |
|---------|-------------|---------|
| `backup` | Backup database | `tywindb backup --db mydb.tdb --output backup.tdb` |
| `restore` | Restore from backup | `tywindb restore --db mydb.tdb --backup backup.tdb` |

### Data Import/Export

| Command | Description | Example |
|---------|-------------|---------|
| `export-csv` | Export table to CSV | `tywindb export-csv --db mydb.tdb --output users.csv users` |
| `import-csv` | Import CSV to table | `tywindb import-csv --db mydb.tdb --input users.csv users` |

### Migrations

| Command | Description | Example |
|---------|-------------|---------|
| `migrate-create` | Create new migration | `tywindb migrate-create --db mydb.tdb "add users"` |
| `migrate-up` | Run pending migrations | `tywindb migrate-up --db mydb.tdb` |
| `migrate-down` | Rollback last migration | `tywindb migrate-down --db mydb.tdb` |
| `migrate-status` | Show migration status | `tywindb migrate-status --db mydb.tdb` |
| `migrate-rollback` | Rollback to version | `tywindb migrate-rollback --db mydb.tdb 1` |
| `migrate-dry-run` | Preview SQL | `tywindb migrate-dry-run --db mydb.tdb 1` |

---

## 💻 SQL Examples

### Create Table

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Insert Data

```sql
INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');
INSERT INTO users (name, email) VALUES ('Bob', 'bob@example.com');
```

### Query Data

```sql
SELECT * FROM users;
SELECT * FROM users WHERE id = 1;
SELECT * FROM users ORDER BY name;
SELECT * FROM users LIMIT 10;
```

### Update Data

```sql
UPDATE users SET email = 'alice@new.com' WHERE id = 1;
```

### Delete Data

```sql
DELETE FROM users WHERE id = 1;
```

### Transactions

```sql
BEGIN;
INSERT INTO users (name, email) VALUES ('Charlie', 'charlie@example.com');
UPDATE accounts SET balance = balance - 100 WHERE user_id = 1;
COMMIT;
```

---

## 📊 Comparison

### Tywindb vs Other Databases

| Feature | Tywindb | SQLite | PostgreSQL | MongoDB |
|---------|---------|--------|------------|---------|
| **Type** | Embedded | Embedded | Server | Server |
| **Language** | Rust | C | C | C++ |
| **License** | MIT | Public Domain | PostgreSQL | SSPL |
| **Password Auth** | ✅ | ❌ | ✅ | ✅ |
| **Encryption** | ✅ AES-256 | ❌ | ✅ | ✅ |
| **JSON Support** | ✅ | ⚠️ | ✅ | ✅ |
| **SQL Support** | ✅ | ✅ | ✅ | ❌ |
| **MVCC** | ✅ | ❌ | ✅ | ❌ |
| **Backup** | ✅ | ⚠️ | ✅ | ✅ |
| **Migration** | ✅ | ❌ | ⚠️ | ❌ |
| **Setup Required** | None | None | High | Medium |

### When to Use Tywindb

✅ **Use Tywindb when:**
- Building desktop or mobile apps
- Need password protection
- Want embedded database
- Need SQL + JSON support
- Prototyping or small projects

❌ **Use something else when:**
- Need massive scale (use PostgreSQL)
- Need replication (use MySQL)
- Need real-time caching (use Redis)

---

## 🔧 Configuration

### Environment Variables

```bash
# Database path
TYWINDB_DB=mydb.tdb

# Log level
TYWINDB_LOG=info
```

### CLI Options

```bash
# Specify database path
tywindb repl --db /path/to/database.tdb

# Show version
tywindb --version

# Show help
tywindb --help
```

---

## 📁 Project Structure

```
tywindb/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── db/              # Database core
│   ├── engine/          # Query engine
│   ├── sql/             # SQL parser
│   ├── storage/         # Storage engine
│   ├── crypto/          # Encryption & hashing
│   ├── backup.rs        # Backup & restore
│   ├── migration.rs     # Migration system
│   └── export.rs        # CSV import/export
├── Cargo.toml           # Dependencies
├── LICENSE              # MIT License
└── README.md            # This file
```

---

## 🤝 Contributing

Contributions are welcome! Here's how:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Inspired by [SQLite](https://www.sqlite.org/), [PostgreSQL](https://www.postgresql.org/), and [MongoDB](https://www.mongodb.com/)
- Built with [Rust](https://www.rust-lang.org/) 🦀
- Password hashing with [Argon2](https://github.com/P-H-C/phc-winner-argon2)
- Encryption with [AES-256-GCM](https://github.com/RustCrypto/AEADs)

---

<div align="center">

**Made with ❤️ by Paong**

[⬆ Back to Top](#-tywindb)

</div>
