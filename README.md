<div align="center">
  <img src="assets/icon.png" alt="SAP Logo" width="200"/>

  # SAP - a drop in `ls` replacement

  **A blazing-fast drop-in replacement for `lsd`**

  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
  [![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
</div>

---

## Overview

SAP (Smart Adaptive ls) is a high-performance file system tool that combines the best features of `lsd`. Built in Rust 2024, it provides beautiful human-friendly terminal output with modern features.

### Why SAP?

- **🚀 Blazing Fast** - Parallel directory traversal with `jwalk` and `rayon`
- **🔄 Drop-in Replacement** - Compatible with `lsd` command-line options
- **🎨 Beautiful Output** - Rich formatting with icons, colors, and tree views
- **⚡ Smart Filtering** - Efficient glob-based exclusions during traversal
- **📊 Git Integration** - Display git status directly in listings

---

## Installation

```bash
cargo install --git https://github.com/cyrup-ai/sap
```

---

## Quick Start

```bash
# Basic usage (drop-in replacement for lsd)
sap

# Tree view with ignored directories
sap --tree --ignore-glob 'node_modules' --ignore-glob '.git'

# Long format with git status
sap -l --git
```

---

## Core Features

### Display Modes

| Mode | Flag | Description |
|------|------|-------------|
| Grid | *(default)* | Multi-column grid layout |
| Tree | `--tree` | Recursive tree visualization |
| Long | `-l, --long` | Extended metadata table |
| One Line | `-1, --oneline` | Single entry per line |

### Filtering & Display

```bash
# Show all files including hidden
sap -a, --all

# Show almost all (exclude . and ..)
sap -A, --almost-all

# Ignore patterns (supports multiple)
sap -I, --ignore-glob '*.log' --ignore-glob 'tmp'

# Directory only view
sap -d, --directory-only

# Literal names (no quoting)
sap -N, --literal
```

### Recursion & Depth

```bash
# Recursive listing
sap -R, --recursive

# Tree view with depth limit
sap --tree --depth 3

# Unlimited depth tree
sap --tree  # Uses max depth by default
```

### Sorting Options

```bash
# Time-based sorting
sap -t, --timesort

# Size-based sorting
sap -S, --sizesort

# Extension sorting
sap -X, --extensionsort

# Git status sorting
sap -G, --gitsort

# Natural version sorting
sap -v, --versionsort

# Custom sort type
sap --sort <TYPE>  # size|time|version|extension|git|none

# Disable sorting (directory order)
sap -U, --no-sort

# Reverse order
sap -r, --reverse

# Group directories
sap --group-dirs <first|last|none>
sap --group-directories-first  # Alias for --group-dirs=first
```

### Customization

```bash
# Color control
sap --color <always|auto|never>

# Icon settings
sap --icon <always|auto|never>
sap --icon-theme <fancy|unicode>

# Permission display
sap --permission <rwx|octal|attributes|disable>

# Size display format
sap --size <default|short|bytes>

# Date format
sap --date <date|locale|relative|+custom-format>

# Custom blocks (choose what to display)
sap --blocks <permission,user,group,size,date,name,inode,links,git>

# Classic mode (ls-like output)
sap --classic
```

### Advanced Features

```bash
# Display total directory sizes
sap --total-size

# Show inode numbers
sap -i, --inode

# Git status indicators
sap -g, --git  # (requires --long)

# Dereference symlinks
sap -L, --dereference

# Security context (SELinux)
sap -Z, --context

# Hyperlinks to files
sap --hyperlink <always|auto|never>

# Display column headers
sap --header

# Truncate long owner names
sap --truncate-owner-after <NUM>
sap --truncate-owner-marker <STR>

# Don't display symlink targets
sap --no-symlink
```

---

## Configuration

SAP supports configuration files for persistent settings:

```bash
# Use custom config
sap --config-file ~/.config/sap/config.toml

# Ignore default config
sap --ignore-config
```

### Config File Example

```toml
# ~/.config/sap/config.toml
layout = "tree"
icon-theme = "fancy"
color = "always"

[recursion]
enabled = true
depth = 5

[ignore_globs]
patterns = ["node_modules", ".git", "target", "*.log"]
```

---

## Performance Features

SAP is built for speed:

- **Parallel Traversal** - Uses `jwalk` for multi-threaded directory walking
- **Streaming Architecture** - Process entries as they arrive, no buffering overhead
- **Smart Filtering** - Ignore patterns applied during traversal (not post-processing)
- **Efficient Git Integration** - Uses `gix` (pure Rust) instead of libgit2
- **Zero-Copy Where Possible** - Minimize allocations and data copying

### Benchmarks

```bash
# Typical speedup vs traditional ls
sap --tree large_project/     # ~3-5x faster than lsd
sap -R --ignore-glob 'node_modules'  # Filters during traversal
```

---

## Architecture Highlights

- **Rust 2024 Edition** - Latest language features and optimizations
- **Async Streaming** - Futures-based for efficient I/O
- **Parallel Processing** - Multi-threaded directory traversal
- **Git-Aware** - Native repository detection and status tracking

---

## Complete CLI Reference

### General Options

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-a` | `--all` | Show all entries including hidden (starting with .) |
| `-A` | `--almost-all` | Show all except . and .. |
| `-F` | `--classify` | Append indicator to filenames (*/=>@\|) |
| `-l` | `--long` | Long format with extended metadata |
| `-1` | `--oneline` | One entry per line |
| `-R` | `--recursive` | Recurse into directories |
| `-h` | `--human-readable` | Human-readable sizes (default) |
| `-d` | `--directory-only` | List directories themselves, not contents |
| `-i` | `--inode` | Show inode numbers |
| `-g` | `--git` | Show git status (requires -l) |
| `-L` | `--dereference` | Follow symbolic links |
| `-Z` | `--context` | Show security context |
| `-N` | `--literal` | Don't quote entry names |
| `-V` | `--version` | Show version |
|      | `--help` | Show help information |

### Layout Options

| Flag | Description |
|------|-------------|
| `--tree` | Tree view with hierarchical structure |
| `--depth <NUM>` | Maximum recursion depth |
| `--classic` | Classic ls-style output |

### Sort Options

| Flag | Long Form | Values | Description |
|------|-----------|--------|-------------|
| `-t` | `--timesort` | - | Sort by modification time |
| `-S` | `--sizesort` | - | Sort by file size |
| `-X` | `--extensionsort` | - | Sort by file extension |
| `-G` | `--gitsort` | - | Sort by git status |
| `-v` | `--versionsort` | - | Natural version number sort |
| `-U` | `--no-sort` | - | No sorting (directory order) |
| `-r` | `--reverse` | - | Reverse sort order |
|      | `--sort` | `size\|time\|version\|extension\|git\|none` | Specify sort type |
|      | `--group-dirs` | `first\|last\|none` | Group directories |
|      | `--group-directories-first` | - | Alias for --group-dirs=first |

### Display Customization

| Flag | Values | Description |
|------|--------|-------------|
| `--color` | `always\|auto\|never` | Color output control |
| `--icon` | `always\|auto\|never` | Icon display control |
| `--icon-theme` | `fancy\|unicode` | Icon style |
| `--permission` | `rwx\|octal\|attributes\|disable` | Permission format |
| `--size` | `default\|short\|bytes` | Size display format |
| `--date` | `date\|locale\|relative\|+format` | Date format |
| `--hyperlink` | `always\|auto\|never` | Hyperlink files |
| `--blocks` | `permission,user,group,size,date,name,inode,links,git` | Custom block order |
| `--header` | - | Display block headers |
| `--total-size` | - | Show total directory sizes |
| `--no-symlink` | - | Don't show symlink targets |
| `--truncate-owner-after` | `<NUM>` | Truncate owner names after N chars |
| `--truncate-owner-marker` | `<STR>` | Marker for truncated names |

### Filtering

| Flag | Description |
|------|-------------|
| `-I, --ignore-glob <PATTERN>` | Exclude files matching glob (repeatable) |

### Configuration

| Flag | Description |
|------|-------------|
| `--ignore-config` | Ignore configuration file |
| `--config-file <PATH>` | Use custom config file |

---

## Examples

### Basic Usage

```bash
# Simple listing
sap

# Show hidden files
sap -a

# Long format with icons
sap -l --icon always

# Colored output even when piped
sap --color always | less -R
```

### Tree Views

```bash
# Basic tree
sap --tree

# Tree with depth limit
sap --tree --depth 2

# Tree with filters
sap --tree --ignore-glob 'node_modules' --ignore-glob '.git' --ignore-glob 'target'

# Tree with directories only
sap --tree -d
```

### Sorting Examples

```bash
# Sort by size, largest first
sap -S -r

# Sort by modification time (newest first)
sap -t

# Natural version sorting
sap -v

# Git status sorting with details
sap -l -g -G

# Group directories first, sort by size
sap --group-directories-first -S
```

### Advanced Filtering

```bash
# Ignore multiple patterns
sap --tree -I '*.log' -I 'tmp' -I '.cache'

# Show only specific file types
sap | grep '.rs$'

# Custom blocks
sap --blocks permission,size,name
```

### Custom Formatting

```bash
# Octal permissions
sap -l --permission octal

# Short size format
sap -l --size short

# Relative dates
sap -l --date relative

# Custom date format
sap -l --date '+%Y-%m-%d %H:%M'

# Unicode icons
sap --icon-theme unicode
```

---

## Comparison with lsd

SAP is designed as a drop-in replacement for lsd with additional features:

| Feature | lsd | SAP |
|---------|-----|-----|
| Tree view | ✅ | ✅ |
| Git integration | ✅ | ✅ |
| Icons & colors | ✅ | ✅ |
| Parallel traversal | ✅ | ✅ Enhanced |
| Smart filtering | ❌ | ✅ (during traversal) |
| Streaming architecture | ❌ | ✅ |
| Pure Rust Git (`gix`) | ❌ | ✅ |

### Migration from lsd

Simply alias or replace:

```bash
# In your shell config (.bashrc, .zshrc, etc.)
alias lsd='sap'

# Or install as lsd replacement
ln -s $(which sap) /usr/local/bin/lsd
```

All lsd commands work identically:

```bash
lsd -la --tree    # Works exactly the same with sap
```

---

## Contributing

Contributions are welcome! Please see [ARCHITECTURE.md](ARCHITECTURE.md) for development guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/cyrup-ai/sap
cd sap

# Build
cargo build --release

# Run tests
cargo test

# Run with development binary
./target/debug/sap [args]

# Check code
cargo clippy
```

### Project Structure

- `src/core.rs` - Core orchestration logic
- `src/stream/` - Streaming infrastructure
- `src/meta/` - File metadata extraction
- `src/display.rs` - Output rendering
- `src/flags/` - CLI flag handling

---

## Roadmap

- [ ] AST metadata for code files (via tree-sitter)
- [ ] Cargo.toml intelligence for Rust projects
- [ ] Interactive TUI mode (via ratatui)
- [ ] Code-aware features (function/struct detection)
- [ ] Plugin system for custom transformers
- [ ] Performance profiling dashboard

---

## License

SAP is dual-licensed under your choice of:

* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

This means you can choose either license when using this software.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

## Acknowledgments

- Built on top of the excellent [jwalk](https://github.com/byron/jwalk) library
- Inspired by [lsd](https://github.com/lsd-rs/lsd)
- Git support through [gix](https://github.com/Byron/gitoxide)

---

<div align="center">
  
  **[Documentation](https://github.com/cyrup-ai/sap)** • 
  **[Report Issues](https://github.com/cyrup-ai/sap/issues)** • 
  **[Contribute](https://github.com/cyrup-ai/sap/pulls)**
  
  Made with ❤️ by [Cyrup AI](https://github.com/cyrup-ai)
  
</div>
