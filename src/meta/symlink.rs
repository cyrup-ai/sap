use crate::color::{ColoredString, Colors, Elem};
use crate::flags::Flags;
use std::fs::read_link;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SymLink {
    target: Option<PathBuf>,
    valid: bool,
}

impl From<&Path> for SymLink {
    fn from(path: &Path) -> Self {
        match read_link(path) {
            Ok(target) => {
                let valid = if target.is_absolute() {
                    target.exists()
                } else if let Some(parent) = path.parent() {
                    parent.join(&target).exists()
                } else {
                    // Symlink in root directory with relative target
                    target.exists()
                };

                Self {
                    target: Some(target),
                    valid,
                }
            }
            Err(_) => Self {
                target: None,
                valid: false,
            },
        }
    }
}

impl SymLink {
    pub fn symlink_string(&self) -> Option<String> {
        self.target
            .as_ref()
            .and_then(|target| target.to_str())
            .map(|s| s.to_string())
    }

    pub fn render(&self, colors: &Colors, flag: &Flags) -> ColoredString {
        match self.symlink_string() {
            Some(target_string) => {
                let elem = if self.valid {
                    &Elem::SymLink
                } else {
                    &Elem::MissingSymLinkTarget
                };

                let arrow = ColoredString::new(
                    Colors::default_style(),
                    format!(" {} ", flag.symlink_arrow),
                );
                let target = colors.colorize(target_string, elem);

                ColoredString::new(Colors::default_style(), format!("{}{}", arrow, target))
            }
            None => ColoredString::new(Colors::default_style(), String::new()),
        }
    }
}
