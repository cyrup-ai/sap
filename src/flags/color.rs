use super::Configurable;
use crate::app::Cli;
use crate::config_file::Config;

use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::env;
use std::fmt;

/// A collection of flags on how to use colors.
#[derive(Clone, Debug, Default)]
pub struct Color {
    /// When to use color.
    pub when: ColorOption,
    pub theme: ThemeOption,
}

impl Color {
    /// Get a `Color` struct from [Cli], a [Config] or the [Default] values.
    ///
    /// The [ColorOption] is configured with their respective [Configurable] implementation.
    pub fn configure_from(cli: &Cli, config: &Config) -> Self {
        let when = ColorOption::configure_from(cli, config);
        let theme = ThemeOption::from_config(config);
        Self { when, theme }
    }
}

/// ThemeOption could be one of the following:
/// Custom(*.yaml): use the YAML theme file as theme file
/// if error happened, use the default theme
#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub enum ThemeOption {
    NoColor,
    #[default]
    Default,
    #[allow(dead_code)]
    NoLscolors,
    CustomLegacy(String),
    Custom,
}

impl ThemeOption {
    fn from_config(config: &Config) -> ThemeOption {
        if config.classic == Some(true) {
            ThemeOption::NoColor
        } else {
            config
                .color
                .as_ref()
                .and_then(|c| c.theme.clone())
                .unwrap_or_default()
        }
    }
}

impl<'de> de::Deserialize<'de> for ThemeOption {
    fn deserialize<D>(deserializer: D) -> Result<ThemeOption, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ThemeOptionVisitor;

        impl<'de> Visitor<'de> for ThemeOptionVisitor {
            type Value = ThemeOption;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`default` or <theme-file-path>")
            }

            fn visit_str<E>(self, value: &str) -> Result<ThemeOption, E>
            where
                E: de::Error,
            {
                match value {
                    "default" => Ok(ThemeOption::Default),
                    "custom" => Ok(ThemeOption::Custom),
                    str => Ok(ThemeOption::CustomLegacy(str.to_string())),
                }
            }
        }

        deserializer.deserialize_identifier(ThemeOptionVisitor)
    }
}

/// The flag showing when to use colors in the output.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ColorOption {
    Always,
    #[default]
    Auto,
    Never,
}

impl ColorOption {
    fn from_arg_str(value: &str) -> Self {
        match value {
            "always" => Self::Always,
            "auto" => Self::Auto,
            "never" => Self::Never,
            // Invalid value should be handled by `clap` when building an `Cli`
            other => unreachable!("Invalid value '{other}' for 'color'"),
        }
    }
}

impl Configurable<Self> for ColorOption {
    /// Get a potential `ColorOption` variant from [Cli].
    ///
    /// If the "classic" argument is passed, then this returns the [ColorOption::Never] variant in
    /// a [Some]. Otherwise if the argument is passed, this returns the variant corresponding to
    /// its parameter in a [Some]. Otherwise this returns [None].
    fn from_cli(cli: &Cli) -> Option<Self> {
        if cli.classic {
            Some(Self::Never)
        } else {
            cli.color.as_deref().map(Self::from_arg_str)
        }
    }

    /// Get a potential `ColorOption` variant from a [Config].
    ///
    /// If the `Config::classic` is `true` then this returns the Some(ColorOption::Never),
    /// Otherwise if the `Config::color::when` has value and is one of "always", "auto" or "never"
    /// this returns its corresponding variant in a [Some]. Otherwise this returns [None].
    fn from_config(config: &Config) -> Option<Self> {
        if config.classic == Some(true) {
            Some(Self::Never)
        } else {
            config.color.as_ref().and_then(|c| c.when)
        }
    }

    fn from_environment() -> Option<Self> {
        if env::var("NO_COLOR").is_ok() {
            Some(Self::Never)
        } else {
            None
        }
    }
}
