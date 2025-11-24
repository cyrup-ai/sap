use crate::git::GitStatus;
use crate::meta::FileType;
use crossterm::style::Color;
use serde::Deserialize;

/// Extended color that supports RGBA (with faux alpha for terminals)
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ExtendedColor {
    /// Standard crossterm color
    Crossterm(Color),
    /// RGBA color with alpha channel
    Rgba { r: u8, g: u8, b: u8, a: f32 },
}

impl ExtendedColor {
    /// Convert to terminal-displayable color by blending with background
    pub fn to_terminal_color(self, background: Color) -> Color {
        match self {
            ExtendedColor::Crossterm(c) => c,
            ExtendedColor::Rgba { r, g, b, a } => {
                // Use the alpha blending from the theme module
                super::alpha::mute_color(
                    Color::Rgb { r, g, b },
                    background,
                    a,
                )
            }
        }
    }
}

// Manual Eq implementation for ExtendedColor
impl Eq for ExtendedColor {}

// Manual Eq implementation for DisplaySettings
impl Eq for DisplaySettings {}

/// A render rule that matches conditions and applies display settings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRule {
    pub matchers: RuleMatchers,
    pub display: DisplaySettings,
}

/// Conditions to match against
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleMatchers {
    pub file_types: Option<Vec<FileType>>,
    pub extensions: Option<Vec<String>>,
    pub git_statuses: Option<Vec<GitStatus>>,
    pub error_status: Option<ErrorStatus>,
    pub highlight: Option<Highlight>,
}

/// Error status for future error highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ErrorStatus {
    HasError,
    NoError,
}

/// Highlight level for drawing attention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Highlight {
    None,
    Subtle,
    MaxAttention,
}

/// Display settings to apply when rule matches
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct DisplaySettings {
    pub icon: Option<String>,
    pub icon_color: Option<ExtendedColor>,
    pub name_color: Option<ExtendedColor>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
}

impl RenderRule {
    /// Check if this rule matches the given conditions
    pub fn matches(
        &self,
        file_type: &FileType,
        extension: Option<&str>,
        git_status: Option<GitStatus>,
        error_status: ErrorStatus,
        highlight: Highlight,
    ) -> bool {
        // Check file type match
        if let Some(ref types) = self.matchers.file_types
            && !types.iter().any(|t| t == file_type) {
                return false;
            }

        // Check extension match
        if let Some(ref exts) = self.matchers.extensions {
            match extension {
                Some(ext) => {
                    if !exts.iter().any(|e| e == ext) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check git status match
        if let Some(ref statuses) = self.matchers.git_statuses {
            match git_status {
                Some(status) => {
                    if !statuses.iter().any(|s| s == &status) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check error status match
        if let Some(expected_error) = self.matchers.error_status
            && expected_error != error_status {
                return false;
            }

        // Check highlight match
        if let Some(expected_highlight) = self.matchers.highlight
            && expected_highlight != highlight {
                return false;
            }

        // All conditions match (or are not specified)
        true
    }
}


