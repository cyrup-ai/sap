//! This module defines the [DateFlag]. To set it up from [Cli], a [Config] and its
//! [Default] value, use its [configure_from](Configurable::configure_from) method.

use super::Configurable;

use crate::app::{self, Cli};
use crate::config_file::Config;
use crate::print_error;

/// The flag showing which kind of time stamps to display.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum DateFlag {
    #[default]
    Date,
    Locale,
    Relative,
    Iso,
    Formatted(String),
}

impl DateFlag {
    /// Get a value from a date format string
    fn from_format_string(value: &str) -> Option<Self> {
        if app::validate_time_format(value).is_ok() {
            Some(Self::Formatted(value[1..].to_string()))
        } else {
            print_error!("Not a valid date format: {}.", value);
            None
        }
    }

    /// Get a value from a str.
    fn from_str<S: AsRef<str>>(value: S) -> Option<Self> {
        let value = value.as_ref();
        match value {
            "date" => Some(Self::Date),
            "locale" => Some(Self::Locale),
            "relative" => Some(Self::Relative),
            _ if value.starts_with('+') => Self::from_format_string(value),
            _ => {
                print_error!("Not a valid date value: {}.", value);
                None
            }
        }
    }
}

impl Configurable<Self> for DateFlag {
    /// Get a potential `DateFlag` variant from [Cli].
    ///
    /// If the "classic" argument is passed, then this returns the [DateFlag::Date] variant in a
    /// [Some]. Otherwise if the argument is passed, this returns the variant corresponding to its
    /// parameter in a [Some]. Otherwise this returns [None].
    fn from_cli(cli: &Cli) -> Option<Self> {
        if cli.classic {
            Some(Self::Date)
        } else {
            cli.date.as_deref().and_then(Self::from_str)
        }
    }

    /// Get a potential `DateFlag` variant from a [Config].
    ///
    /// If the `Config::classic` is `true` then this returns the Some(DateFlag::Date),
    /// Otherwise if the `Config::date` has value and is one of "date", "locale" or "relative",
    /// this returns its corresponding variant in a [Some].
    /// Otherwise this returns [None].
    fn from_config(config: &Config) -> Option<Self> {
        if config.classic == Some(true) {
            Some(Self::Date)
        } else {
            config.date.as_ref().and_then(Self::from_str)
        }
    }

    /// Get a potential `DateFlag` variant from the environment.
    fn from_environment() -> Option<Self> {
        if let Ok(value) = std::env::var("TIME_STYLE") {
            match value.as_str() {
                "full-iso" => Some(Self::Formatted("%F %T.%f %z".into())),
                "long-iso" => Some(Self::Formatted("%F %R".into())),
                "locale" => Some(Self::Locale),
                "iso" => Some(Self::Iso),
                _ if value.starts_with('+') => Self::from_format_string(&value),
                _ => {
                    print_error!("Not a valid date value: {}.", value);
                    None
                }
            }
        } else {
            None
        }
    }
}
