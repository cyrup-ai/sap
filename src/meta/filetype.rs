use crate::color::{ColoredString, Colors, Elem};
use std::fs::Metadata;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(windows, allow(dead_code))]
pub enum FileType {
    BlockDevice,
    CharDevice,
    Directory { uid: bool },
    File { uid: bool, exec: bool },
    SymLink { is_dir: bool },
    Pipe,
    Socket,
    Special,
}

impl FileType {
    #[cfg(windows)]
    const EXECUTABLE_EXTENSIONS: &'static [&'static str] = &["exe", "msi", "bat", "ps1"];

    #[cfg(unix)]
    pub fn new(
        meta: &Metadata,
        symlink_meta: Option<&Metadata>,
        permissions: &crate::meta::Permissions,
    ) -> Self {
        use std::os::unix::fs::FileTypeExt;

        let file_type = meta.file_type();

        if file_type.is_file() {
            FileType::File {
                exec: permissions.is_executable(),
                uid: permissions.setuid,
            }
        } else if file_type.is_dir() {
            FileType::Directory {
                uid: permissions.setuid,
            }
        } else if file_type.is_fifo() {
            FileType::Pipe
        } else if file_type.is_symlink() {
            FileType::SymLink {
                // if broken, defaults to false
                is_dir: symlink_meta.map(|m| m.is_dir()).unwrap_or_default(),
            }
        } else if file_type.is_char_device() {
            FileType::CharDevice
        } else if file_type.is_block_device() {
            FileType::BlockDevice
        } else if file_type.is_socket() {
            FileType::Socket
        } else {
            FileType::Special
        }
    }

    #[cfg(windows)]
    pub fn new(meta: &Metadata, symlink_meta: Option<&Metadata>, path: &std::path::Path) -> Self {
        let file_type = meta.file_type();

        if file_type.is_file() {
            let exec = path
                .extension()
                .map(|ext| {
                    Self::EXECUTABLE_EXTENSIONS
                        .iter()
                        .map(std::ffi::OsStr::new)
                        .any(|exec_ext| ext == exec_ext)
                })
                .unwrap_or(false);
            FileType::File { exec, uid: false }
        } else if file_type.is_dir() {
            FileType::Directory { uid: false }
        } else if file_type.is_symlink() {
            FileType::SymLink {
                // if broken, defaults to false
                is_dir: symlink_meta.map(|m| m.is_dir()).unwrap_or_default(),
            }
        } else {
            FileType::Special
        }
    }

    pub fn is_dirlike(self) -> bool {
        matches!(
            self,
            FileType::Directory { .. } | FileType::SymLink { is_dir: true }
        )
    }
}

impl FileType {
    pub fn render(self, colors: &Colors) -> ColoredString {
        match self {
            FileType::File { exec, .. } => colors.colorize('󰈔', &Elem::File { exec, uid: false }),
            FileType::Directory { .. } => colors.colorize('󰉋', &Elem::Dir { uid: false }),
            FileType::Pipe => colors.colorize('󰟦', &Elem::Pipe),
            FileType::SymLink { .. } => colors.colorize('󰌹', &Elem::SymLink),
            FileType::BlockDevice => colors.colorize('󰋊', &Elem::BlockDevice),
            FileType::CharDevice => colors.colorize('󱓞', &Elem::CharDevice),
            FileType::Socket => colors.colorize('󰛳', &Elem::Socket),
            FileType::Special => colors.colorize('󰋗', &Elem::Special),
        }
    }
}
