use crate::color::{ColoredString, Colors, Elem};
use crate::flags::{Flags, SizeFlag};
use std::fs::Metadata;

const KB: u64 = 1024;
const MB: u64 = 1024_u64.pow(2);
const GB: u64 = 1024_u64.pow(3);
const TB: u64 = 1024_u64.pow(4);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unit {
    Byte,
    Kilo,
    Mega,
    Giga,
    Tera,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Size {
    bytes: u64,
}

impl From<&Metadata> for Size {
    fn from(meta: &Metadata) -> Self {
        Self { bytes: meta.len() }
    }
}

impl Size {
    #[allow(dead_code)] // Used by old code path
    pub fn new(bytes: u64) -> Self {
        Self { bytes }
    }

    pub fn get_bytes(&self) -> u64 {
        self.bytes
    }

    fn format_size(&self, number: f64) -> String {
        if number < 10.0 {
            format!("{:.1}", number)
        } else {
            format!("{:.0}", number)
        }
    }

    fn get_unit(&self, flags: &Flags) -> Unit {
        if flags.size == SizeFlag::Bytes {
            return Unit::Byte;
        }

        match self.bytes {
            b if b < KB => Unit::Byte,
            b if b < MB => Unit::Kilo,
            b if b < GB => Unit::Mega,
            b if b < TB => Unit::Giga,
            _ => Unit::Tera,
        }
    }

    pub fn render(
        &self,
        colors: &Colors,
        flags: &Flags,
        val_alignment: Option<usize>,
    ) -> ColoredString {
        let val_content = self.render_value(colors, flags);
        let unit_content = self.render_unit(colors, flags);

        let left_pad = if let Some(align) = val_alignment {
            " ".repeat(align.saturating_sub(val_content.content().len()))
        } else {
            String::new()
        };

        let mut strings: Vec<ColoredString> = vec![
            ColoredString::new(Colors::default_style(), left_pad),
            val_content,
        ];

        // Add thin space between value and unit for better readability
        if flags.size != SizeFlag::Short && flags.size != SizeFlag::Bytes {
            strings.push(ColoredString::new(
                Colors::default_style(),
                "\u{2009}".into(),
            ));
        }

        strings.push(unit_content);

        let res = strings
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("");
        ColoredString::new(Colors::default_style(), res)
    }

    fn paint(&self, colors: &Colors, content: String) -> ColoredString {
        let bytes = self.get_bytes();

        let elem = if bytes >= GB {
            &Elem::FileLarge
        } else if bytes >= MB {
            &Elem::FileMedium
        } else {
            &Elem::FileSmall
        };

        colors.colorize(content, elem)
    }

    pub fn render_value(&self, colors: &Colors, flags: &Flags) -> ColoredString {
        let content = self.value_string(flags);
        self.paint(colors, content)
    }

    pub fn value_string(&self, flags: &Flags) -> String {
        let unit = self.get_unit(flags);

        match unit {
            Unit::Byte => self.bytes.to_string(),
            Unit::Kilo => self.format_size(self.bytes as f64 / KB as f64),
            Unit::Mega => self.format_size(self.bytes as f64 / MB as f64),
            Unit::Giga => self.format_size(self.bytes as f64 / GB as f64),
            Unit::Tera => self.format_size(self.bytes as f64 / TB as f64),
        }
    }

    pub fn render_unit(&self, colors: &Colors, flags: &Flags) -> ColoredString {
        let content = self.unit_string(flags);
        self.paint(colors, content)
    }

    pub fn unit_string(&self, flags: &Flags) -> String {
        let unit = self.get_unit(flags);

        match flags.size {
            SizeFlag::Default => match unit {
                Unit::Byte => String::from("B"),
                Unit::Kilo => String::from("KB"),
                Unit::Mega => String::from("MB"),
                Unit::Giga => String::from("GB"),
                Unit::Tera => String::from("TB"),
            },
            SizeFlag::Short => match unit {
                Unit::Byte => String::from("B"),
                Unit::Kilo => String::from("K"),
                Unit::Mega => String::from("M"),
                Unit::Giga => String::from("G"),
                Unit::Tera => String::from("T"),
            },
            SizeFlag::Bytes => String::new(),
        }
    }
}
