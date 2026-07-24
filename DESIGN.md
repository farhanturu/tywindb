# Tywindb — Design Specification

> Database modern yang mengambil yang terbaik dari setiap database, dipadukan jadi satu yang lebih mudah, cepat, dan plug-and-play.

---

## 1. Vision & Philosophy

**Tagline:** "The database that just works."

Tywindb didesain untuk menjadi database yang:
- **Plug-and-play** — install, langsung jalan, zero config
- **Multi-model** — document, relational, graph, vector dalam 1 database
- **Blazing fast** — performa production-ready dari hari pertama
- **Developer first** — API yang intuitive, bahasa modern (Rust core, SDK Go/Python/JS/TS)
- **Portable** — embedded mode (single file) atau client-server, cloud atau self-hosted

### Core Principles

1. **Simplicity over complexity** — API yang mudah dipahami, bukan SQL yang rumit
2. **Convention over configuration** — zero-config defaults yang benar
3. **Best of breed** — ambil fitur terbaik dari setiap database, poles jadi lebih baik
4. **Performance is not optional** — fast by default, bukan afterthought

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Tywindb Core                           │
├─────────────────────────────────────────────────────────────┤
│  Query Engine  │  Storage Engine  │  Index Engine           │
│  ─────────────│──────────────────│─────────────────        │
│  • Parser      │  • LSM Tree      │  • B-Tree               │
│  • Optimizer   │  • WAL           │  • HNSW (vector)        │
│  • Executor    │  • Compression   │  • Full-text (BM25)     │
│  • Cache       │  • MVCC          │  • Geospatial           │
├─────────────────────────────────────────────────────────────┤
│                     API Layer                               │
│  ─────────────────────────────────────────────────          │
│  • REST API  • gRPC  • Embedded API  • CLI                 │
├─────────────────────────────────────────────────────────────┤
│                   SDK Layer                                 │
│  ─────────────────────────────────────────────────          │
│  • Rust (native)  • Go  • Python  • TypeScript/JavaScript  │
│  • Java  • C/C++  • Dart  • Swift                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Features — Best of Each Database

### 3.1 From SQLite/libSQL
**Diambil:** Simplicity, portability, zero-config, single-file database
**Dipoles:** Concurrent writes via MVCC (inspired by Turso Database)

```rust
// Tywindb — SQLite simplicity + modern features
let db = Tywindb::open("my_data.tdb")?;  // Single file, zero config

// Concurrent writes — no more "database is locked"
db.exec("INSERT INTO users (name, email) VALUES (?, ?)", params!["John", "john@example.com"])?;
```

**Fitur yang diintegrasikan:**
- Single-file database format (portable, backup mudah)
- Embedded mode (in-process, no server needed)
- ACID transactions
- WAL (Write-Ahead Logging) untuk concurrent reads
- **NEW:** MVCC untuk concurrent writes (inspired by Turso)
- **NEW:** Embedded replicas untuk high availability

### 3.2 From DuckDB
**Diambil:** Columnar-vectorized execution untuk OLAP
**Dipoles:** Hybrid engine yang otomatis switch antara row-store dan column-store

```rust
// OLAP queries — automatic columnar optimization
let analytics = db.query("
    SELECT category, SUM(amount) as total 
    FROM sales 
    GROUP BY category 
    ORDER BY total DESC
")?;

// Result otomatis di-cache dalam columnar format untuk queries serupa
```

**Fitur yang diintegrasikan:**
- Vectorized execution untuk analytics
- Automatic query optimization
- Parquet/JSON native support
- **NEW:** Hybrid row/column storage — otomatis pilih yang terbaik

### 3.3 From SurrealDB
**Diambil:** Multi-model (document + graph + vector), SurrealQL
**Dipoles:** Query language yang lebih familiar (SQL-like + GraphQL-like)

```rust
// Multi-model dalam 1 database
db.exec("
    CREATE user:john SET 
        name = 'John Doe',
        email = 'john@example.com',
        friends = ->knows->['jane', 'bob'],
        embedding = vector::embed('text-embedding-3-small', 'Hello world')
")?;

// Graph traversal — mudah seperti SQL
let friends = db.query("
    SELECT ->knows->user.* AS friends 
    FROM user:john 
    WHERE ->knows->user.age > 25
")?;

// Vector search — similarity search built-in
let similar = db.query("
    SELECT *, vector::similarity(embedding, $query_embedding) AS score 
    FROM documents 
    WHERE score > 0.8 
    ORDER BY score DESC 
    LIMIT 10
", params!["query_embedding": embedding])?;
```

**Fitur yang diintegrasikan:**
- Document store (JSON-like documents)
- Graph relationships (nodes + edges)
- Vector embeddings (HNSW index)
- Live queries (real-time updates)
- **NEW:** SQL-like syntax yang lebih familiar dari SurrealQL

### 3.4 From Redis
**Diambil:** In-memory performance, data structures
**Dipoles:** Optional in-memory mode untuk caching

```rust
// In-memory mode — untuk caching/sessions
let cache = Tywindb::in_memory()?;

// Typed data structures
cache.set("session:123", &Session { user_id: 42, expires: 3600 })?;
cache.sorted_set("leaderboard", "player:1", 1000.0)?;

// TTL support
cache.set_with_ttl("temp:data", &data, Duration::from_secs(300))?;
```

### 3.5 From PostgreSQL
**Diambil:** ACID, extensions, JSONB support
**Dipoles:** Extension system yang lebih mudah

```rust
// Extensions — load seperti plugin
db.load_extension("tywindb-json")?;
db.load_extension("tywindb-geo")?;
db.load_extension("tywindb-auth")?;

// JSONB-like operations
db.exec("SELECT data->>'name' FROM users WHERE data @> '{\"active\": true}'")?;
```

### 3.6 From MongoDB
**Diambil:** Flexible schema, aggregation pipeline
**Dipoles:** Typed schema yang optional

```rust
// Schema-less — untuk development cepat
db.collection("users").insert(doc! {
    "name": "John",
    "metadata": { "preferences": { "theme": "dark" } }
})?;

// ATAU schema-aware — untuk production
db.schema().collection("users")
    .field("name", Type::String)
    .field("email", Type::String, unique: true)
    .field("created_at", Type::DateTime, default: "now()")
    .build()?;
```

---

## 4. Query Language — TywindQL

TywindQL adalah query language yang mengambil yang terbaik dari SQL, SurrealQL, dan GraphQL:

### 4.1 Basic CRUD

```sql
-- Create
CREATE user SET name = 'John', email = 'john@example.com';

-- Read
SELECT * FROM user WHERE name = 'John';
SELECT name, email FROM user WHERE age > 25 ORDER BY name;

-- Update
UPDATE user:john SET email = 'newemail@example.com';

-- Delete
DELETE user WHERE active = false;
```

### 4.2 Document Operations

```sql
-- Nested document access
SELECT data->>'address'->>'city' FROM users;

-- Array operations
SELECT * FROM users WHERE tags CONTAINS 'admin';

-- JSON patch
UPDATE user:john MERGE { preferences: { theme: 'dark', lang: 'en' } };
```

### 4.3 Graph Queries

```sql
-- Create relationships
RELATE user:john->knows->user:jane SET since = '2020-01-01';

-- Traverse graph
SELECT ->knows->user.* AS friends FROM user:john;

-- Multi-hop traversal
SELECT ->knows->->knows->user.* AS friends_of_friends FROM user:john;

-- Shortest path
SELECT graph::shortest_path(user:john, user:alice, 'knows');
```

### 4.4 Vector Search

```sql
-- Create vector index
DEFINE INDEX doc_embedding ON documents FIELDS embedding HNSW DIMENSION 1536;

-- Insert with embedding
CREATE document SET 
    title = 'Introduction to Databases',
    content = '...',
    embedding = vector::embed('text-embedding-3-small', content);

-- Similarity search
SELECT *, vector::similarity(embedding, $query) AS score 
FROM documents 
WHERE score > 0.8 
ORDER BY score DESC 
LIMIT 10;
```

### 4.5 Full-Text Search

```sql
-- Full-text index
DEFINE INDEX doc_fts ON documents FIELDS content SEARCH ANALYZER bm25;

-- Search
SELECT * FROM documents WHERE content @1@ 'database design';

-- Highlight matches
SELECT *, search::highlight(content, 'database') AS highlighted 
FROM documents WHERE content @1@ 'database';
```

### 4.6 Aggregations

```sql
-- Group by
SELECT category, COUNT() AS count, AVG(price) AS avg_price 
FROM products 
GROUP BY category;

-- Window functions
SELECT name, salary, RANK() OVER (ORDER BY salary DESC) AS rank 
FROM employees;

-- Time-series
SELECT DATE(created_at) AS day, COUNT() AS count 
FROM events 
WHERE created_at > time::now() - 7d 
GROUP BY day;
```

---

## 5. Storage Engine

### 5.1 Hybrid Storage Model

```
┌─────────────────────────────────────────┐
│           Tywindb Storage               │
├─────────────────────────────────────────┤
│  Hot Data (Recent/Frequent)            │
│  ─────────────────────────────         │
│  • Row-store (B+Tree)                  │
│  • Optimized for OLTP                  │
│  • In-memory cache                     │
├─────────────────────────────────────────┤
│  Cold Data (Analytics/Historical)      │
│  ─────────────────────────────         │
│  • Column-store (compressed)           │
│  • Optimized for OLAP                  │
│  • Auto-compaction                     │
├─────────────────────────────────────────┤
│  Vector Data                           │
│  ─────────────────────────────         │
│  • HNSW index                          │
│  • IVF-Flat (fallback)                 │
├─────────────────────────────────────────┤
│  Graph Data                            │
│  ─────────────────────────────         │
│  • Adjacency list                      │
│  • Edge index                          │
└─────────────────────────────────────────┘
```

### 5.2 File Format

```
my_data.tdb
├── Header (magic bytes, version, metadata)
├── Schema (table definitions, indexes)
├── Data Segments
│   ├── Row segments (hot data)
│   ├── Column segments (cold data)
│   ├── Vector segments (embeddings)
│   └── Graph segments (relationships)
├── Indexes
│   ├── B-Tree indexes
│   ├── HNSW vector indexes
│   └── Full-text indexes (inverted)
├── WAL (Write-Ahead Log)
└── Checkpoint
```

### 5.3 Concurrency Model

- **MVCC (Multi-Version Concurrency Control)** — readers don't block writers
- **WAL** — crash recovery, point-in-time recovery
- **Lock-free reads** — snapshot isolation
- **Write batching** — multiple writes per transaction

---

## 6. API Design

### 6.1 Embedded API (Rust)

```rust
use tywindb::Tywindb;

// Open database (creates if not exists)
let db = Tywindb::open("my_data.tdb")?;

// Simple query
let users: Vec<User> = db.query("SELECT * FROM user WHERE active = true")?;

// Parameterized query
let user: Option<User> = db.query_one(
    "SELECT * FROM user WHERE id = $1",
    params![user_id]
)?;

// Transaction
db.transaction(|tx| {
    tx.exec("INSERT INTO users (name) VALUES ($1)", params!["John"])?;
    tx.exec("UPDATE accounts SET balance = balance - $1 WHERE id = $2", params![100, account_id])?;
    Ok(())
})?;

// Real-time subscription
let mut sub = db.subscribe("SELECT * FROM messages WHERE room = $1", params!["general"])?;
while let Some(event) = sub.next().await {
    println!("New message: {:?}", event);
}
```

### 6.2 Go SDK

```go
import "github.com/tywindb/tywindb-go"

db, _ := tywindb.Open("my_data.tdb")
defer db.Close()

// Query
var users []User
db.Query("SELECT * FROM user WHERE active = true", &users)

// Transaction
db.Transaction(func(tx *tywindb.Tx) error {
    tx.Exec("INSERT INTO users (name) VALUES ($1)", "John")
    return nil
})
```

### 6.3 Python SDK

```python
from tywindb import Tywindb

db = Tywindb.open("my_data.tdb")

# Query
users = db.query("SELECT * FROM user WHERE active = true")

# Async support
async def main():
    async with Tywindb.open_async("my_data.tdb") as db:
        users = await db.query("SELECT * FROM user")
```

### 6.4 TypeScript/JavaScript SDK

```typescript
import { Tywindb } from '@tywindb/sdk';

const db = await Tywindb.open('my_data.tdb');

// Query with types
const users = await db.query<User>('SELECT * FROM user WHERE active = true');

// Reactive queries
db.subscribe('SELECT * FROM messages', (messages) => {
    console.log('New messages:', messages);
});
```

### 6.5 REST API

```bash
# Start server
tywindb serve --port 8080

# Query
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * FROM users WHERE active = true"}'

# Transaction
curl -X POST http://localhost:8080/transaction \
  -H "Content-Type: application/json" \
  -d '{
    "statements": [
      "INSERT INTO users (name) VALUES ($1)",
      "UPDATE accounts SET balance = balance - $2"
    ],
    "params": ["John", 100]
  }'
```

---

## 7. Deployment Modes

### 7.1 Embedded (Default)

```bash
# Install
cargo install tywindb

# Use in code
let db = Tywindb::open("data.tdb")?;
```

- Zero config
- Single binary
- Single-file database
- Best for: desktop apps, mobile, edge, CLI tools

### 7.2 Client-Server

```bash
# Start server
tywindb serve --port 8080 --data /var/lib/tywindb

# Connect from client
let db = Tywindb::connect("localhost:8080")?;
```

- Multi-client access
- Network transparency
- Best for: web apps, microservices

### 7.3 Clustered

```bash
# Start cluster
tywindb cluster --nodes node1:8080,node2:8080,node3:8080

# Replication
tywindb serve --replicate-from primary:8080
```

- Horizontal scaling
- High availability
- Best for: production systems, global apps

### 7.4 Cloud (Managed)

```bash
# Deploy to Tywindb Cloud
tywindb cloud deploy --name my-app --region us-east-1

# Or use existing cloud
tywindb cloud connect --url cloud.tywindb.com/app123
```

- Zero ops
- Auto-scaling
- Backups included
- Best for: SaaS, startups

---

## 8. Extensions System

```rust
// Extensions directory
~/.tywindb/extensions/
├── tywindb-json/        # Advanced JSON operations
├── tywindb-geo/         # Geospatial queries
├://tywindb-auth/         # Built-in authentication
├── tywindb-search/      # Advanced full-text search
├── tywindb-vector/      # Advanced vector operations
└── tywindb-time/        # Time-series optimizations
```

### Extension API

```rust
// Define extension
#[tywindb::extension]
pub mod my_extension {
    #[function]
    pub fn my_func(input: &str) -> String {
        format!("Processed: {}", input)
    }
    
    #[index]
    pub fn my_index(field: &Value) -> Vec<u8> {
        // Custom index logic
    }
}
```

---

## 9. Performance Targets

| Metric | Target | Inspired By |
|--------|--------|-------------|
| **OLTP Throughput** | >50K TPS single node | CockroachDB, MySQL |
| **OLAP Throughput** | >1B rows/sec | ClickHouse, DuckDB |
| **Read Latency** | <1ms p99 | Redis, SQLite |
| **Write Latency** | <5ms p99 | RocksDB |
| **Startup Time** | <10ms | SQLite, DuckDB |
| **Memory Footprint** | <10MB idle | SQLite |
| **Concurrent Connections** | >10K | PostgreSQL |
| **Vector Search** | <10ms for 1M vectors | Pinecone, Weaviate |

---

## 10. Comparison Matrix

| Feature | Tywindb | SQLite | DuckDB | SurrealDB | Redis | MongoDB |
|---------|---------|--------|--------|-----------|-------|---------|
| **Embedded Mode** | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Client-Server** | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ |
| **ACID** | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ |
| **Document Store** | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ |
| **Graph** | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| **Vector Search** | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ |
| **Full-Text Search** | ✅ | ⚠️ | ⚠️ | ✅ | ✅ | ✅ |
| **SQL Support** | ✅ | ✅ | ✅ | ⚠️ | ❌ | ❌ |
| **Concurrent Writes** | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Single-File DB** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **Zero Config** | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | ❌ |
| **Go SDK** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Rust SDK** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **TypeScript SDK** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Python SDK** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **License** | MIT | Public Domain | MIT | BSL | SSPL | SSPL |

---

## 11. Roadmap

### Phase 1: Foundation (MVP)
- [ ] Core storage engine (row-store + WAL)
- [ ] Basic SQL parser and executor
- [ ] ACID transactions
- [ ] Embedded mode
- [ ] Rust SDK
- [ ] CLI tool

### Phase 2: Modern Features
- [ ] MVCC for concurrent writes
- [ ] Document operations (JSON)
- [ ] Basic vector search (HNSW)
- [ ] Full-text search (BM25)
- [ ] Go and Python SDKs

### Phase 3: Advanced Features
- [ ] Graph relationships and traversal
- [ ] Hybrid row/column storage
- [ ] Client-server mode
- [ ] REST and gRPC APIs
- [ ] TypeScript/JavaScript SDK

### Phase 4: Production Ready
- [ ] Clustered mode (replication)
- [ ] Extension system
- [ ] Cloud managed service
- [ ] Advanced monitoring and metrics
- [ ] Enterprise features (auth, RBAC)

---

## 12. Tech Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Core** | Rust | Memory safety, performance, no GC |
| **Query Parser** | pest (PEG parser) | Fast, maintainable |
| **Storage Engine** | Custom LSM + B+Tree | Best of both worlds |
| **Vector Index** | HNSW (hnsw-rs) | State-of-the-art ANN |
| **Full-Text** | Tantivy (BM25) | Fast, Rust-native |
| **CLI** | clap | Ergonomic Rust CLI |
| **REST API** | axum | Fast, type-safe |
| **gRPC** | tonic | High performance |
| **Serialization** | bincode + serde | Fast binary format |

---

## 13. Decision Trace

```json
[
  {
    "decision": "Use Rust as core language",
    "reason": "Memory safety without GC, excellent performance, strong ecosystem for systems programming",
    "alternatives": ["Go (simpler but slower)", "C++ (faster but unsafe)", "Zig (emerging but small ecosystem)"],
    "tradeoff": "Steeper learning curve than Go, smaller community than C++"
  },
  {
    "decision": "Hybrid row/column storage",
    "reason": "Best of both OLTP (row-store) and OLAP (column-store), automatic optimization",
    "alternatives": ["Pure row-store (simpler)", "Pure column-store (analytics only)", "Separate engines (complex)"],
    "tradeoff": "More complex storage engine, larger codebase"
  },
  {
    "decision": "SQL-like query language instead of custom DSL",
    "reason": "Familiar to developers, easier adoption, can extend with graph/vector syntax",
    "alternatives": ["Custom DSL (more expressive)", "GraphQL (different paradigm)", "MongoDB aggregation (complex)"],
    "tradeoff": "Less expressive than custom DSL for graph queries"
  },
  {
    "decision": "Single-file database format",
    "reason": "Portability, simplicity, easy backup/transfer, inspired by SQLite success",
    "alternatives": ["Directory-based (PostgreSQL style)", "Multiple files (better for large datasets)"],
    "tradeoff": "File size limits, harder to parallelize I/O"
  },
  {
    "decision": "Multi-model from day one",
    "reason": "Modern apps need document + graph + vector, avoiding database sprawl",
    "alternatives": ["Single-model (simpler)", "Plugin-based (more flexible but complex)"],
    "tradeoff": "Larger initial codebase, more testing surface"
  },
  {
    "decision": "MIT License",
    "reason": "Maximum adoption, commercial friendly, no license friction",
    "alternatives": ["Apache 2.0 (patent protection)", "BSL (commercial protection)", "AGPL (copyleft)"],
    "tradeoff": "No patent protection clause"
  }
]
```

---

## 14. Anti-Patterns to Avoid

1. **Feature creep** — don't add every database feature; focus on core use cases
2. ** premature optimization** — get it working first, then optimize
3. **Complexity bias** — simple solutions are better than clever ones
4. **Breaking changes** — version carefully, deprecation warnings before removal
5. **Documentation debt** — docs are first-class, not afterthought

---

## 15. Success Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| **Time to first query** | <30 seconds | User testing |
| **Developer satisfaction** | >4.5/5 | Surveys |
| **Production adoption** | >1000 apps in Year 1 | Analytics |
| **GitHub stars** | >5000 in Year 1 | GitHub |
| **Performance** | Top 3 in embedded DB benchmarks | TPC-like benchmarks |

---

## Appendix A: Naming Conventions

- **Database file:** `*.tdb` (tywindb)
- **Binary:** `tywindb`
- **CLI commands:** `tywindb <command>` (e.g., `tywindb serve`, `tywindb query`)
- **SDK packages:** `tywindb` (Rust), `tywindb-go` (Go), `tywindb` (Python), `@tywindb/sdk` (JS)
- **Config file:** `tywindb.toml` or `tywindb.yaml`

---

## Appendix B: Environment Variables

```bash
TYWINDB_DATA=/var/lib/tywindb      # Data directory
TYWINDB_PORT=8080                    # Server port
TYWINDB_LOG=info                     # Log level
TYWINDB_CACHE_SIZE=256MB             # Cache size
TYWINDB_MAX_CONNECTIONS=1000         # Max connections
```

---

*Last updated: 2026-07-22*
*Version: 0.1.0 (Design Phase)*
