// SPDX-License-Identifier: GPL-3.0-only

use crate::fl;
use crate::model::appearance::{
    DEFAULT_ICON_SIZE, DEFAULT_POPUP_HEIGHT, DEFAULT_POPUP_WIDTH, ListDensity,
};
use crate::model::application_category::CustomCategory;
use crate::model::power_action::PowerAction;
use cosmic::{
    Application,
    cosmic_config::{self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
#[id = "cosmic-ext-classic-menu-plus"]
pub struct AppletConfig {
    pub app_menu_position: HorizontalPosition,
    pub search_field_position: VerticalPosition,
    pub applet_button_style: AppletButtonStyle,
    pub user_widget: UserWidgetStyle,
    pub button_label: String,
    pub button_icon: String,
    pub recent_applications: Vec<RecentApplication>,
    pub power_buttons: Vec<PowerAction>,
    pub popup_width: u32,
    pub popup_height: u32,
    pub app_icon_size: u16,
    pub list_density: ListDensity,
    pub pinned_apps: Vec<String>,
    pub custom_categories: Vec<CustomCategory>,
    /// app id -> category key ("Audio", "custom:c1", ...) it was moved to.
    pub app_category_overrides: BTreeMap<String, String>,
    pub hidden_apps: Vec<String>,
    /// Category keys hidden from the menu.
    pub hidden_categories: Vec<String>,
    /// Category keys in display order; unlisted ones follow in default order.
    pub category_order: Vec<String>,
}

impl Default for AppletConfig {
    fn default() -> Self {
        AppletConfig {
            app_menu_position: HorizontalPosition::default(),
            search_field_position: VerticalPosition::default(),
            applet_button_style: AppletButtonStyle::default(),
            user_widget: UserWidgetStyle::default(),
            button_label: fl!("menu-label").to_owned(),
            button_icon: format!(
                "/usr/share/cosmic/{}/applet-buttons/default.svg",
                crate::applet::Applet::APP_ID
            )
            .to_owned(),
            recent_applications: vec![],
            power_buttons: PowerAction::DEFAULT_ORDER.to_vec(),
            popup_width: DEFAULT_POPUP_WIDTH,
            popup_height: DEFAULT_POPUP_HEIGHT,
            app_icon_size: DEFAULT_ICON_SIZE,
            list_density: ListDensity::default(),
            pinned_apps: vec![],
            custom_categories: vec![],
            app_category_overrides: BTreeMap::new(),
            hidden_apps: vec![],
            hidden_categories: vec![],
            category_order: vec![],
        }
    }
}

impl AppletConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(crate::applet::Applet::APP_ID, 1).ok()
    }

    pub fn config() -> AppletConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                AppletConfig::get_entry(&config_handler).unwrap_or_else(|(_errs, config)| config)
            }
            None => AppletConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]

pub enum AppletButtonStyle {
    IconOnly,
    LabelOnly,
    IconAndLabel,
    Auto,
}

impl Default for AppletButtonStyle {
    fn default() -> Self {
        AppletButtonStyle::Auto
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]

pub enum UserWidgetStyle {
    UsernamePrefered,
    RealNamePrefered,
    None,
}

impl Default for UserWidgetStyle {
    fn default() -> Self {
        UserWidgetStyle::UsernamePrefered
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]

pub enum HorizontalPosition {
    Left,
    Right,
}

impl Default for HorizontalPosition {
    fn default() -> Self {
        HorizontalPosition::Left
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum VerticalPosition {
    Top,
    Bottom,
}

impl Default for VerticalPosition {
    fn default() -> Self {
        VerticalPosition::Top
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecentApplication {
    pub app_id: String,
    pub launch_count: u32,
}
