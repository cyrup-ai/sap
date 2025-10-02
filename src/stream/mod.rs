use futures::{Stream, StreamExt};
use jwalk::DirEntry;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

mod aggregated_chat_stream;
// mod llm_stream;
mod tree_accumulator;

pub use aggregated_chat_stream::AggregatedChatStream;

use crate::git::GitStatusInfo;
use crate::meta::{FileType, Permissions};

/// A file system entry discovered during traversal
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub file_type: FileType,
    pub metadata: std::fs::Metadata,
    pub depth: usize,
    pub is_symlink: bool,

    // Lazy-loaded fields populated by transformers (planned for future optimization)
    pub git_status: Option<GitStatusInfo>,
    #[allow(dead_code)]
    pub permissions: Option<crate::meta::Permissions>,
    #[allow(dead_code)]
    pub size: Option<crate::meta::Size>,
    #[allow(dead_code)]
    pub modified: Option<crate::meta::Date>,
}

impl FileEntry {
    pub fn from_jwalk(
        entry: DirEntry<((), ())>,
        base_depth: usize,
    ) -> Result<Self, std::io::Error> {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata()?;
        let permissions = Permissions::from(&metadata);
        let file_type = FileType::new(&metadata, None, &permissions);
        let depth = entry.depth() - base_depth;
        let is_symlink = metadata.file_type().is_symlink();

        Ok(FileEntry {
            path,
            name,
            file_type,
            metadata,
            depth,
            is_symlink,
            git_status: None,
            permissions: None,
            size: None,
            modified: None,
        })
    }

    /// Convert FileEntry to Meta using already-loaded metadata
    pub fn to_meta(&self, permission_flag: crate::flags::PermissionFlag) -> crate::meta::Meta {
        use crate::meta::*;

        #[cfg(unix)]
        let (owner, permissions) = match permission_flag {
            crate::flags::PermissionFlag::Disable => (None, None),
            _ => (
                Some(Owner::from(&self.metadata)),
                Some(Permissions::from(&self.metadata)),
            ),
        };
        #[cfg(unix)]
        let permissions_or_attributes = permissions.map(crate::meta::permissions_or_attributes::PermissionsOrAttributes::Permissions);

        #[cfg(windows)]
        let (owner, permissions_or_attributes) = match permission_flag {
            crate::flags::PermissionFlag::Disable => (None, None),
            crate::flags::PermissionFlag::Attributes => {
                use crate::meta::permissions_or_attributes::get_attributes;
                (
                    None,
                    Some(crate::meta::permissions_or_attributes::PermissionsOrAttributes::WindowsAttributes(get_attributes(
                        &self.metadata,
                    ))),
                )
            },
            _ => {
                #[cfg(windows)]
                match crate::meta::windows_utils::get_file_data(&self.path) {
                    Ok((owner_win, permissions_win)) => (
                        Some(owner_win),
                        Some(crate::meta::permissions_or_attributes::PermissionsOrAttributes::Permissions(permissions_win)),
                    ),
                    Err(_e) => (None, None),
                }
                #[cfg(not(windows))]
                (None, None)
            }
        };

        // Use FileEntry.name if it differs from path-derived name (for special entries like . and ..)
        let name = if self.name == "." || self.name == ".." {
            // Create Name directly with our custom name
            Name::new(&std::path::PathBuf::from(&self.name), self.file_type)
        } else {
            Name::new(&self.path, self.file_type)
        };

        Meta {
            inode: Some(INode::from(&self.metadata)),
            links: Some(Links::from(&self.metadata)),
            path: self.path.clone(),
            symlink: SymLink::from(self.path.as_path()),
            size: Some(Size::from(&self.metadata)),
            date: Some(Date::from(&self.metadata)),
            indicator: Indicator::from(self.file_type),
            owner,
            permissions_or_attributes,
            name,
            file_type: self.file_type,
            content: None,
            access_control: Some(AccessControl::for_path(&self.path)),
            git_status: self.git_status.as_ref().map(|info| GitFileStatus::from_gix_status(info)),
        }
    }
}

/// Result type for stream operations
pub type StreamResult<T> = Result<T, StreamError>;

/// Errors that can occur during stream processing
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    #[allow(dead_code)]
    Git(String),

    #[error("Traversal error: {0}")]
    Traversal(String),
}

/// Core file stream that produces entries from directory traversal
pub struct FileStream {
    inner: Pin<Box<dyn Stream<Item = StreamResult<FileEntry>> + Send>>,
}

impl FileStream {
    /// Create a new file stream from the given paths
    pub fn new(
        paths: Vec<PathBuf>,
        max_depth: usize,
        ignore_globs: &crate::flags::IgnoreGlobs,
        display: crate::flags::Display,
    ) -> Self {
        let ignore_globs = ignore_globs.clone();
        
        // Create a stream that processes all paths
        let stream = futures::stream::iter(paths.into_iter())
            .flat_map(move |path| {
                let ignore_globs = ignore_globs.clone();
                let display_mode = display;
                
                // Create jwalk walker for this path
                let ignore_globs_for_callback = ignore_globs.clone();
                let walker = jwalk::WalkDir::new(&path)
                    .max_depth(max_depth)
                    .sort(true)
                    .skip_hidden(false)
                    .follow_links(false)
                    .parallelism(jwalk::Parallelism::RayonNewPool(0))
                    .process_read_dir(move |_depth, _path, _state, children| {
                        // Filter out ignored entries during traversal (prevents descending)
                        children.retain(|dir_entry_result| {
                            dir_entry_result.as_ref().map(|dir_entry| {
                                dir_entry.file_name.to_str()
                                    .map(|name| !ignore_globs_for_callback.is_match(std::ffi::OsStr::new(name)))
                                    .unwrap_or(true)
                            }).unwrap_or(true)
                        });
                    });
                
                // jwalk always starts at depth 0 for the root path
                let base_depth = 0;
                let walker_iter = walker.into_iter();
                
                // For Display::All in non-tree modes, prepend .. and . entries
                // (Tree mode doesn't need these - they interfere with hierarchy)
                let special_entries: Vec<StreamResult<FileEntry>> = Vec::new();
                
                // Chain special entries with walker stream
                futures::stream::iter(special_entries)
                    .chain(
                        futures::stream::iter(walker_iter)
                            .filter_map(move |entry_result| {
                                match entry_result {
                                    Ok(entry) => {
                                        // Apply display mode filter (ignore_globs now in process_read_dir)
                                        if let Some(name) = entry.file_name().to_str() {
                                            // Apply display mode filter
                                            use crate::flags::Display;
                                            match display_mode {
                                                Display::VisibleOnly => {
                                                    if name.starts_with('.') {
                                                        return futures::future::ready(None);
                                                    }
                                                }
                                                Display::AlmostAll => {
                                                    if name == "." || name == ".." {
                                                        return futures::future::ready(None);
                                                    }
                                                }
                                                Display::All => {}
                                                Display::DirectoryOnly => {
                                                    if !entry.file_type().is_dir() {
                                                        return futures::future::ready(None);
                                                    }
                                                }
                                                Display::SystemProtected => {}
                                            }
                                        }
                                        
                                        // Convert to FileEntry
                                        match FileEntry::from_jwalk(entry, base_depth) {
                                            Ok(file_entry) => futures::future::ready(Some(Ok(file_entry))),
                                            Err(e) => futures::future::ready(Some(Err(StreamError::Io(e)))),
                                        }
                                    }
                                    Err(e) => futures::future::ready(Some(Err(StreamError::Traversal(e.to_string())))),
                                }
                            })
                    )
            });

        FileStream {
            inner: Box::pin(stream),
        }
    }
}

impl Stream for FileStream {
    type Item = StreamResult<FileEntry>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Output events emitted by accumulators (planned for future streaming optimization)
#[allow(dead_code)]
#[derive(Debug)]
pub enum OutputEvent {
    /// Header for a directory in human-readable output
    DirectoryHeader {
        path: PathBuf,
        file_count: usize,
        dir_count: usize,
        total_size: u64,
    },

    /// A formatted file/directory entry for display
    FileRow {
        entry: FileEntry,
        formatted: Vec<String>, // Pre-formatted columns
    },

    /// Tree node with hierarchy information
    TreeNode {
        entry: FileEntry,
        is_last: bool,
        prefix: String,
    },

    /// Stream completion event
    StreamComplete {
        total_files: usize,
        total_dirs: usize,
    },
}

/// Trait for accumulator implementations (planned for future streaming optimization)
#[allow(dead_code)]
pub trait Accumulator: Stream<Item = StreamResult<OutputEvent>> + Unpin {
    /// Process a file entry
    fn process_entry(&mut self, entry: FileEntry) -> AccumulatorAction;
}

/// Actions an accumulator can take when processing an entry (planned for future streaming optimization)
#[allow(dead_code)]
#[derive(Debug)]
pub enum AccumulatorAction {
    /// Buffer the entry for later emission
    Buffer,

    /// Emit one or more events immediately
    Emit(Vec<OutputEvent>),

    /// Request transformation via rules engine
    Transform,
}
