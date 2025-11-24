use crate::color::{self, ColoredString, Colors};
use crate::git::GitStatus;
use crate::git_theme::GitTheme;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GitFileStatus {
    pub index: GitStatus,
    pub workdir: GitStatus,
}

impl Default for GitFileStatus {
    fn default() -> Self {
        Self {
            index: GitStatus::Default,
            workdir: GitStatus::Default,
        }
    }
}

impl GitFileStatus {
    pub fn from_gix_status(status_info: &crate::git::GitStatusInfo) -> Self {
        Self {
            index: status_info.index_status.unwrap_or(GitStatus::Unmodified),
            workdir: status_info.workdir_status.unwrap_or(GitStatus::Unmodified),
        }
    }

    pub fn is_new(&self) -> bool {
        matches!(self.workdir, GitStatus::NewInWorkdir) || matches!(self.index, GitStatus::NewInIndex)
    }

    pub fn is_modified(&self) -> bool {
        matches!(self.workdir, GitStatus::Modified) || matches!(self.index, GitStatus::Modified)
    }

    #[allow(dead_code)]
    pub fn render(&self, colors: &Colors, git_theme: &GitTheme) -> ColoredString {
        let index_symbol = colors.colorize(
            git_theme.get_symbol(&self.index),
            &color::Elem::GitStatus { status: self.index },
        );

        let workdir_symbol = colors.colorize(
            git_theme.get_symbol(&self.workdir),
            &color::Elem::GitStatus {
                status: self.workdir,
            },
        );

        // Build the result with improved formatting
        let mut result = String::with_capacity(160);

        // Only show git status if there are actual changes
        if self.index != GitStatus::Unmodified || self.workdir != GitStatus::Unmodified {
            // Use a more subtle visual indicator
            result.push('│');

            // Show index status if modified
            if self.index != GitStatus::Unmodified {
                result.push_str(&index_symbol.to_string());
            }

            // Add separator only if both statuses are shown
            if self.index != GitStatus::Unmodified && self.workdir != GitStatus::Unmodified {
                result.push('·');
            }

            // Show workdir status if modified
            if self.workdir != GitStatus::Unmodified {
                result.push_str(&workdir_symbol.to_string());
            }

            result.push('│');
        }

        ColoredString::new(Colors::default_style(), result)
    }
}
