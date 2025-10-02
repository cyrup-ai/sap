use crate::color::{ColoredString, Colors, Elem};
use std::fs::Metadata;

/// Represents file system link information with a clean, modern API
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Links {
    /// Number of hard links to the file
    link_count: Option<u64>,
}

impl From<&Metadata> for Links {
    #[cfg(unix)]
    fn from(metadata: &Metadata) -> Self {
        use std::os::unix::fs::MetadataExt;

        Self {
            link_count: Some(metadata.nlink()),
        }
    }

    #[cfg(windows)]
    fn from(_metadata: &Metadata) -> Self {
        Self { link_count: None }
    }
}

impl Links {
    /// Renders the link count with appropriate styling
    pub fn render(&self, colors: &Colors) -> ColoredString {
        match self.link_count {
            Some(count) => colors.colorize(count.to_string(), &Elem::Links { valid: true }),
            None => colors.colorize(
                '—', // Using em dash for better visual appeal
                &Elem::Links { valid: false },
            ),
        }
    }

    /// Returns the number of links if available
    pub fn _count(&self) -> Option<u64> {
        self.link_count
    }

    /// Checks if link information is available
    pub fn _is_available(&self) -> bool {
        self.link_count.is_some()
    }
}
