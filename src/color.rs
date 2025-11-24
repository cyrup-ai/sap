use crossterm::style::Color;
use crossterm::style::{Attribute, ContentStyle, StyledContent, Stylize};
use lscolors::{Indicator, LsColors};
use std::path::Path;

pub use crate::flags::color::ThemeOption;
use crate::git::GitStatus;
use crate::meta::{FileType, GitFileStatus};

use crate::theme::{color::ColorTheme, Theme};
use crate::theme::render::{ErrorStatus, Highlight};

#[allow(dead_code)]
#[derive(Hash, Debug, Eq, PartialEq, Clone)]
pub enum Elem {
    /// Node type
    File {
        exec: bool,
        uid: bool,
    },
    SymLink,
    BrokenSymLink,
    MissingSymLinkTarget,
    Dir {
        uid: bool,
    },
    Pipe,
    BlockDevice,
    CharDevice,
    Socket,
    Special,

    /// Permission
    Read,
    Write,
    Exec,
    ExecSticky,
    NoAccess,
    Octal,
    Acl,
    Context,

    /// Attributes
    Archive,
    AttributeRead,
    Hidden,
    System,

    /// Last Time Modified
    DayOld,
    HourOld,
    Older,

    /// User / Group Name
    User,
    Group,

    /// File Size
    NonFile,
    FileLarge,
    FileMedium,
    FileSmall,

    /// INode
    INode {
        valid: bool,
    },

    Links {
        valid: bool,
    },

    TreeEdge,

    GitStatus {
        status: GitStatus,
    },
}

impl Elem {
    fn has_suid(&self) -> bool {
        matches!(self, Elem::Dir { uid: true } | Elem::File { uid: true, .. })
    }

    pub fn get_color(&self, theme: &ColorTheme) -> Color {
        match self {
            Elem::File {
                exec: true,
                uid: true,
            } => theme.file_type.file.exec_uid,
            Elem::File {
                exec: false,
                uid: true,
            } => theme.file_type.file.uid_no_exec,
            Elem::File {
                exec: true,
                uid: false,
            } => theme.file_type.file.exec_no_uid,
            Elem::File {
                exec: false,
                uid: false,
            } => theme.file_type.file.no_exec_no_uid,
            Elem::SymLink => theme.file_type.symlink.default,
            Elem::BrokenSymLink => theme.file_type.symlink.broken,
            Elem::MissingSymLinkTarget => theme.file_type.symlink.missing_target,
            Elem::Dir { uid: true } => theme.file_type.dir.uid,
            Elem::Dir { uid: false } => theme.file_type.dir.no_uid,
            Elem::Pipe => theme.file_type.pipe,
            Elem::BlockDevice => theme.file_type.block_device,
            Elem::CharDevice => theme.file_type.char_device,
            Elem::Socket => theme.file_type.socket,
            Elem::Special => theme.file_type.special,

            Elem::Read => theme.permission.read,
            Elem::Write => theme.permission.write,
            Elem::Exec => theme.permission.exec,
            Elem::ExecSticky => theme.permission.exec_sticky,
            Elem::NoAccess => theme.permission.no_access,
            Elem::Octal => theme.permission.octal,
            Elem::Acl => theme.permission.acl,
            Elem::Context => theme.permission.context,

            Elem::Archive => theme.attributes.archive,
            Elem::AttributeRead => theme.attributes.read,
            Elem::Hidden => theme.attributes.hidden,
            Elem::System => theme.attributes.system,

            Elem::DayOld => theme.date.day_old,
            Elem::HourOld => theme.date.hour_old,
            Elem::Older => theme.date.older,

            Elem::User => theme.user,
            Elem::Group => theme.group,
            Elem::NonFile => theme.size.none,
            Elem::FileLarge => theme.size.large,
            Elem::FileMedium => theme.size.medium,
            Elem::FileSmall => theme.size.small,
            Elem::INode { valid: true } => theme.inode.valid,
            Elem::INode { valid: false } => theme.inode.invalid,
            Elem::TreeEdge => theme.tree_edge,
            Elem::Links { valid: false } => theme.links.invalid,
            Elem::Links { valid: true } => theme.links.valid,

            Elem::GitStatus {
                status: GitStatus::Default,
            } => theme.git_status.default,
            Elem::GitStatus {
                status: GitStatus::Unmodified,
            } => theme.git_status.unmodified,
            Elem::GitStatus {
                status: GitStatus::Ignored,
            } => theme.git_status.ignored,
            Elem::GitStatus {
                status: GitStatus::NewInIndex,
            } => theme.git_status.new_in_index,
            Elem::GitStatus {
                status: GitStatus::NewInWorkdir,
            } => theme.git_status.new_in_workdir,
            Elem::GitStatus {
                status: GitStatus::Typechange,
            } => theme.git_status.typechange,
            Elem::GitStatus {
                status: GitStatus::Deleted,
            } => theme.git_status.deleted,
            Elem::GitStatus {
                status: GitStatus::Renamed,
            } => theme.git_status.renamed,
            Elem::GitStatus {
                status: GitStatus::Modified,
            } => theme.git_status.modified,
            Elem::GitStatus {
                status: GitStatus::Conflicted,
            } => theme.git_status.conflicted,
            Elem::GitStatus {
                status: GitStatus::GitConflicted,
            } => theme.git_status.conflicted,
        }
    }
}

pub type ColoredString = StyledContent<String>;

pub struct Colors {
    theme: Option<ColorTheme>,
    lscolors: Option<LsColors>,
}

fn load_legacy_theme_with_feedback(file: &str) -> ColorTheme {
    let theme_path = Path::new("themes").join(file);
    
    let path_str = match theme_path.to_str() {
        Some(s) => s,
        None => {
            eprintln!("Warning: Invalid theme path 'themes/{}' (non-UTF8)", file);
            return ColorTheme::default_dark();
        }
    };
    
    match Theme::from_path::<ColorTheme>(path_str) {
        Ok(theme) => {
            eprintln!("Warning: Using deprecated theme directory. Please migrate to colors.yaml");
            theme
        }
        Err(e) => {
            eprintln!("Error loading theme from '{}': {}", path_str, e);
            eprintln!("Falling back to default dark theme");
            ColorTheme::default_dark()
        }
    }
}

impl Colors {
    pub fn new(t: ThemeOption) -> Self {
        let theme = match t {
            ThemeOption::NoColor => None,
            ThemeOption::Default | ThemeOption::NoLscolors => Some(Theme::default().color),
            ThemeOption::Custom => {
                // Handle the case where the path cannot be converted to a string
                let path_str = Path::new("colors").to_str().unwrap_or_else(|| {
                    eprintln!("Warning: Path 'colors' contains invalid UTF-8 characters");
                    "colors"
                });
                Some(Theme::from_path::<ColorTheme>(path_str).unwrap_or_default())
            },
            ThemeOption::CustomLegacy(ref file) => {
                Some(load_legacy_theme_with_feedback(file))
            }
        };
        let lscolors = match t {
            ThemeOption::Default | ThemeOption::Custom | ThemeOption::CustomLegacy(_) => {
                Some(LsColors::from_env().unwrap_or_default())
            }
            _ => None,
        };

        Self { theme, lscolors }
    }

    pub fn colorize<S: Into<String>>(&self, input: S, elem: &Elem) -> ColoredString {
        self.style(elem).apply(input.into())
    }

    pub fn default_style() -> ContentStyle {
        ContentStyle::default()
    }

    fn style(&self, elem: &Elem) -> ContentStyle {
        match &self.lscolors {
            Some(lscolors) => match self.get_indicator_from_elem(elem) {
                Some(style) => {
                    let style = lscolors.style_for_indicator(style);
                    style.map(to_content_style).unwrap_or_default()
                }
                None => self.style_default(elem),
            },
            None => self.style_default(elem),
        }
    }

    fn style_default(&self, elem: &Elem) -> ContentStyle {
        if let Some(t) = &self.theme {
            let style_fg = ContentStyle::default().with(elem.get_color(t));
            if elem.has_suid() {
                style_fg.on(Color::AnsiValue(124)) // Red3
            } else {
                style_fg
            }
        } else {
            ContentStyle::default()
        }
    }

    fn get_indicator_from_elem(&self, elem: &Elem) -> Option<Indicator> {
        let indicator_string = match elem {
            Elem::File { exec, uid } => match (exec, uid) {
                (_, true) => None,
                (true, false) => Some("ex"),
                (false, false) => Some("fi"),
            },
            Elem::Dir { uid } => {
                if *uid {
                    None
                } else {
                    Some("di")
                }
            }
            Elem::SymLink => Some("ln"),
            Elem::Pipe => Some("pi"),
            Elem::Socket => Some("so"),
            Elem::BlockDevice => Some("bd"),
            Elem::CharDevice => Some("cd"),
            Elem::BrokenSymLink => Some("or"),
            Elem::MissingSymLinkTarget => Some("mi"),
            _ => None,
        };

        match indicator_string {
            Some(ids) => Indicator::from(ids),
            None => None,
        }
    }
}

fn to_content_style(ls: &lscolors::Style) -> ContentStyle {
    let to_crossterm_color = |c: &lscolors::Color| match c {
        lscolors::style::Color::RGB(r, g, b) => Color::Rgb {
            r: *r,
            g: *g,
            b: *b,
        },
        lscolors::style::Color::Fixed(n) => Color::AnsiValue(*n),
        lscolors::style::Color::Black => Color::Black,
        lscolors::style::Color::Red => Color::DarkRed,
        lscolors::style::Color::Green => Color::DarkGreen,
        lscolors::style::Color::Yellow => Color::DarkYellow,
        lscolors::style::Color::Blue => Color::DarkBlue,
        lscolors::style::Color::Magenta => Color::DarkMagenta,
        lscolors::style::Color::Cyan => Color::DarkCyan,
        lscolors::style::Color::White => Color::Grey,
        lscolors::style::Color::BrightBlack => Color::DarkGrey,
        lscolors::style::Color::BrightRed => Color::Red,
        lscolors::style::Color::BrightGreen => Color::Green,
        lscolors::style::Color::BrightYellow => Color::Yellow,
        lscolors::style::Color::BrightBlue => Color::Blue,
        lscolors::style::Color::BrightMagenta => Color::Magenta,
        lscolors::style::Color::BrightCyan => Color::Cyan,
        lscolors::style::Color::BrightWhite => Color::White,
    };
    let mut style = ContentStyle {
        foreground_color: ls.foreground.as_ref().map(to_crossterm_color),
        background_color: ls.background.as_ref().map(to_crossterm_color),
        ..ContentStyle::default()
    };

    if ls.font_style.bold {
        style.attributes.set(Attribute::Bold);
    }
    if ls.font_style.dimmed {
        style.attributes.set(Attribute::Dim);
    }
    if ls.font_style.italic {
        style.attributes.set(Attribute::Italic);
    }
    if ls.font_style.underline {
        style.attributes.set(Attribute::Underlined);
    }
    if ls.font_style.rapid_blink {
        style.attributes.set(Attribute::RapidBlink);
    }
    if ls.font_style.slow_blink {
        style.attributes.set(Attribute::SlowBlink);
    }
    if ls.font_style.reverse {
        style.attributes.set(Attribute::Reverse);
    }
    if ls.font_style.hidden {
        style.attributes.set(Attribute::Hidden);
    }
    if ls.font_style.strikethrough {
        style.attributes.set(Attribute::CrossedOut);
    }

    style
}

/// Decision about how to render a file
pub struct RenderDecision {
    pub icon: String,
    pub icon_style: ContentStyle,
    pub name_style: ContentStyle,
}

impl Colors {
    /// Make a render decision based on file metadata and context
    pub fn render_decision(
        &self,
        file_type: &FileType,
        extension: Option<&str>,
        git_status: Option<&GitFileStatus>,
        has_error: bool,
        draw_attention: bool,
    ) -> RenderDecision {
        if let Some(theme) = &self.theme {
            // Convert git status to simple enum
            let simple_git_status = git_status.and_then(|gs| {
                if gs.is_modified() {
                    Some(GitStatus::Modified)
                } else if gs.is_new() {
                    Some(GitStatus::NewInWorkdir)
                } else {
                    None
                }
            });
            
            // Convert booleans to enums
            let error_status = if has_error {
                ErrorStatus::HasError
            } else {
                ErrorStatus::NoError
            };
            
            let highlight = if draw_attention {
                Highlight::MaxAttention
            } else {
                Highlight::None
            };
            
            // Evaluate rules in order - first match wins
            for rule in &theme.render_rules {
                if rule.matches(file_type, extension, simple_git_status, error_status, highlight) {
                    return self.apply_rule_actions(&rule.display, file_type);
                }
            }
        }
        
        // Default fallback using existing elem system
        self.default_render_decision(file_type)
    }
    
    fn apply_rule_actions(
        &self,
        display: &crate::theme::render::DisplaySettings,
        file_type: &FileType,
    ) -> RenderDecision {
        let background = Color::Black; // Assume dark terminal
        
        // Get default colors from existing elem system
        let elem = match file_type {
            FileType::Directory { uid } => Elem::Dir { uid: *uid },
            FileType::File { uid, exec } => Elem::File { uid: *uid, exec: *exec },
            FileType::SymLink { .. } => Elem::SymLink,
            _ => Elem::File { uid: false, exec: false },
        };
        
        let default_color = self.style(&elem).foreground_color.unwrap_or(Color::White);
        
        // Apply display settings
        let icon_color = display.icon_color
            .map(|c| c.to_terminal_color(background))
            .unwrap_or(default_color);
            
        let name_color = display.name_color
            .map(|c| c.to_terminal_color(background))
            .unwrap_or(default_color);
        
        let mut icon_style = ContentStyle {
            foreground_color: Some(icon_color),
            ..ContentStyle::default()
        };
        
        let mut name_style = ContentStyle {
            foreground_color: Some(name_color),
            ..ContentStyle::default()
        };
        
        // Apply text attributes
        if display.bold.unwrap_or(false) {
            icon_style.attributes.set(Attribute::Bold);
            name_style.attributes.set(Attribute::Bold);
        }
        
        if display.italic.unwrap_or(false) {
            icon_style.attributes.set(Attribute::Italic);
            name_style.attributes.set(Attribute::Italic);
        }
        
        RenderDecision {
            icon: display.icon.clone().unwrap_or_default(),
            icon_style,
            name_style,
        }
    }
    
    fn default_render_decision(&self, file_type: &FileType) -> RenderDecision {
        let elem = match file_type {
            FileType::Directory { uid } => Elem::Dir { uid: *uid },
            FileType::File { uid, exec } => Elem::File { uid: *uid, exec: *exec },
            FileType::SymLink { .. } => Elem::SymLink,
            FileType::CharDevice => Elem::CharDevice,
            FileType::BlockDevice => Elem::BlockDevice,
            FileType::Pipe => Elem::Pipe,
            FileType::Socket => Elem::Socket,
            FileType::Special => Elem::Special,
        };
        
        let style = self.style(&elem);
        
        RenderDecision {
            icon: String::new(),
            icon_style: style,
            name_style: style,
        }
    }
}
