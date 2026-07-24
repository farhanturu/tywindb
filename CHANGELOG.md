# Changelog

All notable changes to Tywindb will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure
- SQL parser with support for SELECT, INSERT, UPDATE, DELETE, CREATE TABLE
- ACID transaction support (BEGIN, COMMIT, ROLLBACK)
- Persistent storage engine
- MVCC (Multi-Version Concurrency Control) for concurrent writes
- Document operations (JSON support)
- Vector search with HNSW index
- Full-text search with BM25 ranking
- CLI tool with interactive REPL
- MIT License with derivative works notice

### Changed
- Improved error handling with custom error types
- Enhanced SQL parser to handle multiple statements

### Fixed
- Fixed borrow checker issues in query executor
- Resolved column ordering in query results

## [0.1.0] - 2026-07-24

### Added
- Core storage engine
- Basic SQL parser
- Transaction manager
- CLI tool
- Documentation

---

## Version History

- **0.1.0** — Initial release with Phase 1 MVP
- **0.2.0** — Phase 2: Modern features (planned)
- **0.3.0** — Phase 3: SDKs and APIs (planned)
- **1.0.0** — Production ready (planned)

---

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.x.0`
4. Push tag: `git push origin v0.x.0`
5. Create GitHub release with release notes
