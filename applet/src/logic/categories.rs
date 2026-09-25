// SPDX-License-Identifier: GPL-3.0-only

//! Category resolution: which categories to show, in what order, and which
//! apps belong to each. Pure functions over [`AppletConfig`] so they can be
//! unit-tested without touching the desktop-entry cache.

use std::collections::HashSet;
use std::sync::Arc;

use crate::config::AppletConfig;
use crate::model::application_category::{ApplicationCategory, CategoryKind, CustomCategory};
use crate::model::application_entry::ApplicationEntry;
use crate::model::favorites::resolve_favorites;
use crate::model::power_action::MoveDirection;

/// Categories an app is filed under, honouring a move-to override: if the
/// override still points at a category that exists, that replaces the
/// `.desktop` categories entirely; otherwise the app keeps its original ones.
pub fn effective_categories<'a>(
    app: &'a ApplicationEntry,
    config: &'a AppletConfig,
) -> Vec<&'a str> {
    if let Some(target) = current_override(&app.id, config) {
        return vec![target];
    }
    app.category.iter().map(String::as_str).collect()
}

/// The override target for an app, if it is set and still points at a
/// category that exists (standard or a live custom one).
pub fn current_override<'a>(app_id: &str, config: &'a AppletConfig) -> Option<&'a str> {
    let target = config.app_category_overrides.get(app_id)?;
    category_exists(target, config).then_some(target.as_str())
}

fn category_exists(key: &str, config: &AppletConfig) -> bool {
    ApplicationCategory::STANDARD
        .iter()
        .any(|c| c.mime_name == key)
        || config
            .custom_categories
            .iter()
            .any(|c| ApplicationCategory::custom_key(&c.id) == key)
}

fn is_hidden(category: &ApplicationCategory, config: &AppletConfig) -> bool {
    config
        .hidden_categories
        .iter()
        .any(|key| key == category.key.as_ref())
}

/// Standard and custom categories, reordered per `config.category_order`;
/// keys absent from it keep their default relative order at the end.
fn ordered_rest(config: &AppletConfig) -> Vec<ApplicationCategory> {
    let mut rest: Vec<ApplicationCategory> = ApplicationCategory::STANDARD
        .into_iter()
        .chain(
            config
                .custom_categories
                .iter()
                .map(ApplicationCategory::from_custom),
        )
        .collect();
    apply_order(&mut rest, &config.category_order);
    rest
}

fn apply_order(categories: &mut Vec<ApplicationCategory>, order: &[String]) {
    let mut ordered = Vec::with_capacity(categories.len());
    for key in order {
        if let Some(pos) = categories
            .iter()
            .position(|c| c.key.as_ref() == key.as_str())
        {
            ordered.push(categories.remove(pos));
        }
    }
    ordered.append(categories);
    *categories = ordered;
}

/// Whether at least one pinned app is installed and not hidden.
pub fn has_favorites(config: &AppletConfig, all_apps: &[Arc<ApplicationEntry>]) -> bool {
    let favorites = resolve_favorites(&config.pinned_apps, all_apps);
    !without_hidden(&favorites, &config.hidden_apps).is_empty()
}

/// Categories to show in the popup, in final order: permanent categories
/// first (Favorites only when non-empty, Recently used unless hidden, All
/// always), then standard categories with at least one visible app, then
/// custom categories (shown even if empty), minus anything hidden.
pub fn resolve_categories(
    apps: &[Arc<ApplicationEntry>],
    config: &AppletConfig,
    has_favorites: bool,
) -> Vec<ApplicationCategory> {
    let used: HashSet<&str> = apps
        .iter()
        .filter(|app| !config.hidden_apps.contains(&app.id))
        .flat_map(|app| effective_categories(app, config))
        .collect();

    let mut categories: Vec<ApplicationCategory> = ApplicationCategory::PERMANENT
        .into_iter()
        .filter(|c| {
            if *c == ApplicationCategory::FAVORITES {
                has_favorites
            } else if *c == ApplicationCategory::ALL {
                true // All applications can never be hidden
            } else {
                !is_hidden(c, config)
            }
        })
        .collect();

    categories.extend(
        ordered_rest(config)
            .into_iter()
            .filter(|c| !is_hidden(c, config))
            .filter(|c| c.kind == CategoryKind::Custom || used.contains(c.mime_name)),
    );

    categories
}

pub fn default_category(has_favorites: bool) -> ApplicationCategory {
    if has_favorites {
        ApplicationCategory::FAVORITES
    } else {
        ApplicationCategory::ALL
    }
}

/// Keeps the selection if it is still shown; otherwise falls back to the
/// default category. `None` (no category selected, e.g. mid-search) stays
/// `None`.
pub fn category_after_change(
    selected: Option<ApplicationCategory>,
    available: &[ApplicationCategory],
    has_favorites: bool,
) -> Option<ApplicationCategory> {
    let selected = selected?;
    if available.iter().any(|a| a.key == selected.key) {
        Some(selected)
    } else {
        Some(default_category(has_favorites))
    }
}

fn without_hidden(apps: &[Arc<ApplicationEntry>], hidden: &[String]) -> Vec<Arc<ApplicationEntry>> {
    apps.iter()
        .filter(|app| !hidden.contains(&app.id))
        .cloned()
        .collect()
}

/// Apps not hidden and, for a search filter, matching the filter; used to
/// keep hidden apps out of fuzzy search too.
pub fn visible_apps(
    apps: Vec<Arc<ApplicationEntry>>,
    config: &AppletConfig,
) -> Vec<Arc<ApplicationEntry>> {
    without_hidden(&apps, &config.hidden_apps)
}

/// Apps to show for a selected category.
pub fn apps_of_category(
    category: &ApplicationCategory,
    all_apps: &[Arc<ApplicationEntry>],
    recent_apps: &[Arc<ApplicationEntry>],
    config: &AppletConfig,
) -> Vec<Arc<ApplicationEntry>> {
    if *category == ApplicationCategory::ALL {
        without_hidden(all_apps, &config.hidden_apps)
    } else if *category == ApplicationCategory::RECENTLY_USED {
        without_hidden(recent_apps, &config.hidden_apps)
    } else if *category == ApplicationCategory::FAVORITES {
        without_hidden(
            &resolve_favorites(&config.pinned_apps, all_apps),
            &config.hidden_apps,
        )
    } else {
        without_hidden(all_apps, &config.hidden_apps)
            .into_iter()
            .filter(|app| effective_categories(app, config).contains(&category.key.as_ref()))
            .collect()
    }
}

/// Categories offered in the "Move to..." submenu: standard and custom,
/// minus hidden ones, in display order.
pub fn move_targets(config: &AppletConfig) -> Vec<ApplicationCategory> {
    ordered_rest(config)
        .into_iter()
        .filter(|c| !is_hidden(c, config))
        .collect()
}

/// Every category (including hidden and unused ones) with its visibility,
/// in display order; used by the settings page.
pub fn editor_rows(config: &AppletConfig) -> Vec<(ApplicationCategory, bool)> {
    let mut rows: Vec<(ApplicationCategory, bool)> = ApplicationCategory::PERMANENT
        .into_iter()
        .map(|c| {
            let visible = c == ApplicationCategory::ALL || !is_hidden(&c, config);
            (c, visible)
        })
        .collect();

    rows.extend(ordered_rest(config).into_iter().map(|c| {
        let visible = !is_hidden(&c, config);
        (c, visible)
    }));
    rows
}

/// Swap the category with its neighbour in the standard+custom order;
/// materializes the full order into `config.category_order`. No-op at the
/// edges or for permanent categories.
pub fn move_category(config: &mut AppletConfig, key: &str, direction: MoveDirection) {
    let mut order: Vec<String> = ordered_rest(config)
        .into_iter()
        .map(|c| c.key.into_owned())
        .collect();
    let Some(index) = order.iter().position(|k| k == key) else {
        return;
    };
    let target = match direction {
        MoveDirection::Up => index.checked_sub(1),
        MoveDirection::Down => (index + 1 < order.len()).then_some(index + 1),
    };
    if let Some(target) = target {
        order.swap(index, target);
    }
    config.category_order = order;
}

/// Toggle a category's visibility; a no-op for "All applications", which can
/// never be hidden.
pub fn toggle_category_visibility(config: &mut AppletConfig, key: &str) {
    if key == ApplicationCategory::ALL.key.as_ref() {
        return;
    }
    if let Some(pos) = config.hidden_categories.iter().position(|k| k == key) {
        config.hidden_categories.remove(pos);
    } else {
        config.hidden_categories.push(key.to_string());
    }
}

/// Next free custom category id ("c1", "c2", ...).
pub fn next_custom_id(existing: &[CustomCategory]) -> String {
    let next = existing
        .iter()
        .filter_map(|c| c.id.strip_prefix('c').and_then(|n| n.parse::<u32>().ok()))
        .max()
        .map_or(1, |n| n + 1);
    format!("c{next}")
}

/// Deletes a custom category and every reference to it: overrides pointing
/// at it fall back to the app's original categories, and it is dropped from
/// hidden and ordering lists.
pub fn remove_custom_category(config: &mut AppletConfig, id: &str) {
    let key = ApplicationCategory::custom_key(id);
    config.custom_categories.retain(|c| c.id != id);
    config
        .app_category_overrides
        .retain(|_, target| *target != key);
    config.hidden_categories.retain(|k| k != &key);
    config.category_order.retain(|k| k != &key);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, categories: &[&str]) -> Arc<ApplicationEntry> {
        Arc::new(ApplicationEntry {
            name: id.to_string(),
            generic_name: None,
            id: id.to_string(),
            icon: None,
            comment: None,
            exec: None,
            category: categories.iter().map(|c| c.to_string()).collect(),
            is_terminal: false,
            item_id: cosmic::widget::Id::unique(),
            desktop_actions: vec![],
        })
    }

    fn custom(id: &str, name: &str) -> CustomCategory {
        CustomCategory {
            id: id.to_string(),
            name: name.to_string(),
            icon: "folder-symbolic".to_string(),
        }
    }

    fn keys(categories: &[ApplicationCategory]) -> Vec<String> {
        categories.iter().map(|c| c.key.to_string()).collect()
    }

    #[test]
    fn resolve_shows_permanent_then_used_standard_then_custom() {
        let config = AppletConfig::default();
        let apps = vec![entry("a", &["Audio"])];
        let got = resolve_categories(&apps, &config, false);
        assert_eq!(keys(&got), ["all-applications", "recently-used", "Audio"]);
    }

    #[test]
    fn favorites_shown_only_when_non_empty() {
        let config = AppletConfig::default();
        let apps = vec![entry("a", &[])];
        let with = resolve_categories(&apps, &config, true);
        assert_eq!(keys(&with)[0], "favorites");
        let without = resolve_categories(&apps, &config, false);
        assert!(!keys(&without).contains(&"favorites".to_string()));
    }

    #[test]
    fn hidden_standard_category_is_dropped() {
        let mut config = AppletConfig::default();
        config.hidden_categories.push("Audio".to_string());
        let apps = vec![entry("a", &["Audio"])];
        let got = resolve_categories(&apps, &config, false);
        assert!(!keys(&got).contains(&"Audio".to_string()));
    }

    #[test]
    fn recently_used_can_be_hidden_but_all_applications_cannot() {
        let mut config = AppletConfig::default();
        config.hidden_categories.push("recently-used".to_string());
        config
            .hidden_categories
            .push("all-applications".to_string());
        let got = resolve_categories(&[], &config, false);
        assert_eq!(keys(&got), ["all-applications"]);
    }

    #[test]
    fn custom_category_shown_even_when_empty() {
        let mut config = AppletConfig::default();
        config.custom_categories.push(custom("c1", "Work"));
        let got = resolve_categories(&[], &config, false);
        assert!(keys(&got).contains(&"custom:c1".to_string()));
    }

    #[test]
    fn override_moves_app_out_of_original_category() {
        let mut config = AppletConfig::default();
        config
            .app_category_overrides
            .insert("a".to_string(), "Video".to_string());
        let apps = vec![entry("a", &["Audio"])];
        assert_eq!(effective_categories(&apps[0], &config), vec!["Video"]);
        let audio = apps_of_category(&ApplicationCategory::AUDIO, &apps, &[], &config);
        assert!(audio.is_empty());
        let video = apps_of_category(&ApplicationCategory::VIDEO, &apps, &[], &config);
        assert_eq!(video.len(), 1);
        // All applications and search still see it.
        assert_eq!(
            apps_of_category(&ApplicationCategory::ALL, &apps, &[], &config).len(),
            1
        );
    }

    #[test]
    fn override_pointing_at_deleted_category_falls_back_to_original() {
        let mut config = AppletConfig::default();
        config
            .app_category_overrides
            .insert("a".to_string(), "custom:gone".to_string());
        let apps = vec![entry("a", &["Audio"])];
        assert_eq!(effective_categories(&apps[0], &config), vec!["Audio"]);
    }

    #[test]
    fn standard_category_disappears_once_its_only_app_is_hidden() {
        let mut config = AppletConfig::default();
        config.hidden_apps.push("a".to_string());
        let apps = vec![entry("a", &["Audio"])];
        let got = resolve_categories(&apps, &config, false);
        assert!(!keys(&got).contains(&"Audio".to_string()));
    }

    #[test]
    fn hidden_app_is_excluded_from_all_applications() {
        let mut config = AppletConfig::default();
        config.hidden_apps.push("a".to_string());
        let apps = vec![entry("a", &[]), entry("b", &[])];
        let all = apps_of_category(&ApplicationCategory::ALL, &apps, &[], &config);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "b");
    }

    #[test]
    fn visible_apps_drops_hidden_ones() {
        let mut config = AppletConfig::default();
        config.hidden_apps.push("a".to_string());
        let apps = vec![entry("a", &[]), entry("b", &[])];
        let got = visible_apps(apps, &config);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].id, "b");
    }

    #[test]
    fn category_order_reorders_unlisted_keys_stay_after() {
        let mut config = AppletConfig::default();
        config.category_order = vec!["Video".to_string(), "Audio".to_string()];
        let apps = vec![
            entry("a", &["Audio"]),
            entry("b", &["Video"]),
            entry("c", &["Game"]),
        ];
        let got = resolve_categories(&apps, &config, false);
        assert_eq!(
            keys(&got),
            [
                "all-applications",
                "recently-used",
                "Video",
                "Audio",
                "Game"
            ]
        );
    }

    #[test]
    fn move_category_swaps_with_neighbour() {
        let mut config = AppletConfig::default();
        config.custom_categories.push(custom("c1", "Work"));
        let mut order: Vec<String> = ordered_rest(&config)
            .into_iter()
            .map(|c| c.key.into_owned())
            .collect();
        assert_eq!(order.last().unwrap(), "custom:c1");
        move_category(&mut config, "custom:c1", MoveDirection::Up);
        order = config.category_order.clone();
        let pos = order.iter().position(|k| k == "custom:c1").unwrap();
        assert!(pos < order.len() - 1);
    }

    #[test]
    fn move_category_is_noop_at_edges_and_unknown_key() {
        let mut config = AppletConfig::default();
        move_category(&mut config, "Audio", MoveDirection::Up);
        assert!(config.category_order.is_empty() || config.category_order[0] == "Audio");
        move_category(&mut config, "does-not-exist", MoveDirection::Up);
    }

    #[test]
    fn toggle_category_visibility_flips_and_ignores_all() {
        let mut config = AppletConfig::default();
        toggle_category_visibility(&mut config, "Audio");
        assert_eq!(config.hidden_categories, ["Audio"]);
        toggle_category_visibility(&mut config, "Audio");
        assert!(config.hidden_categories.is_empty());
        toggle_category_visibility(&mut config, "all-applications");
        assert!(config.hidden_categories.is_empty());
    }

    #[test]
    fn next_custom_id_picks_first_free_slot() {
        assert_eq!(next_custom_id(&[]), "c1");
        assert_eq!(
            next_custom_id(&[custom("c1", "a"), custom("c3", "b")]),
            "c4"
        );
    }

    #[test]
    fn remove_custom_category_clears_all_references() {
        let mut config = AppletConfig::default();
        config.custom_categories.push(custom("c1", "Work"));
        config
            .app_category_overrides
            .insert("a".to_string(), "custom:c1".to_string());
        config.hidden_categories.push("custom:c1".to_string());
        config.category_order.push("custom:c1".to_string());

        remove_custom_category(&mut config, "c1");

        assert!(config.custom_categories.is_empty());
        assert!(config.app_category_overrides.is_empty());
        assert!(config.hidden_categories.is_empty());
        assert!(config.category_order.is_empty());
    }

    #[test]
    fn has_favorites_ignores_hidden_pinned_apps() {
        let mut config = AppletConfig::default();
        config.pinned_apps.push("a".to_string());
        config.hidden_apps.push("a".to_string());
        let apps = vec![entry("a", &[])];
        assert!(!has_favorites(&config, &apps));
    }

    #[test]
    fn editor_rows_include_hidden_categories() {
        let mut config = AppletConfig::default();
        config.hidden_categories.push("Audio".to_string());
        let rows = editor_rows(&config);
        let audio_row = rows
            .iter()
            .find(|(c, _)| c.key.as_ref() == "Audio")
            .unwrap();
        assert!(!audio_row.1);
        let all_row = rows
            .iter()
            .find(|(c, _)| c == &ApplicationCategory::ALL)
            .unwrap();
        assert!(all_row.1);
    }

    #[test]
    fn move_targets_excludes_hidden() {
        let mut config = AppletConfig::default();
        config.hidden_categories.push("Audio".to_string());
        let targets = move_targets(&config);
        assert!(!targets.iter().any(|c| c.key.as_ref() == "Audio"));
        assert!(targets.iter().any(|c| c.key.as_ref() == "Video"));
    }
}
