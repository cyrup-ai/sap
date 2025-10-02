use crate::{
    color::{ColoredString, Colors, Elem},
    flags::Flags,
};

use std::os::windows::fs::MetadataExt;

#[derive(Debug, Clone)]
pub struct WindowsAttributes {
    pub archive: bool,
    pub readonly: bool,
    pub hidden: bool,
    pub system: bool,
}

pub fn get_attributes(metadata: &std::fs::Metadata) -> WindowsAttributes {
    use windows::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY,
        FILE_ATTRIBUTE_SYSTEM, FILE_FLAGS_AND_ATTRIBUTES,
    };

    let bits = metadata.file_attributes();
    let has_bit = |bit: FILE_FLAGS_AND_ATTRIBUTES| bits & bit.0 == bit.0;

    // https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
    WindowsAttributes {
        archive: has_bit(FILE_ATTRIBUTE_ARCHIVE),
        readonly: has_bit(FILE_ATTRIBUTE_READONLY),
        hidden: has_bit(FILE_ATTRIBUTE_HIDDEN),
        system: has_bit(FILE_ATTRIBUTE_SYSTEM),
    }
}

impl WindowsAttributes {
    pub fn render(&self, colors: &Colors, _flags: &Flags) -> ColoredString {
        let res = [
            match self.archive {
                true => colors.colorize("a", &Elem::Archive),
                false => colors.colorize('-', &Elem::NoAccess),
            },
            match self.readonly {
                true => colors.colorize("r", &Elem::AttributeRead),
                false => colors.colorize('-', &Elem::NoAccess),
            },
            match self.hidden {
                true => colors.colorize("h", &Elem::Hidden),
                false => colors.colorize('-', &Elem::NoAccess),
            },
            match self.system {
                true => colors.colorize("s", &Elem::System),
                false => colors.colorize('-', &Elem::NoAccess),
            },
        ]
        .into_iter()
        .fold(String::with_capacity(4), |mut acc, x| {
            acc.push_str(&x.to_string());
            acc
        });
        ColoredString::new(Colors::default_style(), res)
    }
}
