//! Git integration (to be integrated with streaming code)
#![allow(dead_code)]

use crate::meta::git_file_status::GitFileStatus;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum GitStatus {
    /// No status info
    #[default]
    Default,
    /// No changes (got from git status)
    Unmodified,
    /// Entry is ignored item in workdir
    Ignored,
    /// Entry does not exist in old version (now in stage)
    NewInIndex,
    /// Entry does not exist in old version (not in stage)
    NewInWorkdir,
    /// Type of entry changed between old and new
    Typechange,
    /// Entry does not exist in new version
    Deleted,
    /// Entry was renamed between old and new
    Renamed,
    /// Entry content changed between old and new
    Modified,
    /// Entry in the index is conflicted
    Conflicted,
    /// Entry is conflicted
    GitConflicted,
}

#[derive(Debug, Clone)]
pub struct GitStatusInfo {
    pub index_status: Option<GitStatus>,
    pub workdir_status: Option<GitStatus>,
}

pub struct GitCache {
    statuses: Vec<(PathBuf, GitStatusInfo)>,
}

impl GitCache {
    pub fn new(path: &Path) -> GitCache {
        // Discover the git repository from the given path
        let repo = match gix::discover(path) {
            Ok(r) => r,
            Err(_e) => {
                // Unable to retrieve Git info; it doesn't seem to be a git directory
                return Self::empty();
            }
        };

        if let Some(workdir) = repo.workdir().and_then(|x| std::fs::canonicalize(x).ok()) {
            let mut statuses = Vec::new();
            
            // Retrieving Git statuses for workdir
            match repo.status(gix::progress::Discard) {
                Ok(platform) => {
                    // Configure status to include untracked files
                    let status_iter = platform
                        .untracked_files(gix::status::UntrackedFiles::Files)
                        .into_iter(Vec::new());
                    
                    match status_iter {
                        Ok(iter) => {
                            for item in iter {
                                match item {
                                    Ok(gix::status::Item::IndexWorktree(status_item)) => {
                                        use gix::bstr::ByteSlice;
                                        let path_str = match &status_item {
                                            gix::status::index_worktree::Item::Modification { rela_path, .. } => rela_path.as_bstr(),
                                            gix::status::index_worktree::Item::DirectoryContents { entry, .. } => entry.rela_path.as_bstr(),
                                            gix::status::index_worktree::Item::Rewrite { dirwalk_entry, .. } => dirwalk_entry.rela_path.as_bstr(),
                                        };
                                        // Convert from Unix-style path to platform path
                                        // Use to_str_lossy() instead of unwrap_or_default() to handle non-UTF8 paths
                                        // with replacement characters instead of empty string
                                        let path: PathBuf = path_str
                                            .to_str_lossy()
                                            .split('/')
                                            .collect();
                                        let path = workdir.join(path);
                                        
                                        let git_status = Self::convert_gix_status(&status_item);
                                        statuses.push((path, git_status));
                                    }
                                    Ok(gix::status::Item::TreeIndex(tree_index_change)) => {
                                        use gix::bstr::ByteSlice;
                                        use gix::diff::index::Change;

                                        // Extract the relative path from the change
                                        let location = match &tree_index_change {
                                            Change::Addition { location, .. } => location.as_ref(),
                                            Change::Deletion { location, .. } => location.as_ref(),
                                            Change::Modification { location, .. } => location.as_ref(),
                                            Change::Rewrite { location, .. } => location.as_ref(),
                                        };

                                        // Convert from Unix-style path to platform PathBuf
                                        // Use to_str_lossy() instead of unwrap_or_default() to handle non-UTF8 paths
                                        // with replacement characters instead of empty string
                                        let path: PathBuf = location
                                            .to_str_lossy()
                                            .split('/')
                                            .collect();
                                        let path = workdir.join(path);

                                        // Create status info for TreeIndex changes
                                        let git_status = Self::convert_tree_index_status(&tree_index_change);
                                        statuses.push((path, git_status));
                                    }
                                    Err(err) => {
                                        crate::print_error!("Error processing status item: {}", err);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            crate::print_error!(
                                "Cannot create status iterator for directory {:?}: {}",
                                workdir,
                                err
                            );
                        }
                    }
                }
                Err(err) => {
                    crate::print_error!(
                        "Cannot retrieve Git statuses for directory {:?}: {}",
                        workdir,
                        err
                    );
                }
            }

            GitCache { statuses }
        } else {
            // No workdir
            Self::empty()
        }
    }

    pub fn empty() -> Self {
        GitCache {
            statuses: Vec::new(),
        }
    }

    pub fn get(&self, filepath: &PathBuf, is_directory: bool) -> Option<GitFileStatus> {
        match std::fs::canonicalize(filepath) {
            Ok(filename) => Some(self.inner_get(&filename, is_directory)),
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    crate::print_error!("Cannot get git status for {:?}:  {}", filepath, err);
                }
                None
            }
        }
    }

    fn inner_get(&self, filepath: &PathBuf, is_directory: bool) -> GitFileStatus {
        if is_directory {
            self.statuses
                .iter()
                .filter(|&x| x.0.starts_with(filepath))
                .map(|x| GitFileStatus::from_gix_status(&x.1))
                .fold(GitFileStatus::default(), |acc, x| GitFileStatus {
                    index: std::cmp::max(acc.index, x.index),
                    workdir: std::cmp::max(acc.workdir, x.workdir),
                })
        } else {
            self.statuses
                .iter()
                .find(|&x| filepath == &x.0)
                .map(|e| GitFileStatus::from_gix_status(&e.1))
                .unwrap_or_default()
        }
    }
    
    fn convert_gix_status(item: &gix::status::index_worktree::Item) -> GitStatusInfo {
        match item {
            gix::status::index_worktree::Item::Modification { status, .. } => {
                use gix::status::plumbing::index_as_worktree::{Change, EntryStatus};
                match status {
                    EntryStatus::Conflict(_) => GitStatusInfo {
                        index_status: Some(GitStatus::Conflicted),
                        workdir_status: Some(GitStatus::Conflicted),
                    },
                    EntryStatus::Change(change) => match change {
                        Change::Removed => GitStatusInfo {
                            index_status: None,
                            workdir_status: Some(GitStatus::Deleted),
                        },
                        Change::Type { .. } => GitStatusInfo {
                            index_status: None,
                            workdir_status: Some(GitStatus::Typechange),
                        },
                        Change::Modification { .. } => GitStatusInfo {
                            index_status: None,
                            workdir_status: Some(GitStatus::Modified),
                        },
                        Change::SubmoduleModification(_) => GitStatusInfo {
                            index_status: None,
                            workdir_status: Some(GitStatus::Modified),
                        },
                    },
                    EntryStatus::NeedsUpdate(_) => GitStatusInfo {
                        index_status: None,
                        workdir_status: None,
                    },
                    EntryStatus::IntentToAdd => GitStatusInfo {
                        index_status: Some(GitStatus::NewInIndex),
                        workdir_status: None,
                    },
                }
            }
            gix::status::index_worktree::Item::DirectoryContents { entry, .. } => {
                use gix::dir::entry::Status;
                match entry.status {
                    Status::Untracked => GitStatusInfo {
                        index_status: None,
                        workdir_status: Some(GitStatus::NewInWorkdir),
                    },
                    Status::Ignored(_) => GitStatusInfo {
                        index_status: None,
                        workdir_status: Some(GitStatus::Ignored),
                    },
                    _ => GitStatusInfo {
                        index_status: None,
                        workdir_status: None,
                    },
                }
            }
            gix::status::index_worktree::Item::Rewrite { copy, .. } => {
                if *copy {
                    GitStatusInfo {
                        index_status: None,
                        workdir_status: Some(GitStatus::Modified),
                    }
                } else {
                    GitStatusInfo {
                        index_status: None,
                        workdir_status: Some(GitStatus::Renamed),
                    }
                }
            }
        }
    }

    fn convert_tree_index_status(change: &gix::diff::index::Change) -> GitStatusInfo {
        use gix::diff::index::Change;

        match change {
            Change::Addition { .. } => GitStatusInfo {
                index_status: Some(GitStatus::NewInIndex),
                workdir_status: Some(GitStatus::Unmodified),
            },
            Change::Deletion { .. } => GitStatusInfo {
                index_status: Some(GitStatus::Deleted),
                workdir_status: Some(GitStatus::Unmodified),
            },
            Change::Modification { .. } => GitStatusInfo {
                index_status: Some(GitStatus::Modified),
                workdir_status: Some(GitStatus::Unmodified),
            },
            Change::Rewrite { .. } => GitStatusInfo {
                index_status: Some(GitStatus::Renamed),
                workdir_status: Some(GitStatus::Unmodified),
            },
        }
    }
}