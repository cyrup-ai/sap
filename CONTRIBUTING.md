# Contributing to SAP

Thank you for your interest in contributing to SAP (Smart Adaptive ls)! We welcome contributions from the community.

## Code of Conduct

This project adheres to a respectful and inclusive environment. Be kind, professional, and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 2024 edition or later
- Git
- Familiarity with async Rust and the futures ecosystem

### Development Setup

```bash
# Clone the repository
git clone https://github.com/cyrup-ai/sap
cd sap

# Build the project
cargo build

# Run tests
cargo test

# Run with development binary
./target/debug/sap [args]

# Check code quality
cargo clippy
cargo fmt --check
```

## Architecture Overview

Before contributing, please read [ARCHITECTURE.md](ARCHITECTURE.md) to understand:

- **Streaming accumulator pattern** - Single traversal feeds multiple outputs
- **Core components** - FileStream, accumulators, transformers, LLM integration
- **Performance considerations** - Parallel processing, zero-copy, caching

### Key Directories

```
src/
├── core.rs              # Orchestration logic
├── stream/              # Streaming infrastructure
│   ├── mod.rs          # FileStream implementation
│   └── accumulators/   # Output format builders
├── llm/                 # LLM/MCP integration
├── meta/                # File metadata extraction
├── display.rs           # Output rendering
└── flags/               # CLI flag handling
```

## How to Contribute


### 1. Finding Issues to Work On

- Check the [GitHub Issues](https://github.com/cyrup-ai/sap/issues) page
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to let others know you're working on it

### 2. Creating a Feature Request

Before implementing a new feature:

1. Check if a similar feature has been proposed
2. Open an issue describing:
   - The problem it solves
   - Proposed implementation approach
   - Potential impact on performance/architecture
3. Wait for maintainer feedback before starting work

### 3. Reporting Bugs

When reporting bugs, include:

- **SAP version**: `sap --version`
- **OS and version**: `uname -a` (Linux/macOS) or Windows version
- **Rust version**: `rustc --version`
- **Steps to reproduce**
- **Expected vs actual behavior**
- **Minimal example** if possible

### 4. Development Workflow

1. **Fork and clone** the repository
2. **Create a branch** for your work:
   ```bash
   git checkout -b feature/my-new-feature
   # or
   git checkout -b fix/issue-123
   ```
3. **Make your changes** following the guidelines below
4. **Write tests** for new functionality
5. **Run the test suite**:
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt
   ```
6. **Commit your changes** with clear, descriptive messages
7. **Push to your fork**
8. **Open a Pull Request** with:
   - Clear description of changes
   - Reference to related issue(s)
   - Screenshots/examples if applicable

## Coding Guidelines

### Code Style

- Follow **Rust 2024 conventions**
- Use `cargo fmt` for formatting (enforced in CI)
- Use `cargo clippy` and fix all warnings
- Write **idiomatic Rust** - leverage the type system, use iterators, avoid unnecessary clones

### Performance Guidelines

SAP is performance-critical. Keep these principles in mind:

1. **Stream, don't collect** - Process entries as they arrive
2. **Parallel by default** - Use `rayon` for CPU work, `tokio` for I/O
3. **Cache aggressively** - Metadata, Git repos, parsed configs
4. **Profile before optimizing** - Use `cargo flamegraph` or `perf`
5. **Benchmark changes** - Compare before/after on large directories

### Testing

- Write **unit tests** for individual functions
- Write **integration tests** in `tests/` for end-to-end behavior
- Test edge cases: empty directories, symlinks, permissions errors
- Use `cargo test -- --nocapture` to debug with println!

Example test:

```rust
#[test]
fn test_ignore_globs_filter_directories() {
    let temp_dir = tempdir().unwrap();
    // ... setup test structure
    
    let result = run_sap(&["--tree", "--ignore-glob", "node_modules"]);
    assert!(!result.contains("node_modules"));
}
```


### Documentation

- Update **README.md** if adding user-facing features
- Update **ARCHITECTURE.md** for architectural changes
- Add **inline documentation** for public APIs
- Include **examples** in doc comments:

```rust
/// Filters directory entries during traversal
///
/// # Example
/// ```
/// let walker = WalkDir::new("src")
///     .process_read_dir(|_depth, _path, _state, children| {
///         children.retain(|entry| !is_ignored(entry));
///     });
/// ```
pub fn process_read_dir<F>(self, callback: F) -> Self { ... }
```

### Commit Messages

Write clear, descriptive commit messages:

```
Short summary (50 chars or less)

More detailed explanation if needed. Wrap at 72 characters.
Explain the problem this commit solves and why this approach
was chosen.

Fixes #123
```

**Good examples:**
- `Fix tree mode hierarchy by sorting entries by depth`
- `Add ignore_globs filtering during jwalk traversal`
- `Implement JSON Lines output for LLM integration`

**Bad examples:**
- `fix bug`
- `WIP`
- `update code`

## Pull Request Guidelines

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy is happy (`cargo clippy -- -D warnings`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for notable changes)


### PR Description Template

```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed? What problem does it solve?

## Changes
- Bullet point list of specific changes

## Testing
How was this tested? Include commands run and results.

## Related Issues
Fixes #123
Relates to #456

## Screenshots (if applicable)
Before/after comparisons for UI changes
```

### Review Process

1. Maintainers will review your PR within a few days
2. Address any requested changes
3. Once approved, a maintainer will merge your PR
4. Your contribution will be included in the next release!

## Areas Needing Help

We especially welcome contributions in these areas:

### High Priority

- **Tree-sitter integration** - AST metadata for code files
- **Cargo.toml parsing** - Project intelligence for Rust codebases
- **Performance benchmarks** - Automated benchmarking suite
- **TUI mode** - Interactive file browser with ratatui

### Good First Issues

- Documentation improvements
- Additional test coverage
- Bug fixes in existing features
- Code cleanup and refactoring

### LLM Integration

- Enhanced JSON output formats
- Additional metadata transformers
- MCP protocol extensions
- Content preview generation

## Implementation Notes

### Adding New Metadata Types


1. Create transformer in `src/stream/transformers/`
2. Implement `Transformer` trait
3. Add to pipeline in `Core::build_pipeline()`
4. Update JSON serialization in `src/llm/`

### Adding New CLI Flags

1. Add field to appropriate struct in `src/flags/`
2. Implement `Configurable` trait
3. Add to `Cli` struct in `src/app.rs`
4. Update config file schema if needed
5. Update README.md with new flag

### Debugging Tips

```bash
# Enable debug logs
RUST_LOG=debug ./target/debug/sap --tree

# Run specific test
cargo test test_name -- --nocapture

# Profile performance
cargo build --release
perf record ./target/release/sap --tree large_dir
perf report

# Memory profiling
valgrind ./target/debug/sap --tree large_dir
```

## License

By contributing to SAP, you agree that your contributions will be licensed under the same dual MIT/Apache-2.0 license. See the [LICENSE](LICENSE) file for details.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Questions?

- Open a [GitHub Discussion](https://github.com/cyrup-ai/sap/discussions)
- Join our community chat (link TBD)
- Email: david@cyrup.ai

## Acknowledgments

Thank you for contributing to SAP! Every contribution, no matter how small, helps make the project better.
