// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

pub const POPUP_WIDTH_RANGE: (u32, u32) = (500, 1200);
pub const POPUP_HEIGHT_RANGE: (u32, u32) = (400, 1200);
pub const ICON_SIZE_RANGE: (u16, u16) = (16, 64);

pub const DEFAULT_POPUP_WIDTH: u32 = 600;
pub const DEFAULT_POPUP_HEIGHT: u32 = 700;
/// 0 means "follow the theme" (`space_l`), as before this feature.
pub const DEFAULT_ICON_SIZE: u16 = 0;
/// Minimum row height of `ListColumn` items (libcosmic).
pub const MIN_ROW_HEIGHT: u16 = 32;
/// Vertical padding of the app button (5 px on each side).
const BUTTON_PADDING: u16 = 10;

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
    let themed = match density {
        ListDensity::Normal => space_xl,
        ListDensity::Compact => space_l,
    };
    f32::from(themed.max(icon_size + BUTTON_PADDING).max(MIN_ROW_HEIGHT))
}

/// The comment line is shown only in `Normal` density, and only when the row
/// is tall enough (same rule as before this feature).
pub fn show_comment(density: ListDensity, space_xl: u16) -> bool {
    density == ListDensity::Normal && space_xl >= 40
}

/// Icon size to draw: the configured one, or `space_l` when it is 0 (auto).
pub fn resolve_icon_size(configured: u16, space_l: u16) -> u16 {
    clamp_icon_size(if configured == 0 { space_l } else { configured })
}

/// Smallest icon drawn in a grid cell; the list icon size is used when larger.
const GRID_MIN_ICON: u16 = 48;
/// Width a grid cell aims for; the list width is split into as many as fit.
const GRID_CELL_WIDTH: f32 = 100.0;
/// Cell padding, gap between icon and name, and the name line.
const GRID_CELL_EXTRA: u16 = 2 * 8 + 4 + 20;

pub fn grid_icon_size(list_icon_size: u16) -> u16 {
    list_icon_size.max(GRID_MIN_ICON)
}

pub fn grid_cell_height(list_icon_size: u16) -> f32 {
    f32::from(grid_icon_size(list_icon_size) + GRID_CELL_EXTRA)
}

pub fn grid_columns(list_width: f32) -> usize {
    ((list_width / GRID_CELL_WIDTH) as usize).max(1)
}

/// Keyboard selection after moving by `delta` items (±1 within a row, ±columns
/// between rows). Nothing selected: moving forward selects the first item.
/// A move that would leave the list keeps the current item.
pub fn step_selection(current: Option<usize>, delta: isize, len: usize) -> Option<usize> {
    match current {
        _ if len == 0 => None,
        None => (delta > 0).then_some(0),
        Some(index) => Some(
            index
                .checked_add_signed(delta)
                .filter(|next| *next < len)
                .unwrap_or(index),
        ),
    }
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
    fn defaults_keep_the_old_look() {
        assert_eq!(clamp_popup_width(DEFAULT_POPUP_WIDTH), DEFAULT_POPUP_WIDTH);
        assert_eq!(clamp_popup_height(DEFAULT_POPUP_HEIGHT), DEFAULT_POPUP_HEIGHT);
        assert_eq!(DEFAULT_ICON_SIZE, 0);
    }

    #[test]
    fn auto_icon_follows_theme_space_l() {
        assert_eq!(resolve_icon_size(0, 24), 24); // Compact theme
        assert_eq!(resolve_icon_size(0, 32), 32); // Standard theme
        assert_eq!(resolve_icon_size(0, 48), 48); // Spacious theme
        assert_eq!(resolve_icon_size(0, 200), 64);
        assert_eq!(resolve_icon_size(30, 48), 30);
        assert_eq!(resolve_icon_size(500, 48), 64);
    }

    #[test]
    fn normal_keeps_space_xl_with_auto_icon_on_roomy_themes() {
        // (space_l, space_xl): Standard, Spacious
        for (l, xl) in [(32u16, 48u16), (48, 64)] {
            let icon = resolve_icon_size(0, l);
            assert_eq!(item_height(ListDensity::Normal, icon, l, xl), f32::from(xl));
        }
    }

    #[test]
    fn compact_theme_row_grows_by_two_to_fit_the_full_icon() {
        // icon 24 + 10 px button padding > space_xl 32
        assert_eq!(item_height(ListDensity::Normal, 24, 24, 32), 34.0);
    }

    #[test]
    fn row_always_fits_icon_plus_button_padding() {
        assert_eq!(item_height(ListDensity::Normal, 64, 32, 48), 74.0);
        assert_eq!(item_height(ListDensity::Compact, 64, 32, 48), 74.0);
        assert_eq!(item_height(ListDensity::Compact, 48, 48, 64), 58.0);
    }

    #[test]
    fn compact_is_shorter_than_normal_on_roomy_themes() {
        assert_eq!(item_height(ListDensity::Compact, 32, 32, 48), 42.0);
        assert_eq!(item_height(ListDensity::Normal, 32, 32, 48), 48.0);
    }

    #[test]
    fn row_is_never_below_list_column_minimum() {
        assert_eq!(item_height(ListDensity::Compact, 16, 24, 32), 32.0);
        assert_eq!(item_height(ListDensity::Compact, 24, 24, 32), 34.0);
    }

    #[test]
    fn comment_only_in_normal_and_tall_rows() {
        assert!(show_comment(ListDensity::Normal, 40));
        assert!(!show_comment(ListDensity::Normal, 32));
        assert!(!show_comment(ListDensity::Compact, 48));
    }

    #[test]
    fn grid_fits_as_many_cells_as_the_width_allows() {
        assert_eq!(grid_columns(50.0), 1);
        assert_eq!(grid_columns(350.0), 3);
        assert_eq!(grid_icon_size(32), 48);
        assert_eq!(grid_icon_size(64), 64);
        assert_eq!(grid_cell_height(32), 88.0);
    }

    #[test]
    fn selection_steps_stay_inside_the_list() {
        assert_eq!(step_selection(None, 1, 0), None);
        assert_eq!(step_selection(None, -1, 5), None);
        assert_eq!(step_selection(None, 4, 5), Some(0));
        assert_eq!(step_selection(Some(1), 4, 5), Some(1)); // no item below
        assert_eq!(step_selection(Some(0), 4, 5), Some(4));
        assert_eq!(step_selection(Some(2), -4, 5), Some(2)); // no row above
        assert_eq!(step_selection(Some(2), -1, 5), Some(1));
        assert_eq!(step_selection(Some(9), 1, 5), Some(9));
    }
}
