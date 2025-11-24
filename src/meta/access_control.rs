use crate::color::{ColoredString, Colors, Elem};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct AccessControl {
    has_acl: bool,
    selinux_context: String,
    smack_context: String,
}

impl AccessControl {
    #[cfg(not(unix))]
    pub fn for_path(_: &Path) -> Self {
        Self {
            has_acl: false,
            selinux_context: String::new(),
            smack_context: String::new(),
        }
    }

    #[cfg(unix)]
    pub fn for_path(path: &Path) -> Self {
        let has_acl = xattr::get(path, Method::Acl.name())
            .ok()
            .flatten()
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        let selinux_context = xattr::get(path, Method::Selinux.name())
            .ok()
            .flatten()
            .unwrap_or_default();

        let smack_context = xattr::get(path, Method::Smack.name())
            .ok()
            .flatten()
            .unwrap_or_default();

        Self::from_data(has_acl, &selinux_context, &smack_context)
    }

    fn from_data(has_acl: bool, selinux_context: &[u8], smack_context: &[u8]) -> Self {
        Self {
            has_acl,
            selinux_context: String::from_utf8_lossy(selinux_context).into_owned(),
            smack_context: String::from_utf8_lossy(smack_context).into_owned(),
        }
    }

    pub fn render_method(&self, colors: &Colors) -> ColoredString {
        let (symbol, elem) = if self.has_acl {
            ("+", &Elem::Acl)
        } else if self.has_context() {
            (".", &Elem::Context)
        } else {
            ("", &Elem::Acl)
        };
        colors.colorize(symbol, elem)
    }

    pub fn render_context(&self, colors: &Colors) -> ColoredString {
        let context = match (
            self.selinux_context.is_empty(),
            self.smack_context.is_empty(),
        ) {
            (true, true) => "?".to_string(),
            (false, true) => self.selinux_context.clone(),
            (true, false) => self.smack_context.clone(),
            (false, false) => format!("{}+{}", self.selinux_context, self.smack_context),
        };
        colors.colorize(context, &Elem::Context)
    }

    fn has_context(&self) -> bool {
        !self.selinux_context.is_empty() || !self.smack_context.is_empty()
    }
}

#[cfg(unix)]
enum Method {
    Acl,
    Selinux,
    Smack,
}

#[cfg(unix)]
impl Method {
    const fn name(&self) -> &'static str {
        match self {
            Method::Acl => "system.posix_acl_access",
            Method::Selinux => "security.selinux",
            Method::Smack => "security.SMACK64",
        }
    }
}
