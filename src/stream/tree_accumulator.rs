//! Tree accumulator (replaced by in-memory tree building in Core)
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;

use crate::stream::{FileEntry, StreamResult};

/// Tree accumulator that buffers entries and emits formatted tree output
pub struct TreeAccumulator {
    source: Pin<Box<dyn Stream<Item = StreamResult<FileEntry>> + Send>>,
    buffer: HashMap<PathBuf, Vec<FileEntry>>,
    pending_output: Vec<String>,
    is_complete: bool,
    root: PathBuf,
}

impl TreeAccumulator {
    pub fn new(
        source: impl Stream<Item = StreamResult<FileEntry>> + Send + 'static,
        root: PathBuf,
    ) -> Self {
        Self {
            source: Box::pin(source),
            buffer: HashMap::new(),
            pending_output: Vec::new(),
            is_complete: false,
            root,
        }
    }
    
    /// Recursively render tree from buffered entries
    fn render_tree(&self, path: &PathBuf, depth: usize, _is_last_sibling: bool, prefix: &str) -> Vec<String> {
        let mut lines = Vec::new();
        
        if let Some(entries) = self.buffer.get(path) {
            let mut sorted = entries.clone();
            // Sort: directories first, then alphabetically (like src/sort.rs)
            sorted.sort_by(|a, b| {
                let a_is_dir = matches!(a.file_type, crate::meta::FileType::Directory { .. });
                let b_is_dir = matches!(b.file_type, crate::meta::FileType::Directory { .. });
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            });
            
            for (i, entry) in sorted.iter().enumerate() {
                let is_last = i == sorted.len() - 1;
                
                // Build tree characters
                let branch = if is_last { "└── " } else { "├── " };
                let icon = if matches!(entry.file_type, crate::meta::FileType::Directory { .. }) {
                    "📁"
                } else {
                    "📄"
                };
                
                lines.push(format!("{}{}{} {}", prefix, branch, icon, entry.name));
                
                // Recurse into directories
                if matches!(entry.file_type, crate::meta::FileType::Directory { .. }) {
                    let new_prefix = if is_last {
                        format!("{}    ", prefix)  // Spaces for last branch
                    } else {
                        format!("{}│   ", prefix)  // Vertical bar for continuing branch
                    };
                    lines.extend(self.render_tree(&entry.path, depth + 1, is_last, &new_prefix));
                }
            }
        }
        
        lines
    }
}

impl Stream for TreeAccumulator {
    type Item = StreamResult<String>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Drain pending output first
        if let Some(line) = self.pending_output.pop() {
            return Poll::Ready(Some(Ok(line)));
        }
        
        // Buffer entries from source until exhausted
        loop {
            match self.source.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(entry))) => {
                    // Buffer by parent directory
                    let parent = entry.path.parent()
                        .unwrap_or(&self.root)
                        .to_path_buf();
                    self.buffer.entry(parent).or_default().push(entry);
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    // Source exhausted - render tree once
                    if !self.is_complete {
                        self.is_complete = true;
                        
                        // Find the actual root path in the buffer (usually the input path like "./src")
                        // The buffer contains entries organized by their parent paths
                        let actual_root = self.buffer.keys()
                            .filter(|p| p.parent() == Some(self.root.as_path()) || **p == self.root)
                            .min()
                            .cloned()
                            .unwrap_or_else(|| self.root.clone());
                        
                        self.pending_output = self.render_tree(&actual_root, 0, true, "");
                        self.pending_output.reverse();  // Pop from end for efficiency
                    }
                    
                    // Emit pending or signal completion
                    return match self.pending_output.pop() {
                        Some(line) => Poll::Ready(Some(Ok(line))),
                        None => Poll::Ready(None),
                    };
                }
                Poll::Pending => {
                    // No more entries available yet
                    return Poll::Pending;
                }
            }
        }
    }
}
