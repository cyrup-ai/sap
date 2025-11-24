g# `sap` Architecture

## Core Design: Streaming Accumulator Pattern

`sap` uses a streaming, accumulator-based architecture inspired by reactive systems and the patterns found in cyrup's chat completions. This design enables:

1. **Single traversal** with multiple output channels
2. **Parallel processing** at every stage
3. **Progressive output** without holding entire trees in memory
4. **Pluggable transformations** via rules engine

## Key Components

### FileStream

The core abstraction - a stream of file system entries discovered via parallel traversal.

```rust
pub struct FileStream {
    inner: Box<dyn Stream<Item = Result<FileEntry>> + Send + Unpin>,
}

pub struct FileEntry {
    path: PathBuf,
    metadata: Metadata,
    // Lazy-loaded fields populated by transformers
    git_status: Option<GitStatus>,
    permissions: Option<Permissions>,
    content_summary: Option<ContentSummary>,
}
```

### Accumulators

Stateful stream transformers that aggregate entries based on output requirements.

```rust
pub trait Accumulator: Stream<Item = Result<OutputEvent>> {
    type Input;
    fn accumulate(&mut self, entry: Self::Input) -> AccumulatorAction;
}

pub enum AccumulatorAction {
    Buffer,      // Hold for aggregation
    Emit,        // Output immediately
    Transform,   // Apply rule transformation
}
```

### Output Events

Events emitted by accumulators for different output formats.

```rust
pub enum OutputEvent {
    // Human-readable events
    DirectoryHeader { path: PathBuf, stats: DirStats },
    FileRow { entry: FormattedEntry },
    TreeNode { entry: FormattedEntry, depth: usize, is_last: bool },

    // LLM events (JSON Lines)
    JsonEntry { entry: serde_json::Value },

    // Completion
    StreamComplete { total_files: usize, total_dirs: usize },
}
```

## Processing Pipeline

```
Paths → FileStream → Transformers → Accumulator → OutputEvents → Renderer
         ↑             ↑              ↑              ↑
         │             │              │              │
      Parallel      Parallel      Stateful      Format-specific
      jwalk         enrichment    aggregation    display logic
```

### 1. FileStream Creation

- Uses `jwalk` for parallel directory traversal
- Respects ignore patterns
- Handles symlinks and special files

### 2. Transformers (Parallel)

Applied via parallel map operations:

- **GitTransformer**: Adds git status via gix
- **PermissionTransformer**: Extracts permission info
- **ContentTransformer**: File type detection, size calculation
- **MetadataTransformer**: Rust Analyzer, Cargo.toml parsing (future)

### 3. Accumulators

Format-specific accumulators that maintain state:

#### TreeAccumulator

- Buffers entries to maintain parent-child relationships
- Emits complete directory nodes with children
- Calculates directory statistics
- Used for both human display AND as input to LLM processing

#### GridAccumulator

- Batches entries for column alignment
- Computes optimal column widths
- Emits formatted rows

#### LLM Processing

**Design Decision**: For LLM mode, we serialize the TreeAccumulator output directly to JSON using serde.

The flow for LLM mode:
1. TreeAccumulator produces the same tree structure as human mode
2. Output is serialized to JSON using standard serde serialization
3. JSON is output as JSONL (one JSON object per line)

This keeps it simple - same tree traversal, just different output format.

#### Future: Rig Agent Post-Processing

A Rig agent can optionally post-process this JSON output to:
- Shield against overly large outputs
- Apply intelligent filtering based on context
- Summarize large directories or file contents
- Reformat for specific LLM needs

But the core LLM mode just uses straightforward serde JSON serialization.

### 4. Renderers

Consume OutputEvents and produce final output:

- **TerminalRenderer**: Colors, icons, alignment
- **JsonRenderer**: Clean JSON Lines output
- **HtmlRenderer**: Future web UI support

## Parallelism Strategy

1. **I/O Parallelism**: jwalk handles parallel directory reading
2. **CPU Parallelism**: Transformers use rayon for parallel processing
3. **Async I/O**: Git operations, future Rust Analyzer queries
4. **Streaming**: No blocking on full tree traversal

## Rules Engine Integration

The LlmAccumulator integrates with a Rig-based rules engine:

```rust
pub struct RulesEngine {
    agent: Agent<DevstralModel>,
    rules: Vec<Rule>,
}

pub struct Rule {
    pattern: FilePattern,
    action: TransformAction,
}

pub enum TransformAction {
    Summarize { max_tokens: usize },
    Filter,
    Enrich { metadata_type: MetadataType },
    Shield { reason: String },
}
```

## Benefits

1. **Memory Efficient**: Stream processing instead of collecting entire tree
2. **Responsive**: First results appear immediately
3. **Scalable**: Handles massive directory trees
4. **Extensible**: Easy to add new transformers or output formats
5. **Testable**: Each component is isolated and mockable

## Future Enhancements

1. **Incremental Updates**: Watch mode with differential updates
2. **Distributed Processing**: Split large traversals across machines
3. **Caching Layer**: Remember expensive computations (git status, file types)
4. **Plugin System**: User-defined transformers and rules
