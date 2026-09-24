use std::{borrow::Cow, collections::HashSet, fmt::Display, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::fl;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplicationCategory {
    pub display_name: Cow<'static, str>,
    pub icon_svg_bytes: Option<Cow<'static, [u8]>>,
    pub mime_name: Cow<'static, str>,
    pub permanent: bool,
}

impl ApplicationCategory {
    pub const ALL: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("all-applications"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/open-menu-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed(""),
        permanent: true,
    };
    pub const RECENTLY_USED: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("recently-used"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/document-open-recent-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed(""),
        permanent: true,
    };
    pub const AUDIO: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("audio"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-audio-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Audio"),
        permanent: false,
    };
    pub const VIDEO: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("video"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-video-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Video"),
        permanent: false,
    };
    pub const DEVELOPMENT: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("development"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-engineering-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Development"),
        permanent: false,
    };
    pub const GAMES: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("games"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-games-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Game"),
        permanent: false,
    };
    pub const GRAPHICS: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("graphics"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-graphics-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Graphics"),
        permanent: false,
    };
    pub const NETWORK: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("network"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/network-workgroup-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Network"),
        permanent: false,
    };
    pub const OFFICE: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("office"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-office-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Office"),
        permanent: false,
    };
    pub const SCIENCE: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("science"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-science-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Science"),
        permanent: false,
    };
    pub const SETTINGS: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("settings"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/preferences-system-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Settings"),
        permanent: false,
    };
    pub const SYSTEM: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("system"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-system-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("System"),
        permanent: false,
    };
    pub const UTILITY: ApplicationCategory = ApplicationCategory {
        display_name: Cow::Borrowed("utility"),
        icon_svg_bytes: Some(Cow::Borrowed(include_bytes!(
            "../../../res/icons/bundled/applications-utilities-symbolic.svg"
        ))),
        mime_name: Cow::Borrowed("Utility"),
        permanent: false,
    };

    pub fn get_display_name(&self) -> String {
        match self.display_name {
            std::borrow::Cow::Borrowed("all-applications") => fl!("all-applications"),
            std::borrow::Cow::Borrowed("recently-used") => fl!("recently-used"),
            std::borrow::Cow::Borrowed("audio") => fl!("audio"),
            std::borrow::Cow::Borrowed("video") => fl!("video"),
            std::borrow::Cow::Borrowed("development") => fl!("development"),
            std::borrow::Cow::Borrowed("games") => fl!("games"),
            std::borrow::Cow::Borrowed("graphics") => fl!("graphics"),
            std::borrow::Cow::Borrowed("network") => fl!("network"),
            std::borrow::Cow::Borrowed("office") => fl!("office"),
            std::borrow::Cow::Borrowed("science") => fl!("science"),
            std::borrow::Cow::Borrowed("settings") => fl!("settings"),
            std::borrow::Cow::Borrowed("system") => fl!("system"),
            std::borrow::Cow::Borrowed("utility") => fl!("utility"),
            _ => self.display_name.to_string(),
        }
    }

    pub fn get_custom_categories() -> Vec<ApplicationCategory> {
        let mut categories = HashSet::new();

        // Standard XDG locations for custom/merged menu definitions
        let mut search_dirs = Vec::new();

        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            // User-specific merged menus (where Wine, Citrix, etc. usually put .menu files)
            search_dirs.push(home_path.join(".config/menus/applications-merged"));
        }

        // System-wide menu directory overrides
        search_dirs.push(PathBuf::from("/etc/xdg/menus/applications-merged"));

        for dir in search_dirs {
            if !dir.exists() || !dir.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("menu") {
                        dbg!(&path);
                    }
                }
            }
        }

        categories.into_iter().collect()
    }
}

impl Display for ApplicationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mime_name)
    }
}
