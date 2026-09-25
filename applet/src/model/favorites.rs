// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use crate::model::application_entry::ApplicationEntry;
use crate::model::power_action::MoveDirection;

/// Installed apps for the pinned ids, in pinned order; unknown ids and
/// duplicates are skipped.
pub fn resolve_favorites(
    pinned: &[String],
    all: &[Arc<ApplicationEntry>],
) -> Vec<Arc<ApplicationEntry>> {
    let mut seen: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for id in pinned {
        if seen.contains(&id.as_str()) {
            continue;
        }
        if let Some(app) = all.iter().find(|a| &a.id == id) {
            seen.push(id.as_str());
            out.push(app.clone());
        }
    }
    out
}

/// Unpin the app if it is pinned, otherwise pin it at the end. Returns whether
/// it is pinned afterwards.
pub fn toggle_pinned(pinned: &mut Vec<String>, app_id: &str) -> bool {
    let before = pinned.len();
    pinned.retain(|id| id != app_id);
    if pinned.len() == before {
        pinned.push(app_id.to_string());
        true
    } else {
        false
    }
}

/// Swap the app with its neighbour; no-op at the edges or if it is not pinned.
pub fn move_pinned(pinned: &mut Vec<String>, app_id: &str, direction: MoveDirection) {
    let Some(index) = pinned.iter().position(|id| id == app_id) else {
        return;
    };
    let target = match direction {
        MoveDirection::Up => index.checked_sub(1),
        MoveDirection::Down => (index + 1 < pinned.len()).then_some(index + 1),
    };
    if let Some(target) = target {
        pinned.swap(index, target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn entry(id: &str) -> Arc<ApplicationEntry> {
        Arc::new(ApplicationEntry {
            name: id.to_string(),
            generic_name: None,
            id: id.to_string(),
            icon: None,
            comment: None,
            exec: None,
            category: vec![],
            is_terminal: false,
            item_id: cosmic::widget::Id::unique(),
            desktop_actions: vec![],
        })
    }

    fn ids(apps: &[Arc<ApplicationEntry>]) -> Vec<&str> {
        apps.iter().map(|a| a.id.as_str()).collect()
    }

    fn pinned(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn resolve_keeps_pinned_order_not_alphabetical() {
        let all = vec![entry("a"), entry("b"), entry("c")];
        let got = resolve_favorites(&pinned(&["c", "a"]), &all);
        assert_eq!(ids(&got), ["c", "a"]);
    }

    #[test]
    fn resolve_skips_uninstalled_apps() {
        let all = vec![entry("a"), entry("b")];
        let got = resolve_favorites(&pinned(&["gone", "b", "also-gone"]), &all);
        assert_eq!(ids(&got), ["b"]);
    }

    #[test]
    fn resolve_shows_a_duplicate_id_once() {
        let all = vec![entry("a"), entry("b")];
        let got = resolve_favorites(&pinned(&["a", "b", "a"]), &all);
        assert_eq!(ids(&got), ["a", "b"]);
    }

    #[test]
    fn resolve_of_empty_pinned_is_empty() {
        let all = vec![entry("a")];
        assert!(resolve_favorites(&[], &all).is_empty());
    }

    #[test]
    fn toggle_pins_at_the_end_then_unpins() {
        let mut p = pinned(&["a"]);
        assert!(toggle_pinned(&mut p, "b"));
        assert_eq!(p, ["a", "b"]);
        assert!(!toggle_pinned(&mut p, "a"));
        assert_eq!(p, ["b"]);
    }

    #[test]
    fn move_swaps_with_neighbour() {
        let mut p = pinned(&["a", "b", "c"]);
        move_pinned(&mut p, "b", MoveDirection::Up);
        assert_eq!(p, ["b", "a", "c"]);
        move_pinned(&mut p, "b", MoveDirection::Down);
        assert_eq!(p, ["a", "b", "c"]);
    }

    #[test]
    fn move_is_a_noop_at_the_edges_and_for_unknown_ids() {
        let mut p = pinned(&["a", "b"]);
        move_pinned(&mut p, "a", MoveDirection::Up);
        move_pinned(&mut p, "b", MoveDirection::Down);
        move_pinned(&mut p, "zzz", MoveDirection::Up);
        assert_eq!(p, ["a", "b"]);
    }
}
