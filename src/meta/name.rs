use crate::color::{ColoredString, Colors};
use crate::flags::HyperlinkOption;
use crate::icon::Icons;
use crate::meta::filetype::FileType;
use crate::meta::GitFileStatus;
use crate::print_error;
use url::Url;
use std::cmp::{Ordering, PartialOrd};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Display options for file name rendering
#[derive(Debug)]
pub enum DisplayOption<'a> {
    /// Show only the file name
    FileName,
    /// Show relative path from base
    Relative { base_path: &'a Path },
}

/// Represents a file or directory name with associated metadata
#[derive(Clone, Debug, Eq)]
pub struct Name {
    pub name: String,
    path: PathBuf,
    extension: Option<String>,
    file_type: FileType,
}

impl Name {
    /// Creates a new Name instance from a path and file type
    pub fn new(path: &Path, file_type: FileType) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        Self {
            name,
            path: PathBuf::from(path),
            extension,
            file_type,
        }
    }

    /// Returns the file name as a string slice
    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or(&self.name)
    }

    /// Calculates the relative path from a base path
    fn relative_path<T: AsRef<Path>>(&self, base_path: T) -> PathBuf {
        let base_path = base_path.as_ref();

        if self.path == base_path {
            return PathBuf::from(AsRef::<Path>::as_ref(&Component::CurDir));
        }

        // Find common prefix between paths
        let shared_components: PathBuf = self
            .path
            .components()
            .zip(base_path.components())
            .take_while(|(a, b)| a == b)
            .map(|(component, _)| component)
            .collect();

        // Build relative path with parent directory components
        let parent_components = base_path
            .strip_prefix(&shared_components)
            .unwrap_or(base_path)
            .components()
            .map(|_| Component::ParentDir);

        let target_components = self
            .path
            .strip_prefix(&shared_components)
            .unwrap_or(&self.path)
            .components();

        parent_components.chain(target_components).collect()
    }

    /// Escapes special characters in file names for shell safety
    fn escape(&self, string: &str, literal: bool) -> String {
        if literal {
            return self.escape_control_chars(string);
        }

        let escaped = if string.contains('\\') || string.contains('"') {
            format!("'{}'", string.replace('\'', "\'\\\'\'"))
        } else if string.contains('\'') {
            format!("\"{}\"", string)
        } else if string.contains(' ') || string.contains('$') {
            format!("'{}'", string)
        } else {
            string.to_string()
        };

        self.escape_control_chars(&escaped)
    }

    /// Escapes control characters while preserving UTF-8
    fn escape_control_chars(&self, string: &str) -> String {
        string
            .chars()
            .map(|c| {
                if c.is_control() || c == '\x7f' {
                    c.escape_default().to_string()
                } else {
                    c.to_string()
                }
            })
            .collect()
    }

    /// Wraps the name in terminal hyperlink escape sequences
    fn hyperlink(&self, name: String, hyperlink: HyperlinkOption) -> String {
        match hyperlink {
            HyperlinkOption::Always => {
                // HyperlinkOption::Auto gets converted to None or Always in core.rs based on tty_available
                match std::fs::canonicalize(&self.path) {
                    Ok(canonical_path) => {
                        match Url::from_file_path(canonical_path) {
                            Ok(url) => {
                                // OSC 8 hyperlink format
                                // https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
                                format!("\x1B]8;;{url}\x1B\x5C{name}\x1B]8;;\x1B\x5C")
                            }
                            Err(_) => {
                                print_error!("{}: unable to form url.", name);
                                name
                            }
                        }
                    }
                    Err(err) => {
                        // Broken symlinks are expected, don't report as error
                        if err.kind() != std::io::ErrorKind::NotFound {
                            print_error!("{}: {}", name, err);
                        }
                        name
                    }
                }
            }
            _ => name,
        }
    }

    /// Renders the name with colors, icons, and formatting
    pub fn render(
        &self,
        colors: &Colors,
        icons: &Icons,
        display_option: &DisplayOption,
        hyperlink: HyperlinkOption,
        literal: bool,
        git_status: Option<&GitFileStatus>,
    ) -> ColoredString {
        let icon = icons.get(self);

        let display_name = match display_option {
            DisplayOption::FileName => self.escape(self.file_name(), literal),
            DisplayOption::Relative { base_path } => {
                self.escape(&self.relative_path(base_path).to_string_lossy(), literal)
            }
        };

        let hyperlinked_name = self.hyperlink(display_name, hyperlink);
        
        // Use the new render decision system
        let decision = colors.render_decision(
            &self.file_type,
            self.extension.as_deref(),
            git_status,
            false, // has_error - future feature
            false, // draw_attention - future feature
        );
        
        // Apply the decision
        let colored_icon = if !icon.is_empty() {
            if !decision.icon.is_empty() {
                // Use icon from rule if specified
                decision.icon_style.apply(&decision.icon).to_string()
            } else {
                // Use default icon with rule color
                decision.icon_style.apply(&icon).to_string()
            }
        } else {
            String::new()
        };
        
        let colored_name = decision.name_style.apply(&hyperlinked_name).to_string();
        
        // Combine colored icon and colored name
        ColoredString::new(Colors::default_style(), format!("{colored_icon}{colored_name}"))
    }

    /// Returns the file extension if present
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// Returns the file type
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
    }
}
