//! # SAP - Smart Adaptive ls
//!
//! A high-performance file system listing library with modern features.
//!
//! SAP provides a programmatic API for listing directory contents with rich
//! formatting options including icons, colors, tree views, and Git integration.
//!
//! ## Basic Usage
//!
//! ```no_run
//! use sap::{Core, Flags, Config, ExitCode};
//! use clap::Parser;
//!
//! #[derive(clap::Parser)]
//! struct Cli {
//!     #[arg(default_value = ".")]
//!     paths: Vec<std::path::PathBuf>,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let cli = Cli::parse();
//!     let config = Config::default();
//!     let flags = Flags::default();
//!     let core = Core::new(flags);
//!
//!     let exit_code = core.run(cli.paths).await;
//!     std::process::exit(exit_code as i32);
//! }
//! ```
//!
//! ## Features
//!
//! - **Parallel Directory Traversal**: Uses `jwalk` for multi-threaded file system walking
//! - **Rich Formatting**: Icons, colors, and customizable display modes
//! - **Tree View**: Hierarchical directory visualization
//! - **Git Integration**: Display git status alongside file information
//! - **Streaming Architecture**: Process files as they're discovered
//! - **Flexible Configuration**: CLI args, config files, and environment variables

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::match_same_arms,
    clippy::cast_possible_wrap
)]

// Core modules - re-export public APIs
pub mod app;
pub mod color;
pub mod config_file;
pub mod core;
pub mod display;
pub mod flags;
pub mod git;
pub mod git_theme;
pub mod icon;
pub mod meta;
pub mod presentation;
pub mod sort;
pub mod stream;
pub mod theme;

// Re-export commonly used types at the crate root for convenience
pub use app::Cli;
pub use color::Colors;
pub use config_file::Config;
pub use core::Core;
pub use git_theme::GitTheme;
pub use icon::Icons;

// Re-export flag types
pub use flags::{
    icons::IconSeparator, Blocks, Color, ColorOption, DateFlag, Dereference, Display, Flags,
    Header, HyperlinkOption, IconOption, IconTheme, IgnoreGlobs, Indicators, Layout, Literal,
    PermissionFlag, Recursion, SizeFlag, Sorting, ThemeOption, TruncateOwner,
};

// Re-export stream types
pub use stream::{FileEntry, FileStream, StreamError, StreamResult};

// Re-export meta types
pub use meta::{
    AccessControl, Date, FileType, GitFileStatus, INode, Indicator, Links, Meta, Name, Owner,
    Permissions, Size, SymLink,
};

/// Exit codes for SAP operations
#[derive(Debug, PartialEq, Eq, PartialOrd, Copy, Clone)]
pub enum ExitCode {
    /// Operation completed successfully
    OK,
    /// Minor issues encountered (e.g., permission errors on some files)
    MinorIssue,
    /// Major issues encountered (e.g., invalid arguments)
    MajorIssue,
}

impl ExitCode {
    /// Update exit code to a more severe code if needed
    pub fn set_if_greater(&mut self, code: ExitCode) {
        let self_i32 = *self as i32;
        let code_i32 = code as i32;
        if self_i32 < code_i32 {
            *self = code;
        }
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}

/// Macro for printing errors to stderr without panicking on pipe errors
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        {
            use std::io::Write;

            let stderr = std::io::stderr();

            {
                let mut handle = stderr.lock();
                // We can write on stderr, so we simply ignore the error and don't print
                // and stop with success.
                let res = handle.write_all(std::format!("sap: {}\n\n",
                                                        std::format!($($arg)*)).as_bytes());
                if res.is_err() {
                    std::process::exit(0);
                }
            }
        }
    };
}

/// Macro for printing output to stdout without panicking on pipe errors
#[macro_export]
macro_rules! print_output {
    ($($arg:tt)*) => {
        use std::io::Write;

        let stdout = std::io::stdout();

        {
            let mut handle = stdout.lock();
            // We can write on stdout, so we simply ignore the error and don't print
            // and stop with success.
            let res = handle.write_all(std::format!($($arg)*).as_bytes());
            if res.is_err() {
                std::process::exit(0);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_ordering() {
        assert!(ExitCode::OK < ExitCode::MinorIssue);
        assert!(ExitCode::MinorIssue < ExitCode::MajorIssue);
    }

    #[test]
    fn test_exit_code_set_if_greater() {
        let mut code = ExitCode::OK;
        code.set_if_greater(ExitCode::MinorIssue);
        assert_eq!(code, ExitCode::MinorIssue);

        code.set_if_greater(ExitCode::OK);
        assert_eq!(code, ExitCode::MinorIssue);

        code.set_if_greater(ExitCode::MajorIssue);
        assert_eq!(code, ExitCode::MajorIssue);
    }

    #[test]
    fn test_exit_code_to_i32() {
        assert_eq!(i32::from(ExitCode::OK), 0);
        assert_eq!(i32::from(ExitCode::MinorIssue), 1);
        assert_eq!(i32::from(ExitCode::MajorIssue), 2);
    }
}
