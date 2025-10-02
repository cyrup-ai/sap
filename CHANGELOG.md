# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-XX

### Added

- **Core Features**
  - Parallel directory traversal using `jwalk` for high performance
  - Multiple display modes: Grid, Tree, Long, OneLine
  - Comprehensive tree view with hierarchical visualization
  - Git integration showing file/directory status
  - Icon and color theming support (fancy/unicode)
  - Extensive sorting options (time, size, extension, git, version)
  - Directory grouping (first/last/none)
  - Flexible block customization (permission, user, group, size, date, name, inode, links, git)
  
- **LLM Integration**
  - JSON Lines output format for AI consumption
  - Model Context Protocol (MCP) support via Rig framework
  - Context enrichment through `--objective` and `--current-task` flags
  - Structured metadata export for codebase analysis
  
- **Filtering & Display**
  - Smart glob-based filtering during traversal (not post-processing)
  - Support for multiple `--ignore-glob` patterns
  - Display modes: All, AlmostAll, VisibleOnly, DirectoryOnly, SystemProtected
  - Recursive traversal with configurable depth limits
  - Symlink dereferencing support
  
- **Customization**
  - Flexible permission display (rwx, octal, attributes, disable)
  - Multiple size formats (default, short, bytes)
  - Date formatting options (date, locale, relative, custom)
  - Hyperlink support for terminal emulators
  - Header display for columnar output
  - Owner name truncation options

- **Configuration**
  - TOML-based configuration file support
  - Custom config file path option
  - Persistent settings for layout, icons, colors, recursion, ignore patterns
  
- **Performance**
  - Streaming architecture - process entries as they arrive
  - Parallel traversal using `rayon` and `jwalk`
  - Efficient Git integration using pure Rust `gix` library
  - Smart filtering during traversal (prevents unnecessary directory descents)
  - Metadata caching for improved performance

### Fixed

- **Tree Mode Hierarchy**
  - Fixed root entry missing from tree output (was consumed for depth calculation)
  - Fixed incorrect indentation by sorting entries by depth descending before building hierarchy
  - Fixed children not appearing under correct parents due to stale clones
  
- **Ignore Globs**
  - Fixed ignore globs filtering after traversal instead of during traversal
  - Implemented filtering via `jwalk`'s `process_read_dir` callback with `children.retain()`
  - Now completely prevents descending into ignored directories for better performance
  
- **Display Issues**
  - Removed synthetic `.` and `..` entries that interfered with tree hierarchy
  - Fixed base depth calculation (jwalk always starts at depth 0)

### Changed

- Migrated to Rust 2024 edition
- Replaced `walkdir` with `jwalk` for parallel traversal
- Replaced `git2` with `gix` for pure Rust Git integration
- Optimized tree building algorithm to process deepest entries first

### Architecture

- **Streaming Accumulator Pattern** - Single traversal feeds multiple output channels
- **Async Streaming** - Built on futures for efficient I/O
- **Metadata Transformers** - Extensible pipeline for enriching file data
- **Modular Design** - Clean separation: core, stream, llm, meta, display, flags

### Dependencies

- `jwalk` - Fast parallel directory walker
- `gix` - Pure Rust Git implementation
- `rig-core` - LLM integration with MCP support
- `rayon` - Data parallelism
- `tokio` - Async runtime
- `futures` - Async primitives
- `serde` - Serialization for JSON output
- `clap` - CLI argument parsing

### Documentation

- Comprehensive README with usage examples and CLI reference
- Architecture documentation (ARCHITECTURE.md)
- Contributing guidelines (CONTRIBUTING.md)
- Dual MIT/Apache-2.0 license

### Known Issues

- TUI mode not yet implemented
- AST metadata (via tree-sitter) planned for future release
- Cargo.toml parsing for Rust project intelligence planned

---

## Future Roadmap

See [README.md](README.md#roadmap) for planned features:

- Tree-sitter integration for AST metadata
- Cargo.toml intelligence for Rust projects
- Interactive TUI mode with ratatui
- Code-aware features (function/struct detection)
- Enhanced LLM context with content previews
- Plugin system for custom transformers

---

## Migration from lsd

SAP is designed as a drop-in replacement for `lsd`. All `lsd` command-line options are supported. Simply alias:

```bash
alias lsd='sap'
```

Key improvements over lsd:
- LLM integration with JSON Lines output
- MCP protocol support
- Smart filtering during traversal (not post-processing)
- Pure Rust Git integration with `gix`
- Enhanced performance with streaming architecture

---

[Unreleased]: https://github.com/cyrup-ai/sap/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cyrup-ai/sap/releases/tag/v0.1.0
