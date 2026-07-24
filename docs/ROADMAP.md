# Tywindb Roadmap

This document outlines the future direction of Tywindb and features we plan to implement.

---

## Vision

Tywindb aims to be the **go-to database for modern applications** — combining simplicity, performance, and flexibility in one package. We want to make database operations as easy as calling a function.

---

## Short-Term Goals (Next 3 months)

### 1. SDK Releases

#### Go SDK
- Native Go bindings
- Connection pooling
- Prepared statements
- Transaction support

```go
import "github.com/tywindb/tywindb-go"

db, _ := tywindb.Open("mydb.tdb")
defer db.Close()

var users []User
db.Query("SELECT * FROM users WHERE age > $1", &users, 25)
```

#### Python SDK
- Async/await support
- Type hints
- Context manager support
- Pandas integration

```python
from tywindb import Tywindb

async with Tywindb.open_async("mydb.tdb") as db:
    users = await db.query("SELECT * FROM users WHERE age > $1", 25)
```

#### TypeScript SDK
- Full TypeScript support
- Promise-based API
- Type-safe queries
- React hooks integration

```typescript
import { Tywindb } from '@tywindb/sdk';

const db = await Tywindb.open('mydb.tdb');
const users = await db.query<User>('SELECT * FROM users WHERE age > $1', [25]);
```

### 2. Client-Server Mode

- TCP/UDP server
- Connection pooling
- Authentication (optional)
- SSL/TLS support

```bash
# Start server
tywindb serve --port 8080 --auth-token mysecret

# Connect from client
tywindb connect --host localhost --port 8080
```

### 3. REST API

- RESTful endpoints
- OpenAPI/Swagger documentation
- Rate limiting
- CORS support

```bash
# Query via REST
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * FROM users"}'
```

---

## Medium-Term Goals (3-6 months)

### 4. Replication & Clustering

- Primary-replica replication
- Multi-node clustering
- Automatic failover
- Load balancing

```bash
# Start cluster
tywindb cluster --nodes node1:8080,node2:8080,node3:8080

# Replication
tywindb serve --replicate-from primary:8080
```

### 5. Extension System

- Custom functions
- Custom index types
- Custom storage backends
- Plugin architecture

```rust
// Define custom extension
#[tywindb::extension]
pub mod my_extension {
    #[function]
    pub fn my_func(input: &str) -> String {
        format!("Processed: {}", input)
    }
}
```

### 6. Advanced Query Features

- Subqueries
- CTEs (Common Table Expressions)
- Window functions
- LATERAL joins

```sql
-- Subquery
SELECT * FROM users WHERE id IN (SELECT user_id FROM orders);

-- CTE
WITH active_users AS (
    SELECT * FROM users WHERE active = true
)
SELECT * FROM active_users WHERE age > 25;

-- Window function
SELECT name, salary, RANK() OVER (ORDER BY salary DESC) as rank
FROM employees;
```

---

## Long-Term Goals (6-12 months)

### 7. Cloud Integration

- Managed cloud service
- Auto-scaling
- Global distribution
- Pay-per-use pricing

### 8. Advanced Analytics

- Materialized views
- Query caching
- Query optimization
- Cost-based optimizer

### 9. Data Import/Export

- CSV import/export
- Parquet support
- JSON Lines (NDJSON)
- Database migration tools

```bash
# Import CSV
tywindb import --table users --file users.csv

# Export to Parquet
tywindb export --table users --format parquet --file users.parquet
```

### 10. Monitoring & Observability

- Metrics export (Prometheus)
- Query logging
- Performance profiling
- Real-time dashboards

---

## Experimental Features

These features are under consideration and may or may not be implemented:

### Graph Queries (GQL)

```sql
-- Native graph traversal
MATCH (user:User)-[:FRIENDS_WITH]->(friend:User)
WHERE user.age > 25
RETURN user.name, friend.name;
```

### Time-Series Optimizations

```sql
-- Time-series queries
SELECT time_bucket('1 hour', timestamp) as hour, AVG(temperature)
FROM sensor_data
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY hour;
```

### Machine Learning Integration

```sql
-- Built-in ML functions
SELECT predict('churn_model', * FROM customers) as churn_probability
FROM customers;
```

---

## Contributing

We welcome contributions! See our [Contributing Guide](../CONTRIBUTING.md) for details.

### Priority Areas

1. **SDK Development** — Go, Python, TypeScript bindings
2. **Testing** — Unit tests, integration tests, benchmarks
3. **Documentation** — API docs, tutorials, examples
4. **Performance** — Query optimization, storage improvements

---

## Community Input

We value community feedback! Please:

- Vote on features in [GitHub Discussions](https://github.com/tywindb/tywindb/discussions)
- Report issues in [GitHub Issues](https://github.com/tywindb/tywindb/issues)
- Join our Discord (coming soon)

---

*Last updated: 2026-07-24*
