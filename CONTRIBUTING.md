# Contributing to Tywindb

Thank you for your interest in contributing to Tywindb! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Community](#community)

---

## Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive in all interactions.

---

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- A text editor (VS Code with rust-analyzer recommended)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/tywindb.git
   cd tywindb
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/tywindb/tywindb.git
   ```

---

## Development Setup

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Linter

```bash
cargo clippy -- -D warnings
```

### Format Code

```bash
cargo fmt
```

---

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [GitHub Issues](https://github.com/tywindb/tywindb/issues)
2. If not, create a new issue with:
   - A clear, descriptive title
   - Steps to reproduce the issue
   - Expected behavior
   - Actual behavior
   - Environment information (OS, Rust version, etc.)

### Suggesting Features

1. Check [GitHub Discussions](https://github.com/tywindb/tywindb/discussions) for existing proposals
2. Create a new discussion with:
   - A clear description of the feature
   - Use cases
   - Potential implementation approach

### Submitting Changes

1. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and commit:
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

3. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

4. Create a Pull Request on GitHub

---

## Coding Standards

### Rust Style

- Follow the [Rust Style Guide](https://rust-lang.github.io/rustfmt/)
- Use `cargo fmt` before committing
- Use `cargo clippy` to catch common mistakes

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — A new feature
- `fix:` — A bug fix
- `docs:` — Documentation changes
- `style:` — Code style changes (formatting, etc.)
- `refactor:` — Code refactoring
- `test:` — Adding or updating tests
- `chore:` — Maintenance tasks

Examples:
```
feat: add vector search support
fix: resolve memory leak in storage engine
docs: update API documentation
test: add unit tests for BM25 search
```

### Code Organization

- Keep functions focused and small
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Handle errors explicitly with `Result`

---

## Testing

### Unit Tests

Write unit tests for new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function();
        assert_eq!(result, expected_value);
    }
}
```

### Integration Tests

For larger features, add integration tests in the `tests/` directory.

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

---

## Documentation

### Code Documentation

Add documentation comments for public items:

```rust
/// Brief description of the function.
///
/// # Arguments
///
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// let result = my_function(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn my_function(arg1: i32, arg2: i32) -> i32 {
    arg1 + arg2
}
```

### README Updates

Update the README.md if your changes:
- Add new features
- Change installation steps
- Modify usage examples

---

## Community

### Getting Help

- **GitHub Discussions** — Ask questions and share ideas
- **GitHub Issues** — Report bugs and request features
- **Discord** — Real-time chat (coming soon)

### Recognition

Contributors are recognized in:
- The Contributors section of README.md
- Release notes for significant contributions

---

## License

By contributing to Tywindb, you agree that your contributions will be licensed under the [MIT License](LICENSE).

---

Thank you for contributing to Tywindb! 🚀
