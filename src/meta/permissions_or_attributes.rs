#[cfg(windows)]
use super::windows_attributes::WindowsAttributes;
use super::Permissions;
use crate::{
    color::{ColoredString, Colors},
    flags::Flags,
};

/// Represents either Unix-style permissions or Windows file attributes.
#[derive(Clone, Debug)]
pub enum PermissionsOrAttributes {
    /// Unix-style file permissions (read, write, execute)
    Permissions(Permissions),
    /// Windows file attributes (hidden, system, etc.)
    #[cfg(windows)]
    WindowsAttributes(WindowsAttributes),
}

impl Default for PermissionsOrAttributes {
    fn default() -> Self {
        Self::Permissions(Permissions::default())
    }
}

impl PermissionsOrAttributes {
    /// Renders the permissions or attributes as a colored string based on the provided colors and flags.
    pub fn render(&self, colors: &Colors, flags: &Flags) -> ColoredString {
        match self {
            Self::Permissions(permissions) => permissions.render(colors, flags),
            #[cfg(windows)]
            Self::WindowsAttributes(attributes) => attributes.render(colors, flags),
        }
    }
}
