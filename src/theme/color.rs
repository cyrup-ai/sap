//! This module provides methods to create theme from files and operations related to
//! this.
use console::Term;
use crossterm::style::Color;
use serde::{de::IntoDeserializer, Deserialize};
use std::fmt;

// Custom color deserialize
fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct ColorVisitor;
    impl<'de> serde::de::Visitor<'de> for ColorVisitor {
        type Value = Color;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str(
                    "`black`, `blue`, `dark_blue`, `cyan`, `dark_cyan`, `green`, `dark_green`, `grey`, `dark_grey`, `magenta`, `dark_magenta`, `red`, `dark_red`, `white`, `yellow`, `dark_yellow`, `u8`, or `3 u8 array`",
                )
        }

        fn visit_str<E>(self, value: &str) -> Result<Color, E>
        where
            E: serde::de::Error,
        {
            Color::deserialize(value.into_deserializer())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Color, E>
        where
            E: serde::de::Error,
        {
            if value > 255 {
                return Err(E::invalid_value(
                    serde::de::Unexpected::Unsigned(value),
                    &self,
                ));
            }
            Ok(Color::AnsiValue(value as u8))
        }

        fn visit_seq<M>(self, mut seq: M) -> Result<Color, M::Error>
        where
            M: serde::de::SeqAccess<'de>,
        {
            let mut values = Vec::new();
            if let Some(size) = seq.size_hint()
                && size != 3 {
                    return Err(serde::de::Error::invalid_length(
                        size,
                        &"a list of size 3(RGB)",
                    ));
                }
            loop {
                match seq.next_element::<u8>() {
                    Ok(Some(x)) => {
                        values.push(x);
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            // recheck as size_hint sometimes not working
            if values.len() != 3 {
                return Err(serde::de::Error::invalid_length(
                    values.len(),
                    &"a list of size 3(RGB)",
                ));
            }
            Ok(Color::from((values[0], values[1], values[2])))
        }
    }

    deserializer.deserialize_any(ColorVisitor)
}

/// A struct holding the theme configuration
/// Color table: https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ColorTheme {
    #[serde(deserialize_with = "deserialize_color")]
    pub user: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub group: Color,
    pub permission: Permission,
    pub attributes: Attributes,
    pub date: Date,
    pub size: Size,
    pub inode: INode,
    #[serde(deserialize_with = "deserialize_color")]
    pub tree_edge: Color,
    pub links: Links,
    pub git_status: GitStatus,

    #[serde(skip)]
    pub file_type: FileType,
    
    #[serde(skip)]
    pub render_rules: Vec<super::render::RenderRule>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Permission {
    #[serde(deserialize_with = "deserialize_color")]
    pub read: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub write: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub exec: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub exec_sticky: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub no_access: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub octal: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub acl: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub context: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Attributes {
    #[serde(deserialize_with = "deserialize_color")]
    pub archive: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub read: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub hidden: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub system: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FileType {
    pub file: File,
    pub dir: Dir,
    #[serde(deserialize_with = "deserialize_color")]
    pub pipe: Color,
    pub symlink: Symlink,
    #[serde(deserialize_with = "deserialize_color")]
    pub block_device: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub char_device: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub socket: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub special: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct File {
    #[serde(deserialize_with = "deserialize_color")]
    pub exec_uid: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub uid_no_exec: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub exec_no_uid: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub no_exec_no_uid: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Dir {
    #[serde(deserialize_with = "deserialize_color")]
    pub uid: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub no_uid: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Symlink {
    #[serde(deserialize_with = "deserialize_color")]
    pub default: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub broken: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub missing_target: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Date {
    #[serde(deserialize_with = "deserialize_color")]
    pub hour_old: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub day_old: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub older: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Size {
    #[serde(deserialize_with = "deserialize_color")]
    pub none: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub small: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub medium: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub large: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct INode {
    #[serde(deserialize_with = "deserialize_color")]
    pub valid: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub invalid: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Links {
    #[serde(deserialize_with = "deserialize_color")]
    pub valid: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub invalid: Color,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct GitStatus {
    #[serde(deserialize_with = "deserialize_color")]
    pub default: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub unmodified: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub ignored: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub new_in_index: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub new_in_workdir: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub typechange: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub deleted: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub renamed: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub modified: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub conflicted: Color,
}

impl Default for Permission {
    fn default() -> Self {
        Permission {
            read: Color::DarkGreen,
            write: Color::DarkYellow,
            exec: Color::DarkRed,
            exec_sticky: Color::AnsiValue(5),
            no_access: Color::AnsiValue(245), // Grey
            octal: Color::AnsiValue(6),
            acl: Color::DarkCyan,
            context: Color::Cyan,
        }
    }
}
impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            archive: Color::DarkGreen,
            read: Color::DarkYellow,
            hidden: Color::AnsiValue(13), // Pink,
            system: Color::AnsiValue(13), // Pink,
        }
    }
}
impl Default for FileType {
    fn default() -> Self {
        FileType {
            file: File::default(),
            dir: Dir::default(),
            symlink: Symlink::default(),
            pipe: Color::AnsiValue(44),         // DarkTurquoise
            block_device: Color::AnsiValue(44), // DarkTurquoise
            char_device: Color::AnsiValue(172), // Orange3
            socket: Color::AnsiValue(44),       // DarkTurquoise
            special: Color::AnsiValue(44),      // DarkTurquoise
        }
    }
}

impl FileType {
    pub fn cyrup_theme() -> Self {
        FileType {
            file: File {
                exec_uid: Color::Rgb { r: 0, g: 255, b: 117 },      // CYRUP bright green #00ff75
                uid_no_exec: Color::Rgb { r: 249, g: 249, b: 249 }, // CYRUP foreground #F9F9F9
                exec_no_uid: Color::Rgb { r: 0, g: 255, b: 117 },   // CYRUP bright green #00ff75
                no_exec_no_uid: Color::Rgb { r: 249, g: 249, b: 249 }, // CYRUP foreground #F9F9F9
            },
            dir: Dir {
                uid: Color::Rgb { r: 194, g: 97, b: 195 },     // CYRUP accent #c261c3
                no_uid: Color::Rgb { r: 179, g: 172, b: 255 }, // CYRUP hint #b3acff
            },
            symlink: Symlink {
                default: Color::Rgb { r: 125, g: 211, b: 252 },     // CYRUP bright cyan #7dd3fc
                broken: Color::Rgb { r: 224, g: 108, b: 117 },      // CYRUP red #e06c75
                missing_target: Color::Rgb { r: 127, g: 127, b: 127 }, // CYRUP muted #7f7f7f
            },
            pipe: Color::Rgb { r: 255, g: 177, b: 0 },        // CYRUP yellow #ffb100
            block_device: Color::Rgb { r: 255, g: 177, b: 0 }, // CYRUP yellow #ffb100
            char_device: Color::Rgb { r: 255, g: 0, b: 158 },  // CYRUP bright magenta #ff009e
            socket: Color::Rgb { r: 194, g: 97, b: 195 },     // CYRUP accent #c261c3
            special: Color::Rgb { r: 179, g: 172, b: 255 },   // CYRUP hint #b3acff
        }
    }
}
impl Default for File {
    fn default() -> Self {
        File {
            exec_uid: Color::AnsiValue(40),        // Green3
            uid_no_exec: Color::AnsiValue(184),    // Yellow3
            exec_no_uid: Color::AnsiValue(40),     // Green3
            no_exec_no_uid: Color::AnsiValue(184), // Yellow3
        }
    }
}
impl Default for Dir {
    fn default() -> Self {
        Dir {
            uid: Color::AnsiValue(33),    // DodgerBlue1
            no_uid: Color::AnsiValue(33), // DodgerBlue1
        }
    }
}
impl Default for Symlink {
    fn default() -> Self {
        Symlink {
            default: Color::AnsiValue(44),         // DarkTurquoise
            broken: Color::AnsiValue(124),         // Red3
            missing_target: Color::AnsiValue(124), // Red3
        }
    }
}
impl Default for Date {
    fn default() -> Self {
        Date {
            hour_old: Color::AnsiValue(40), // Green3
            day_old: Color::AnsiValue(42),  // SpringGreen2
            older: Color::AnsiValue(36),    // DarkCyan
        }
    }
}

impl Date {
    pub fn cyrup_theme() -> Self {
        Date {
            hour_old: Color::Rgb { r: 0, g: 255, b: 117 },   // CYRUP bright green #00ff75
            day_old: Color::Rgb { r: 255, g: 177, b: 0 },    // CYRUP yellow #ffb100  
            older: Color::Rgb { r: 127, g: 127, b: 127 },    // CYRUP muted #7f7f7f
        }
    }
}
impl Default for Size {
    fn default() -> Self {
        Size {
            none: Color::AnsiValue(245),   // Grey
            small: Color::AnsiValue(229),  // Wheat1
            medium: Color::AnsiValue(216), // LightSalmon1
            large: Color::AnsiValue(172),  // Orange3
        }
    }
}

impl Size {
    pub fn cyrup_theme() -> Self {
        Size {
            none: Color::Rgb { r: 127, g: 127, b: 127 },     // CYRUP muted #7f7f7f
            small: Color::Rgb { r: 249, g: 249, b: 249 },    // CYRUP foreground #F9F9F9
            medium: Color::Rgb { r: 255, g: 177, b: 0 },     // CYRUP yellow #ffb100
            large: Color::Rgb { r: 255, g: 0, b: 158 },      // CYRUP bright magenta #ff009e
        }
    }
}
impl Default for INode {
    fn default() -> Self {
        INode {
            valid: Color::AnsiValue(13),    // Pink
            invalid: Color::AnsiValue(245), // Grey
        }
    }
}
impl Default for Links {
    fn default() -> Self {
        Links {
            valid: Color::AnsiValue(13),    // Pink
            invalid: Color::AnsiValue(245), // Grey
        }
    }
}

impl Default for GitStatus {
    fn default() -> Self {
        GitStatus {
            default: Color::AnsiValue(245),    // Grey
            unmodified: Color::AnsiValue(245), // Grey
            ignored: Color::AnsiValue(245),    // Grey
            new_in_index: Color::DarkGreen,
            new_in_workdir: Color::DarkGreen,
            typechange: Color::DarkYellow,
            deleted: Color::DarkRed,
            renamed: Color::DarkGreen,
            modified: Color::DarkYellow,
            conflicted: Color::DarkRed,
        }
    }
}

fn detect_terminal_theme() -> Option<ColorTheme> {
    let term = Term::stdout();
    
    // Method 1: Try environment variable hints first (most reliable)
    if let Some(theme) = check_terminal_env_hints() {
        return Some(theme);
    }
    
    // Method 2: Check terminal capabilities
    if term.features().colors_supported() {
        // For color-supporting terminals, use environment-based detection
        check_terminal_specific_hints()
    } else {
        None
    }
}

fn check_terminal_env_hints() -> Option<ColorTheme> {
    // Check COLORFGBG environment variable (used by many terminals)
    if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
        // Format is typically "foreground;background"
        if let Some(bg) = colorfgbg.split(';').nth(1) {
            match bg.parse::<u8>() {
                Ok(bg_color) if bg_color > 7 => return Some(ColorTheme::default_light()),
                Ok(_) => return Some(ColorTheme::default_dark()),
                Err(_) => {}
            }
        }
    }
    None
}

fn check_terminal_specific_hints() -> Option<ColorTheme> {
    // Check terminal-specific environment variables
    match std::env::var("TERM_PROGRAM").ok().as_deref() {
        Some("Apple_Terminal") => {
            // macOS Terminal.app typically defaults to light theme
            Some(ColorTheme::default_light())
        }
        Some("iTerm.app") => {
            // iTerm2 often uses dark themes but varies
            None // Let it fall back to default
        }
        _ => None,
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        detect_terminal_theme().unwrap_or_else(Self::default_dark)
    }
}

impl ColorTheme {
    pub fn default_dark() -> Self {
        ColorTheme {
            user: Color::Rgb { r: 194, g: 97, b: 195 },  // CYRUP accent #c261c3
            group: Color::Rgb { r: 179, g: 172, b: 255 }, // CYRUP hint #b3acff
            permission: Permission::default(),
            attributes: Attributes::default(),
            file_type: FileType::cyrup_theme(),
            date: Date::cyrup_theme(),
            size: Size::cyrup_theme(),
            inode: INode::default(),
            links: Links::default(),
            tree_edge: Color::Rgb { r: 127, g: 127, b: 127 }, // CYRUP muted grey #7f7f7f
            git_status: Default::default(),
            render_rules: Self::default_render_rules(),
        }
    }

    pub fn default_light() -> Self {
        ColorTheme {
            user: Color::Rgb { r: 138, g: 43, b: 139 },   // Darker CYRUP accent for light bg
            group: Color::Rgb { r: 98, g: 86, b: 176 },   // Darker CYRUP hint for light bg  
            permission: Permission::default(),
            attributes: Attributes::default(),
            file_type: FileType::cyrup_theme(),
            date: Date::cyrup_theme(),
            size: Size::cyrup_theme(),
            inode: INode::default(),
            links: Links::default(),
            tree_edge: Color::Rgb { r: 100, g: 100, b: 100 }, // Darker grey for light bg
            git_status: Default::default(),
            render_rules: Self::default_render_rules(),
        }
    }
    
    fn default_render_rules() -> Vec<super::render::RenderRule> {
        use super::render::*;
        use crate::git::GitStatus;
        use crate::meta::FileType;
        
        vec![
            // Modified directories - show with bright colors
            RenderRule {
                matchers: RuleMatchers {
                    file_types: Some(vec![FileType::Directory { uid: false }]),
                    git_statuses: Some(vec![GitStatus::Modified]),
                    ..Default::default()
                },
                display: DisplaySettings {
                    icon_color: Some(ExtendedColor::Rgba { r: 255, g: 177, b: 0, a: 1.0 }),
                    name_color: Some(ExtendedColor::Rgba { r: 255, g: 177, b: 0, a: 1.0 }),
                    ..Default::default()
                },
            },
            // Normal directories - muted
            RenderRule {
                matchers: RuleMatchers {
                    file_types: Some(vec![FileType::Directory { uid: false }]),
                    ..Default::default()
                },
                display: DisplaySettings {
                    icon_color: Some(ExtendedColor::Rgba { r: 179, g: 172, b: 255, a: 0.75 }),
                    name_color: Some(ExtendedColor::Rgba { r: 179, g: 172, b: 255, a: 0.75 }),
                    ..Default::default()
                },
            },
            // Modified files - bright with no special icon (icon comes from file type)
            RenderRule {
                matchers: RuleMatchers {
                    git_statuses: Some(vec![GitStatus::Modified, GitStatus::NewInWorkdir]),
                    ..Default::default()
                },
                display: DisplaySettings {
                    icon_color: Some(ExtendedColor::Rgba { r: 249, g: 249, b: 249, a: 1.0 }),
                    name_color: Some(ExtendedColor::Rgba { r: 249, g: 249, b: 249, a: 1.0 }),
                    ..Default::default()
                },
            },
            // All files - default muted
            RenderRule {
                matchers: RuleMatchers::default(), // No conditions = matches all
                display: DisplaySettings {
                    name_color: Some(ExtendedColor::Rgba { r: 249, g: 249, b: 249, a: 0.75 }),
                    ..Default::default()
                },
            },
        ]
    }
}
