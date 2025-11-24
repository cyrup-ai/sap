use crate::flags::{IconOption, IconTheme as FlagTheme};
use crate::meta::{FileType, Name};
use crate::theme::{icon::IconTheme, Theme};

fn _convert_unicode_escapes(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                // Potential start of an escape sequence
                if let Some(&next_ch) = chars.peek()
                    && next_ch == 'u' {
                        chars.next(); // consume 'u'
                                      // Collect hex digits after \u
                        let mut hex_digits = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() {
                                hex_digits.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if !hex_digits.is_empty() {
                            // Parse hex string to a number and then to a char
                            if let Ok(codepoint) = u32::from_str_radix(&hex_digits, 16)
                                && let Some(unicode_char) = char::from_u32(codepoint) {
                                    output.push(unicode_char);
                                    continue; // successfully replaced, skip to next input char
                                }
                            // If parsing failed or codepoint is invalid, fall through to output literal
                        } else {
                            // No hex digits after "\u", treat "\u" as literal
                        }
                        // Output the literal "\u{hex_digits}" since it was not a valid escape
                        output.push('\\');
                        output.push('u');
                        output.push_str(&hex_digits);
                        continue;
                    }
                // Not a "\u" sequence, just a lone backslash or "\<other>"
                output.push(ch);
            } else {
                // Regular character, just copy
                output.push(ch);
            }
        }
        output
    }

pub struct Icons {
    icon_separator: String,
    theme: Option<IconTheme>,
}

// In order to add a new icon, write the unicode value like "\ue5fb" then
// run the command below in vim:
//
// s#\\u[0-9a-f]*#\=eval('"'.submatch(0).'"')#
impl Icons {
    pub fn new(tty: bool, when: IconOption, theme: FlagTheme, icon_separator: String) -> Self {
        let icon_theme = match (tty, when, theme) {
            (_, IconOption::Never, _) | (false, IconOption::Auto, _) => None,
            (_, _, FlagTheme::Fancy) => {
                if let Ok(t) = Theme::from_path::<IconTheme>("icons") {
                    Some(t)
                } else {
                    Some(IconTheme::default())
                }
            }
            (_, _, FlagTheme::Unicode) => Some(IconTheme::unicode()),
        };

        Self {
            icon_separator,
            theme: icon_theme,
        }
    }

    pub fn get(&self, name: &Name) -> String {
        match &self.theme {
            None => String::new(),
            Some(t) => {
                // Check file types
                let file_type: FileType = name.file_type();
                let icon = match file_type {
                    FileType::SymLink { is_dir: true } => &t.filetype.symlink_dir,
                    FileType::SymLink { is_dir: false } => &t.filetype.symlink_file,
                    FileType::Socket => &t.filetype.socket,
                    FileType::Pipe => &t.filetype.pipe,
                    FileType::CharDevice => &t.filetype.device_char,
                    FileType::BlockDevice => &t.filetype.device_block,
                    FileType::Special => &t.filetype.special,
                    _ => {
                        if let Some(icon) = t.name.get(name.file_name().to_lowercase().as_str()) {
                            icon
                        } else if let Some(icon) = name
                            .extension()
                            .and_then(|ext| t.extension.get(ext.to_lowercase().as_str()))
                        {
                            icon
                        } else {
                            match file_type {
                                FileType::Directory { .. } => &t.filetype.dir,
                                // If a file has no extension and is executable, show an icon.
                                // Except for Windows, it marks everything as an executable.
                                #[cfg(not(windows))]
                                FileType::File { exec: true, .. } => &t.filetype.executable,
                                _ => &t.filetype.file,
                            }
                        }
                    }
                };

                format!("{}{}", icon, self.icon_separator)
            }
        }
    }
}
