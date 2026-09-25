use std::borrow::Cow;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::fl;

/// Where a category comes from; permanent ones always sit on top.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CategoryKind {
    Permanent,
    Standard,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CategoryName {
    /// Fluent key of a built-in category.
    Localized(&'static str),
    /// Name typed by the user.
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CategoryIcon {
    Bundled(&'static [u8]),
    /// Icon theme name, e.g. `folder-symbolic`.
    Named(String),
}

/// A user-defined category as stored in the config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomCategory {
    /// Stable id; the category key is `custom:<id>`.
    pub id: String,
    pub name: String,
    /// Stem of a bundled icon (see [`CATEGORY_ICONS`]) or an icon theme name.
    pub icon: String,
}

#[derive(Clone, Debug)]
pub struct ApplicationCategory {
    /// Stable key used in the config: `favorites`, `all-applications`,
    /// `recently-used`, the freedesktop name (`Audio`) or `custom:<id>`.
    pub key: Cow<'static, str>,
    pub name: CategoryName,
    pub icon: CategoryIcon,
    /// Freedesktop category matched against desktop entries; empty for
    /// permanent and custom categories.
    pub mime_name: &'static str,
    pub kind: CategoryKind,
}

/// Categories are identified by key, so a renamed category stays selected.
impl PartialEq for ApplicationCategory {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for ApplicationCategory {}

pub const CUSTOM_KEY_PREFIX: &str = "custom:";
pub const DEFAULT_CUSTOM_ICON: &str = "folder-symbolic";

/// Bundled icons, by file stem; usable as icons of custom categories.
pub const CATEGORY_ICONS: &[(&str, &[u8])] = &[
    (
        "applications-audio-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-audio-symbolic.svg"),
    ),
    (
        "applications-video-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-video-symbolic.svg"),
    ),
    (
        "applications-engineering-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-engineering-symbolic.svg"),
    ),
    (
        "applications-games-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-games-symbolic.svg"),
    ),
    (
        "applications-graphics-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-graphics-symbolic.svg"),
    ),
    (
        "network-workgroup-symbolic",
        include_bytes!("../../../res/icons/bundled/network-workgroup-symbolic.svg"),
    ),
    (
        "applications-office-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-office-symbolic.svg"),
    ),
    (
        "applications-science-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-science-symbolic.svg"),
    ),
    (
        "preferences-system-symbolic",
        include_bytes!("../../../res/icons/bundled/preferences-system-symbolic.svg"),
    ),
    (
        "applications-system-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-system-symbolic.svg"),
    ),
    (
        "applications-utilities-symbolic",
        include_bytes!("../../../res/icons/bundled/applications-utilities-symbolic.svg"),
    ),
    (
        "starred-symbolic",
        include_bytes!("../../../res/icons/bundled/starred-symbolic.svg"),
    ),
    (
        "document-open-recent-symbolic",
        include_bytes!("../../../res/icons/bundled/document-open-recent-symbolic.svg"),
    ),
];

/// Icon theme names offered for custom categories besides the bundled ones.
pub const NAMED_ICON_CHOICES: &[&str] = &[
    "folder-symbolic",
    "applications-internet-symbolic",
    "mail-unread-symbolic",
    "text-editor-symbolic",
    "utilities-terminal-symbolic",
    "camera-photo-symbolic",
    "input-gaming-symbolic",
    "emblem-documents-symbolic",
    "user-home-symbolic",
    "weather-clear-symbolic",
];

impl CategoryIcon {
    /// Bundled icon for a known stem, otherwise an icon theme name.
    pub fn from_name(name: &str) -> CategoryIcon {
        CATEGORY_ICONS
            .iter()
            .find(|(stem, _)| *stem == name)
            .map_or_else(
                || CategoryIcon::Named(name.to_string()),
                |(_, bytes)| CategoryIcon::Bundled(bytes),
            )
    }
}

impl ApplicationCategory {
    pub const FAVORITES: ApplicationCategory = ApplicationCategory {
        key: Cow::Borrowed("favorites"),
        name: CategoryName::Localized("favorites"),
        icon: CategoryIcon::Bundled(include_bytes!(
            "../../../res/icons/bundled/starred-symbolic.svg"
        )),
        mime_name: "",
        kind: CategoryKind::Permanent,
    };
    pub const ALL: ApplicationCategory = ApplicationCategory {
        key: Cow::Borrowed("all-applications"),
        name: CategoryName::Localized("all-applications"),
        icon: CategoryIcon::Bundled(include_bytes!(
            "../../../res/icons/bundled/open-menu-symbolic.svg"
        )),
        mime_name: "",
        kind: CategoryKind::Permanent,
    };
    pub const RECENTLY_USED: ApplicationCategory = ApplicationCategory {
        key: Cow::Borrowed("recently-used"),
        name: CategoryName::Localized("recently-used"),
        icon: CategoryIcon::Bundled(include_bytes!(
            "../../../res/icons/bundled/document-open-recent-symbolic.svg"
        )),
        mime_name: "",
        kind: CategoryKind::Permanent,
    };
    pub const AUDIO: ApplicationCategory = ApplicationCategory::standard(
        "audio",
        "Audio",
        include_bytes!("../../../res/icons/bundled/applications-audio-symbolic.svg"),
    );
    pub const VIDEO: ApplicationCategory = ApplicationCategory::standard(
        "video",
        "Video",
        include_bytes!("../../../res/icons/bundled/applications-video-symbolic.svg"),
    );
    pub const DEVELOPMENT: ApplicationCategory = ApplicationCategory::standard(
        "development",
        "Development",
        include_bytes!("../../../res/icons/bundled/applications-engineering-symbolic.svg"),
    );
    pub const GAMES: ApplicationCategory = ApplicationCategory::standard(
        "games",
        "Game",
        include_bytes!("../../../res/icons/bundled/applications-games-symbolic.svg"),
    );
    pub const GRAPHICS: ApplicationCategory = ApplicationCategory::standard(
        "graphics",
        "Graphics",
        include_bytes!("../../../res/icons/bundled/applications-graphics-symbolic.svg"),
    );
    pub const NETWORK: ApplicationCategory = ApplicationCategory::standard(
        "network",
        "Network",
        include_bytes!("../../../res/icons/bundled/network-workgroup-symbolic.svg"),
    );
    pub const OFFICE: ApplicationCategory = ApplicationCategory::standard(
        "office",
        "Office",
        include_bytes!("../../../res/icons/bundled/applications-office-symbolic.svg"),
    );
    pub const SCIENCE: ApplicationCategory = ApplicationCategory::standard(
        "science",
        "Science",
        include_bytes!("../../../res/icons/bundled/applications-science-symbolic.svg"),
    );
    pub const SETTINGS: ApplicationCategory = ApplicationCategory::standard(
        "settings",
        "Settings",
        include_bytes!("../../../res/icons/bundled/preferences-system-symbolic.svg"),
    );
    pub const SYSTEM: ApplicationCategory = ApplicationCategory::standard(
        "system",
        "System",
        include_bytes!("../../../res/icons/bundled/applications-system-symbolic.svg"),
    );
    pub const UTILITY: ApplicationCategory = ApplicationCategory::standard(
        "utility",
        "Utility",
        include_bytes!("../../../res/icons/bundled/applications-utilities-symbolic.svg"),
    );

    /// Permanent categories in display order.
    pub const PERMANENT: [ApplicationCategory; 3] = [
        ApplicationCategory::FAVORITES,
        ApplicationCategory::ALL,
        ApplicationCategory::RECENTLY_USED,
    ];

    /// Freedesktop categories in default display order.
    pub const STANDARD: [ApplicationCategory; 11] = [
        ApplicationCategory::AUDIO,
        ApplicationCategory::VIDEO,
        ApplicationCategory::DEVELOPMENT,
        ApplicationCategory::GAMES,
        ApplicationCategory::GRAPHICS,
        ApplicationCategory::NETWORK,
        ApplicationCategory::OFFICE,
        ApplicationCategory::SCIENCE,
        ApplicationCategory::SETTINGS,
        ApplicationCategory::SYSTEM,
        ApplicationCategory::UTILITY,
    ];

    const fn standard(
        fl_key: &'static str,
        mime_name: &'static str,
        icon: &'static [u8],
    ) -> ApplicationCategory {
        ApplicationCategory {
            key: Cow::Borrowed(mime_name),
            name: CategoryName::Localized(fl_key),
            icon: CategoryIcon::Bundled(icon),
            mime_name,
            kind: CategoryKind::Standard,
        }
    }

    pub fn custom_key(id: &str) -> String {
        format!("{CUSTOM_KEY_PREFIX}{id}")
    }

    pub fn from_custom(custom: &CustomCategory) -> ApplicationCategory {
        ApplicationCategory {
            key: Cow::Owned(Self::custom_key(&custom.id)),
            name: CategoryName::Custom(custom.name.clone()),
            icon: CategoryIcon::from_name(&custom.icon),
            mime_name: "",
            kind: CategoryKind::Custom,
        }
    }

    pub fn is_permanent(&self) -> bool {
        self.kind == CategoryKind::Permanent
    }

    pub fn get_display_name(&self) -> String {
        match &self.name {
            CategoryName::Custom(name) => name.clone(),
            CategoryName::Localized(key) => match *key {
                "favorites" => fl!("favorites"),
                "all-applications" => fl!("all-applications"),
                "recently-used" => fl!("recently-used"),
                "audio" => fl!("audio"),
                "video" => fl!("video"),
                "development" => fl!("development"),
                "games" => fl!("games"),
                "graphics" => fl!("graphics"),
                "network" => fl!("network"),
                "office" => fl!("office"),
                "science" => fl!("science"),
                "settings" => fl!("settings"),
                "system" => fl!("system"),
                "utility" => fl!("utility"),
                other => other.to_string(),
            },
        }
    }
}

impl Display for ApplicationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}
