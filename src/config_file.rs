//! This module provides methods to handle the program's config files and
//! operations related to this.
use crate::flags::display::Display;
use crate::flags::icons::{IconOption, IconTheme};
use crate::flags::layout::Layout;
use crate::flags::permission::PermissionFlag;
use crate::flags::size::SizeFlag;
use crate::flags::sorting::{DirGrouping, SortColumn};
use crate::flags::HyperlinkOption;
use crate::flags::{ColorOption, ThemeOption};
use crate::print_error;

use std::path::{Path, PathBuf};

use serde::Deserialize;

use std::fs;
use std::io;


/// A struct to hold an optional configuration items, and provides methods
/// around error handling in a config file.
#[derive(Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub classic: Option<bool>,
    pub blocks: Option<Vec<String>>,
    pub color: Option<Color>,
    pub date: Option<String>,
    pub dereference: Option<bool>,
    pub display: Option<Display>,
    pub icons: Option<Icons>,
    pub ignore_globs: Option<Vec<String>>,
    pub indicators: Option<bool>,
    pub layout: Option<Layout>,
    pub recursion: Option<Recursion>,
    pub size: Option<SizeFlag>,
    pub permission: Option<PermissionFlag>,
    pub sorting: Option<Sorting>,
    pub no_symlink: Option<bool>,
    pub total_size: Option<bool>,
    pub symlink_arrow: Option<String>,
    pub hyperlink: Option<HyperlinkOption>,
    pub header: Option<bool>,
    pub literal: Option<bool>,
    pub truncate_owner: Option<TruncateOwner>,
    pub llm: Option<bool>,
}

#[derive(Eq, PartialEq, Debug, Deserialize)]
pub struct Color {
    pub when: Option<ColorOption>,
    pub theme: Option<ThemeOption>,
}

#[derive(Eq, PartialEq, Debug, Deserialize)]
pub struct Icons {
    pub when: Option<IconOption>,
    pub theme: Option<IconTheme>,
    pub separator: Option<String>,
}

#[derive(Eq, PartialEq, Debug, Deserialize)]
pub struct Recursion {
    pub enabled: Option<bool>,
    pub depth: Option<usize>,
}

#[derive(Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Sorting {
    pub column: Option<SortColumn>,
    pub reverse: Option<bool>,
    pub dir_grouping: Option<DirGrouping>,
}

#[derive(Eq, PartialEq, Debug, Deserialize)]
pub struct TruncateOwner {
    pub after: Option<usize>,
    pub marker: Option<String>,
}

/// This expand the `~` in path to HOME dir
/// returns the origin one if no `~` found;
/// returns None if error happened when getting home dir
///
/// Implementing this to reuse the `dirs` dependency, avoid adding new one
pub fn expand_home<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let p = path.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir();
    }
    dirs::home_dir().and_then(|mut h| {
        if h == Path::new("/") {
            // Corner case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").map_or_else(
                |_| {
                    eprintln!("Warning: Failed to strip '~' prefix from path");
                    None
                },
                |stripped| Some(stripped.to_path_buf())
            )
        } else {
            p.strip_prefix("~/").map_or_else(
                |_| {
                    eprintln!("Warning: Failed to strip '~/' prefix from path");
                    None
                },
                |stripped| {
                    h.push(stripped);
                    Some(h)
                }
            )
        }
    })
}

impl Config {
    /// This constructs a Config struct with all None
    pub fn with_none() -> Self {
        Self {
            classic: None,
            blocks: None,
            color: None,
            date: None,
            dereference: None,
            display: None,
            icons: None,
            ignore_globs: None,
            indicators: None,
            layout: None,
            recursion: None,
            size: None,
            permission: None,
            sorting: None,
            no_symlink: None,
            total_size: None,
            symlink_arrow: None,
            hyperlink: None,
            header: None,
            literal: None,
            truncate_owner: None,
            llm: None,
        }
    }

    /// This constructs a Config struct with builtin default values from DEFAULT_CONFIG.
    /// This is useful for tests and when you need deterministic default values without file I/O.
    pub fn builtin() -> Self {
        Self::from_yaml(DEFAULT_CONFIG)
            .unwrap_or_else(|e| {
                eprintln!("sap: fatal error: failed to parse builtin config: {}", e);
                std::process::exit(1);
            })
    }

    /// This constructs a Config struct with a passed file path.
    pub fn from_file<P: AsRef<Path>>(file: P) -> Option<Self> {
        let file = file.as_ref();
        match fs::read(file) {
            Ok(f) => match Self::from_yaml(&String::from_utf8_lossy(&f)) {
                Ok(c) => Some(c),
                Err(e) => {
                    print_error!(
                        "Configuration file {} format error, {}.",
                        file.to_string_lossy(),
                        e
                    );
                    None
                }
            },
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    print_error!(
                        "Can not open config file {}: {}.",
                        file.to_string_lossy(),
                        e
                    );
                }
                None
            }
        }
    }

    /// This constructs a Config struct with a passed [Yaml] str.
    /// If error happened, return the [serde_yaml::Error].
    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str::<Self>(yaml)
    }

    /// Config paths for non-Windows platforms will be read from
    /// `$XDG_CONFIG_HOME/lsd` or `$HOME/.config/lsd`
    /// (usually, those are the same) in that order.
    /// The default paths for Windows will be read from
    /// `%APPDATA%\lsd` or `%USERPROFILE%\.config\lsd` in that order.
    /// This will apply both to the config file and the theme file.
    pub fn config_paths() -> impl Iterator<Item = PathBuf> {
        

        [
            dirs::home_dir().map(|h| h.join(".config")),
            dirs::config_dir(),
            #[cfg(not(windows))]
            None,
        ]
        .iter()
        .filter_map(|p| p.as_ref().map(|p| p.join("lsd")))
        .collect::<Vec<_>>()
        .into_iter()
    }
}

impl Default for Config {
    /// Try to find either config.yaml or config.yml in the config directories
    /// and use the first one that is found. If none are found, or the parsing fails,
    /// use the default from DEFAULT_CONFIG.
    fn default() -> Self {
        Config::config_paths()
            .find_map(|p| {
                let yaml = p.join("config.yaml");
                let yml = p.join("config.yml");
                if yaml.is_file() {
                    Config::from_file(yaml)
                } else if yml.is_file() {
                    Config::from_file(yml)
                } else {
                    None
                }
            })
            .unwrap_or_else(Self::builtin)
    }
}

const DEFAULT_CONFIG: &str = r#"---
# == Classic ==
# This is a shorthand to override some of the options to be backwards compatible
# with `ls`. It affects the "color"->"when", "sorting"->"dir-grouping", "date"
# and "icons"->"when" options.
# Possible values: false, true
classic: false

# == Blocks ==
# This specifies the columns and their order when using the long and the tree
# layout.
# Possible values: permission, user, group, context, size, date, name, inode, git
blocks:
  - permission
  - user
  - group
  - size
  - date
  - name

# == Color ==
# This has various color options. (Will be expanded in the future.)
color:
  # When to colorize the output.
  # When "classic" is set, this is set to "never".
  # Possible values: never, auto, always
  when: auto
  # How to colorize the output.
  # When "classic" is set, this is set to "no-color".
  # Possible values: default, no-color, no-lscolors, <theme-file-name>
  # when specifying <theme-file-name>, lsd will look up theme file in
  # XDG Base Directory if relative
  # The file path if absolute
  theme: default

# == Date ==
# This specifies the date format for the date column. The freeform format
# accepts an strftime like string.
# When "classic" is set, this is set to "date".
# Possible values: date, locale, relative, +<date_format>
# date: date

# == Dereference ==
# Whether to dereference symbolic links.
# Possible values: false, true
dereference: false

# == Display ==
# What items to display. Do not specify this for the default behavior.
# Possible values: all, almost-all, directory-only
# display: all

# == Icons ==
icons:
  # When to use icons.
  # When "classic" is set, this is set to "never".
  # Possible values: always, auto, never
  when: auto
  # Which icon theme to use.
  # Possible values: fancy, unicode
  theme: fancy
  # The string between the icons and the name.
  # Possible values: any string (eg: " |")
  separator: " "

# == Ignore Globs ==
# A list of globs to ignore when listing.
# Default includes common build dirs, dependencies, and large files.
# To override defaults and use custom patterns:
# ignore-globs:
#   - .git
#   - node_modules
#   - "*.tmp"
# To disable all default patterns and start fresh:
# ignore-globs: []

# == Indicators ==
# Whether to add indicator characters to certain listed files.
# Possible values: false, true
indicators: false

# == Layout ==
# Which layout to use. "oneline" might be a bit confusing here and should be
# called "one-per-line". It might be changed in the future.
# Possible values: grid, tree, oneline
layout: grid

# == Recursion ==
recursion:
  # Whether to enable recursion.
  # Possible values: false, true
  enabled: false
  # How deep the recursion should go. This has to be a positive integer. Leave
  # it unspecified for (virtually) infinite.
  # depth: 3

# == Size ==
# Specifies the format of the size column.
# Possible values: default, short, bytes
size: default

# == Permission ==
# Specify the format of the permission column.
# Possible value: rwx, octal, attributes, disable
# permission: rwx

# == Sorting ==
sorting:
  # Specify what to sort by.
  # Possible values: extension, name, time, size, version
  column: name
  # Whether to reverse the sorting.
  # Possible values: false, true
  reverse: false
  # Whether to group directories together and where.
  # When "classic" is set, this is set to "none".
  # Possible values: first, last, none
  dir-grouping: none

# == No Symlink ==
# Whether to omit showing symlink targets
# Possible values: false, true
no-symlink: false

# == Total size ==
# Whether to display the total size of directories.
# Possible values: false, true
total-size: false

# == Hyperlink ==
# Whether to display the total size of directories.
# Possible values: always, auto, never
hyperlink: never

# == Symlink arrow ==
# Specifies how the symlink arrow display, chars in both ascii and utf8
symlink-arrow: ⇒

# == Literal ==
# Whether to print entry names without quoting
# Possible values: false, true
literal: false

# == Truncate owner ==
# How to truncate the username and group name for the file if they exceed a
# certain number of characters.
truncate-owner:
  # Number of characters to keep. By default, no truncation is done (empty value).
  after:
  # String to be appended to a name if truncated.
  marker: ""
"#;
