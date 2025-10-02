use super::locale::current_locale;
use crate::color::{ColoredString, Colors, Elem};
use crate::flags::{DateFlag, Flags};
use chrono::{DateTime, Duration, Local};
use chrono_humanize::HumanTime;
use std::fs::Metadata;

use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Date {
    Date(DateTime<Local>),
    Invalid,
}

// Note that this is split from the From for Metadata so we can test this one (as we can't mock Metadata)
impl From<SystemTime> for Date {
    fn from(systime: SystemTime) -> Self {
        match systime.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                if let Some(datetime) = DateTime::from_timestamp(duration.as_secs() as i64, 0) {
                    Date::Date(datetime.with_timezone(&Local))
                } else {
                    Date::Invalid
                }
            }
            Err(_) => Date::Invalid,
        }
    }
}

impl From<&Metadata> for Date {
    fn from(meta: &Metadata) -> Self {
        match meta.modified() {
            Ok(modified) => modified.into(),
            Err(_) => {
                // Use system time as fallback if modified time is unavailable
                std::time::SystemTime::now().into()
            }
        }
    }
}

impl Date {
    pub fn render(&self, colors: &Colors, flags: &Flags) -> ColoredString {
        let now = Local::now();
        #[allow(deprecated)]
        let elem = match self {
            &Date::Date(modified) if modified > now - Duration::hours(1) => Elem::HourOld,
            &Date::Date(modified) if modified > now - Duration::days(1) => Elem::DayOld,
            &Date::Date(_) | Date::Invalid => Elem::Older,
        };
        colors.colorize(self.date_string(flags), &elem)
    }

    fn date_string(&self, flags: &Flags) -> String {
        let locale = current_locale();

        if let Date::Date(val) = self {
            let date_str = match &flags.date {
                #[allow(deprecated)]
                DateFlag::Date => val.format("%b %d %H:%M").to_string(),
                DateFlag::Locale => val.format_localized("%b %d %H:%M", locale).to_string(),
                DateFlag::Relative => {
                    let duration = *val - Local::now();
                    let human_time = HumanTime::from(duration).to_string();
                    format!(" {}", human_time)
                }
                DateFlag::Iso => {
                    // 365.2425 * 24 * 60 * 60 = 31556952 seconds per year
                    // 15778476 seconds are 6 months
                    #[allow(deprecated)]
                    if *val > Local::now() - Duration::seconds(15_778_476) {
                        format!(" {}", val.format("%m-%d %R"))
                    } else {
                        format!(" {}", val.format("%F"))
                    }
                }
                DateFlag::Formatted(format) => val.format_localized(format, locale).to_string(),
            };

            // Add time-based icon prefix
            let now = Local::now();
            #[allow(deprecated)]
            let icon = if *val > now - Duration::hours(1) {
                "󰓎" // Star icon for very recent/new
            } else if *val > now - Duration::days(7) {
                "󰨱" // Schedule icon for this week
            } else if *val > now - Duration::days(30) {
                "󰗚" // Book icon for this month
            } else {
                "󰘚" // Data/DB icon for archived
            };

            format!("{} {}", icon, date_str)
        } else {
            String::from(" ")
        }
    }
}
