// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

pub const POPUP_WIDTH_RANGE: (u32, u32) = (500, 1200);
pub const POPUP_HEIGHT_RANGE: (u32, u32) = (400, 1200);
pub const ICON_SIZE_RANGE: (u16, u16) = (16, 64);

pub const DEFAULT_POPUP_WIDTH: u32 = 600;
pub const DEFAULT_POPUP_HEIGHT: u32 = 700;
pub const DEFAULT_ICON_SIZE: u16 = 24;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListDensity {
    Compact,
    #[default]
    Normal,
}

pub fn clamp_popup_width(value: u32) -> u32 {
    value.clamp(POPUP_WIDTH_RANGE.0, POPUP_WIDTH_RANGE.1)
}

pub fn clamp_popup_height(value: u32) -> u32 {
    value.clamp(POPUP_HEIGHT_RANGE.0, POPUP_HEIGHT_RANGE.1)
}

pub fn clamp_icon_size(value: u16) -> u16 {
    value.clamp(ICON_SIZE_RANGE.0, ICON_SIZE_RANGE.1)
}

/// Row height of the app list. The only source of truth: rendering and
/// scroll math must both call this.
pub fn item_height(density: ListDensity, icon_size: u16, space_l: u16, space_xl: u16) -> f32 {
    let icon_size = clamp_icon_size(icon_size);
    let height = match density {
        ListDensity::Normal => space_xl.max(icon_size + 8),
        ListDensity::Compact => space_l.max(icon_size + 4),
    };
    f32::from(height)
}

/// The comment line is shown only in `Normal` density, and only when the row
/// is tall enough (same rule as before this feature).
pub fn show_comment(density: ListDensity, space_xl: u16) -> bool {
    density == ListDensity::Normal && space_xl >= 40
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_out_of_range_values() {
        assert_eq!(clamp_popup_width(0), 500);
        assert_eq!(clamp_popup_width(99_999), 1200);
        assert_eq!(clamp_popup_width(800), 800);
        assert_eq!(clamp_popup_height(0), 400);
        assert_eq!(clamp_popup_height(99_999), 1200);
        assert_eq!(clamp_icon_size(0), 16);
        assert_eq!(clamp_icon_size(500), 64);
        assert_eq!(clamp_icon_size(32), 32);
    }

    #[test]
    fn defaults_are_inside_ranges() {
        assert_eq!(clamp_popup_width(DEFAULT_POPUP_WIDTH), DEFAULT_POPUP_WIDTH);
        assert_eq!(clamp_popup_height(DEFAULT_POPUP_HEIGHT), DEFAULT_POPUP_HEIGHT);
        assert_eq!(clamp_icon_size(DEFAULT_ICON_SIZE), DEFAULT_ICON_SIZE);
    }

    #[test]
    fn normal_with_default_icon_keeps_space_xl() {
        assert_eq!(item_height(ListDensity::Normal, 24, 24, 32), 32.0);
        assert_eq!(item_height(ListDensity::Normal, 24, 24, 48), 48.0);
    }

    #[test]
    fn normal_grows_for_big_icon() {
        assert_eq!(item_height(ListDensity::Normal, 48, 24, 32), 56.0);
    }

    #[test]
    fn compact_is_shorter_than_normal_and_respects_space_l() {
        assert_eq!(item_height(ListDensity::Compact, 24, 24, 32), 28.0);
        assert_eq!(item_height(ListDensity::Compact, 16, 24, 32), 24.0);
    }

    #[test]
    fn compact_grows_for_big_icon() {
        assert_eq!(item_height(ListDensity::Compact, 64, 24, 32), 68.0);
    }

    #[test]
    fn comment_only_in_normal_and_tall_rows() {
        assert!(show_comment(ListDensity::Normal, 40));
        assert!(!show_comment(ListDensity::Normal, 32));
        assert!(!show_comment(ListDensity::Compact, 48));
    }
}
