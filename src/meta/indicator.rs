use crate::color::{ColoredString, Colors};
use crate::flags::Flags;
use crate::meta::FileType;

#[derive(Clone, Debug)]
pub struct Indicator(&'static str);

impl From<FileType> for Indicator {
    fn from(file_type: FileType) -> Self {
        let res = match file_type {
            FileType::Directory { .. } => "󰉋 ", // Nerd Font folder icon
            FileType::File { exec: true, .. } => "󰨊 ", // Nerd Font executable icon
            FileType::Pipe => "󰈲 ",             // Nerd Font pipe icon
            FileType::Socket => "󰆨 ",           // Nerd Font socket icon
            FileType::SymLink { .. } => "󰌹 ",   // Nerd Font link icon
            _ => "",
        };

        Indicator(res)
    }
}

impl Indicator {
    pub fn render(&self, flags: &Flags) -> ColoredString {
        if flags.display_indicators.0 {
            ColoredString::new(Colors::default_style(), self.0.to_string())
        } else {
            ColoredString::new(Colors::default_style(), "".into())
        }
    }
}
