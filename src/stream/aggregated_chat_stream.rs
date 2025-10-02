use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use serde_json::json;

use crate::stream::{FileEntry, StreamResult, StreamError};
use crate::meta::{Permissions, Size, Date, Owner, INode, Links};

/// Streams JSONL output for LLM consumption
/// Transforms FileEntry → JSON line-by-line without buffering
pub struct AggregatedChatStream {
    source: Pin<Box<dyn Stream<Item = StreamResult<FileEntry>> + Send>>,
    objective: Option<String>,
    current_task: Option<String>,
}

impl AggregatedChatStream {
    pub fn new(
        source: impl Stream<Item = StreamResult<FileEntry>> + Send + 'static,
        objective: Option<String>,
        current_task: Option<String>,
    ) -> Self {
        Self {
            source: Box::pin(source),
            objective,
            current_task,
        }
    }
    
    /// Convert FileEntry to JSON matching format in src/core.rs:181-194
    fn entry_to_json(&self, entry: &FileEntry) -> serde_json::Value {
        // Use From<&Metadata> conversions like in src/meta/ modules
        let permissions = Permissions::from(&entry.metadata);
        let size = Size::from(&entry.metadata);
        let date = Date::from(&entry.metadata);
        
        #[cfg(unix)]
        let owner = Some(Owner::from(&entry.metadata));
        #[cfg(not(unix))]
        let owner: Option<Owner> = None;
        
        #[cfg(unix)]
        let inode = Some(INode::from(&entry.metadata));
        #[cfg(not(unix))]
        let inode: Option<INode> = None;
        
        #[cfg(unix)]
        let links = Some(Links::from(&entry.metadata));
        #[cfg(not(unix))]
        let links: Option<Links> = None;
        
        json!({
            "path": entry.path.to_string_lossy(),
            "name": entry.name,
            "type": format!("{:?}", entry.file_type),
            "size": size.get_bytes(),
            "modified": format!("{:?}", date),
            "permissions": format!("{:?}", permissions),
            "owner": owner.map(|o| format!("{:?}", o)),
            "symlink": entry.is_symlink,
            "inode": inode.map(|i| format!("{:?}", i)),
            "links": links.map(|l| format!("{:?}", l)),
            "git_status": entry.git_status.as_ref().map(|gs| format!("{:?}", gs)),
            "depth": entry.depth,
            "objective": self.objective.clone(),
            "current_task": self.current_task.clone(),
        })
    }
}

impl Stream for AggregatedChatStream {
    type Item = StreamResult<String>;  // JSONL strings
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.source.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(entry))) => {
                let json = self.entry_to_json(&entry);
                match serde_json::to_string(&json) {
                    Ok(line) => Poll::Ready(Some(Ok(line))),
                    Err(e) => Poll::Ready(Some(Err(StreamError::Traversal(e.to_string())))),
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
