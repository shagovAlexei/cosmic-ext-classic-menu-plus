# Favorites Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A "Favorites" category (first in the list, opened by default when non-empty) filled through a right-click checkbox "Add to favorites", with a settings page to reorder and unpin.

**Architecture:** A new pure module `applet/src/model/favorites.rs` owns ordering/toggling/moving of `pinned_apps` (a list of desktop ids). `AppletConfig` gets one additive field `pinned_apps`. `ApplicationCategory::FAVORITES` is a new permanent category shown only when something is pinned. `logic/apps.rs` gets pure helpers (`build_categories`, `default_category`, `resolve_favorites`) that `load_app_categories`/`get_apps_of_category` call. The two duplicated context-menu builders in `applet.rs` collapse into one `build_app_context_menu` in `applet_menu.rs`, which gains the checkbox. The settings crate gets a fourth nav page.

**Tech Stack:** Rust, libcosmic (`nav_bar`, `button::icon`), cosmic-config, Fluent (`fl!`).

**Spec:** `docs/PLAN.md`, section "4. Избранное" and stage 4 TODO.

## Global Constraints

- Merge-friendly with upstream: additive config fields, no reformatting of untouched code (do not run `cargo fmt` over whole files).
- New config field must have a `Default` (empty) so existing installs keep loading.
- Category key `favorites`, icon `starred-symbolic` (bundled SVG in `res/icons/bundled/`, rendered `.symbolic(true)`).
- Order of the Favorites list = order of `pinned_apps`. Ids of apps no longer installed are silently skipped in the menu (kept in config; shown by id in settings so they can be unpinned).
- On open: `Favorites` if `pinned_apps` resolves to at least one installed app, otherwise `All applications`.
- The checkbox "Add to favorites" is independent of the existing "Pin to panel" (that one writes the dock's `AppListConfig`).
- i18n: every new string in `en` and `ru`.
- Do not tick boxes in `docs/PLAN.md` (the user ticks after manual verification). Do not push, open a PR or merge; the user decides.

## Review Focus

- `pinned_apps` contains an uninstalled app: skipped in the menu, no panic; the category still appears if at least one pinned app is installed, and does not if none is. Pinned by Task 1 `resolve_favorites` tests and Task 2 `build_categories` tests.
- Duplicate ids in hand-edited config: shown once. Pinned by Task 1 test.
- Unpinning the last favorite while the Favorites category is selected: the category vanishes, so the view must fall back to All applications. Pinned by Task 2 `category_after_change` test.
- Search with Favorites selected/default: search still runs over all apps (unchanged). Manual check in Task 4.
- Keyboard selection index after the list changes (pin/unpin, settings edit): reset to `None`. Manual check in Task 4.

---

### Task 1: Favorites model, config field, category, icon

**Files:**
- Create: `applet/src/model/favorites.rs`, `res/icons/bundled/starred-symbolic.svg`
- Modify: `applet/src/model/mod.rs`, `applet/src/model/application_category.rs`, `applet/src/config.rs`, `applet/i18n/en/cosmic_ext_classic_menu_plus_applet.ftl`, `applet/i18n/ru/cosmic_ext_classic_menu_plus_applet.ftl`

**Interfaces:**
- Consumes: `model::power_action::MoveDirection` (existing, `Up`/`Down`), `model::application_entry::ApplicationEntry`.
- Produces:
  - `favorites::resolve_favorites(pinned: &[String], all: &[Arc<ApplicationEntry>]) -> Vec<Arc<ApplicationEntry>>`
  - `favorites::toggle_pinned(pinned: &mut Vec<String>, app_id: &str) -> bool` (returns `true` if the app is pinned after the call)
  - `favorites::move_pinned(pinned: &mut Vec<String>, app_id: &str, direction: MoveDirection)`
  - `ApplicationCategory::FAVORITES`, config field `AppletConfig::pinned_apps: Vec<String>`
  - i18n keys `favorites` and `add-to-favorites` (applet crate)

- [ ] **Step 1: Copy the icon**

```bash
cp /usr/share/icons/Adwaita/symbolic/status/starred-symbolic.svg res/icons/bundled/starred-symbolic.svg
```

- [ ] **Step 2: Write the failing tests** — create `applet/src/model/favorites.rs` with only the tests and the signatures stubbed with `todo!()`:

```rust
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
    todo!()
}

/// Unpin the app if it is pinned, otherwise pin it at the end. Returns whether
/// it is pinned afterwards.
pub fn toggle_pinned(pinned: &mut Vec<String>, app_id: &str) -> bool {
    todo!()
}

/// Swap the app with its neighbour; no-op at the edges or if it is not pinned.
pub fn move_pinned(pinned: &mut Vec<String>, app_id: &str, direction: MoveDirection) {
    todo!()
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
```

Add `pub mod favorites;` to `applet/src/model/mod.rs`.

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet favorites 2>&1 | tail -20`
Expected: 7 tests FAIL with `not yet implemented`.

- [ ] **Step 4: Implement** (replace the three `todo!()` bodies)

```rust
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
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet favorites 2>&1 | tail -15`
Expected: 7 passed.

- [ ] **Step 6: Config field, category, strings**

In `applet/src/config.rs` add to the struct (after `list_density`) `pub pinned_apps: Vec<String>,` and to `Default`: `pinned_apps: vec![],`.

In `applet/src/model/application_category.rs` add before `ALL`:

```rust
    pub const FAVORITES: ApplicationCategory = ApplicationCategory {
        display_name: "favorites",
        icon_svg_bytes: include_bytes!("../../../res/icons/bundled/starred-symbolic.svg"),
        mime_name: "",
        permanent: true,
    };
```

and in `get_display_name` add `"favorites" => fl!("favorites"),`.

`applet/i18n/en/...applet.ftl`: `favorites=Favorites` and `add-to-favorites=Add to favorites`. `ru`: `favorites=Избранное` and `add-to-favorites=В избранное`.

- [ ] **Step 7: Full suite and commit**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet 2>&1 | tail -5` and `cargo build -p cosmic-ext-classic-menu-plus-applet 2>&1 | tail -3`
Expected: 26 passed (19 + 7); build OK.

```bash
git add applet/src res applet/i18n
git commit -m "feat(favorites): pinned_apps config, FAVORITES category and ordering helpers"
```

---

### Task 2: Categories, default category and live refresh

**Files:**
- Modify: `applet/src/logic/apps.rs`, `applet/src/applet.rs`, `applet/src/applet_menu.rs`
- Test: `applet/src/logic/apps.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: Task 1 `resolve_favorites`, `ApplicationCategory::FAVORITES`, `AppletConfig::pinned_apps`.
- Produces:
  - `apps::build_categories(used: &HashSet<String>, has_favorites: bool) -> Vec<ApplicationCategory>` (pure)
  - `apps::default_category(has_favorites: bool) -> ApplicationCategory` (pure)
  - `apps::category_after_change(selected: Option<ApplicationCategory>, has_favorites: bool) -> Option<ApplicationCategory>` (pure: a selected `FAVORITES` becomes `ALL` when nothing is left to show)
  - `load_app_categories()` and `get_apps_of_category(category)` keep their signatures and read `pinned_apps` from `AppletConfig::config()` (same approach as `get_recent_applications`).
  - `Applet::refresh_favorites_view(&mut self) -> Task<Message>`

- [ ] **Step 1: Write the failing tests** — append to `applet/src/logic/apps.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn used(names: &[&str]) -> std::collections::HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn names(categories: &[ApplicationCategory]) -> Vec<&'static str> {
        categories.iter().map(|c| c.display_name).collect()
    }

    #[test]
    fn favorites_come_first_only_when_there_are_some() {
        let with = build_categories(&used(&["Audio"]), true);
        assert_eq!(names(&with), ["favorites", "all-applications", "recently-used", "audio"]);
        let without = build_categories(&used(&["Audio"]), false);
        assert_eq!(names(&without), ["all-applications", "recently-used", "audio"]);
    }

    #[test]
    fn unused_mime_categories_are_dropped() {
        let got = build_categories(&used(&[]), false);
        assert_eq!(names(&got), ["all-applications", "recently-used"]);
    }

    #[test]
    fn default_category_follows_favorites() {
        assert_eq!(default_category(true), ApplicationCategory::FAVORITES);
        assert_eq!(default_category(false), ApplicationCategory::ALL);
    }

    #[test]
    fn selected_favorites_falls_back_to_all_when_empty() {
        assert_eq!(
            category_after_change(Some(ApplicationCategory::FAVORITES), false),
            Some(ApplicationCategory::ALL)
        );
        assert_eq!(
            category_after_change(Some(ApplicationCategory::FAVORITES), true),
            Some(ApplicationCategory::FAVORITES)
        );
        assert_eq!(
            category_after_change(Some(ApplicationCategory::AUDIO), false),
            Some(ApplicationCategory::AUDIO)
        );
        assert_eq!(category_after_change(None, false), None);
    }
}
```

Stubs above the tests module (with `todo!()`) so it compiles:

```rust
pub fn build_categories(
    used: &std::collections::HashSet<String>,
    has_favorites: bool,
) -> Vec<ApplicationCategory> {
    todo!()
}
pub fn default_category(has_favorites: bool) -> ApplicationCategory { todo!() }
pub fn category_after_change(
    selected: Option<ApplicationCategory>,
    has_favorites: bool,
) -> Option<ApplicationCategory> { todo!() }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet logic::apps 2>&1 | tail -15`
Expected: 4 tests FAIL with `not yet implemented`.

- [ ] **Step 3: Implement** — replace the stubs and rewrite `load_app_categories` / `get_apps_of_category`:

```rust
pub fn build_categories(
    used: &std::collections::HashSet<String>,
    has_favorites: bool,
) -> Vec<ApplicationCategory> {
    let mut apps_categories = vec![];
    if has_favorites {
        apps_categories.push(ApplicationCategory::FAVORITES);
    }
    apps_categories.extend([
        ApplicationCategory::ALL,
        ApplicationCategory::RECENTLY_USED,
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
    ]);

    // Filter only available ones
    apps_categories
        .into_iter()
        .filter(|x| {
            x.permanent == true
                || (!x.mime_name.is_empty() && used.contains(&x.mime_name.to_string()))
        })
        .collect()
}

pub fn default_category(has_favorites: bool) -> ApplicationCategory {
    if has_favorites {
        ApplicationCategory::FAVORITES
    } else {
        ApplicationCategory::ALL
    }
}

pub fn category_after_change(
    selected: Option<ApplicationCategory>,
    has_favorites: bool,
) -> Option<ApplicationCategory> {
    match selected {
        Some(c) if c == ApplicationCategory::FAVORITES && !has_favorites => {
            Some(ApplicationCategory::ALL)
        }
        other => other,
    }
}

/// Whether at least one pinned app is installed.
pub fn has_favorites(pinned: &[String]) -> bool {
    !resolve_favorites(pinned, &load_apps()).is_empty()
}

pub fn load_app_categories() -> Vec<ApplicationCategory> {
    use std::collections::HashSet;

    log::info!("Loading app categories...");
    let all_apps = load_apps();
    let used_categories: HashSet<String> =
        all_apps.iter().flat_map(|app| app.category.clone()).collect();
    build_categories(&used_categories, has_favorites(&AppletConfig::config().pinned_apps))
}
```

In `get_apps_of_category` add a branch before the `else`:

```rust
    } else if category == ApplicationCategory::FAVORITES {
        resolve_favorites(&AppletConfig::config().pinned_apps, &load_apps())
    } else {
```

Add `favorites::resolve_favorites` to the `use crate::model::...` imports.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet 2>&1 | tail -5`
Expected: 30 passed (26 + 4).

- [ ] **Step 5: Wire the applet** in `applet/src/applet.rs`:

1. `toggle_popup`: replace
   `self.selected_category = Some(ApplicationCategory::ALL);` and `self.available_applications = load_apps();` with
   ```rust
   let category = crate::logic::apps::default_category(crate::logic::apps::has_favorites(
       &self.config.pinned_apps,
   ));
   self.selected_category = Some(category.clone());
   self.available_applications = crate::logic::apps::get_apps_of_category(category.clone());
   ```
   and in the `MainMenu` branch change the first task from `load_apps()` to
   `crate::logic::apps::get_apps_of_category(category)` (move `category` into the closure: `let category = category.clone();` before `tasks.push`).
2. Add the method:
   ```rust
   /// Reload categories and, if Favorites is shown, its list. Called after
   /// `pinned_apps` changed (context menu or settings).
   fn refresh_favorites_view(&mut self) -> Task<Message> {
       let has = crate::logic::apps::has_favorites(&self.config.pinned_apps);
       self.selected_category =
           crate::logic::apps::category_after_change(self.selected_category.take(), has);
       self.selected_item_index = None;
       let mut tasks = vec![Task::perform(
           tokio::task::spawn_blocking(|| crate::logic::apps::load_app_categories()),
           |res| cosmic::action::app(Message::UpdateAvailableCategories(res.unwrap())),
       )];
       if let Some(category) = self.selected_category.clone() {
           if category == ApplicationCategory::FAVORITES || category == ApplicationCategory::ALL {
               tasks.push(Task::perform(
                   tokio::task::spawn_blocking(move || {
                       crate::logic::apps::get_apps_of_category(category)
                   }),
                   |res| cosmic::action::app(Message::UpdateAvailableApplications(res.unwrap())),
               ));
           }
       }
       Task::batch(tasks)
   }
   ```
   (The list is reloaded for `ALL` too: it covers the fallback case; it is a cheap cached read.)
3. `Message::UpdateConfig`: after `self.config = config;` return
   `if self.popup.is_some() { self.refresh_favorites_view() } else { Task::none() }`.
   Note: read the previous `pinned_apps` first and only refresh when it changed:
   `let changed = self.config.pinned_apps != config.pinned_apps;` before the assignment, and refresh only `if changed && self.popup.is_some()`. This avoids a reload after every launch (recent apps also write the config).

- [ ] **Step 6: Divider position** in `applet/src/applet_menu.rs::create_categories_pane`: replace `categories_pane.insert(2, horizontal_divider);` with

```rust
        let permanent_count = applet
            .available_categories
            .iter()
            .filter(|c| c.permanent)
            .count();
        if !categories_pane.is_empty() {
            categories_pane.insert(permanent_count, horizontal_divider);
        }
```
(and remove the old `if !categories_pane.is_empty()` wrapper line it replaces).

- [ ] **Step 7: Build, full suite, commit**

Run: `cargo build -p cosmic-ext-classic-menu-plus-applet 2>&1 | grep -E "^error" -A8 | head -30; cargo test -p cosmic-ext-classic-menu-plus-applet 2>&1 | tail -4`
Expected: build OK, 30 passed.

```bash
git add applet/src
git commit -m "feat(favorites): Favorites category, default on open, live refresh"
```

---

### Task 3: One context-menu builder and the "Add to favorites" checkbox

**Files:**
- Modify: `applet/src/applet_menu.rs`, `applet/src/applet.rs`

**Interfaces:**
- Consumes: Task 1 `toggle_pinned`, config `pinned_apps`; Task 2 `refresh_favorites_view`.
- Produces:
  - `AppletMenu::build_app_context_menu(app: &ApplicationEntry, app_index: usize, pinned_to_panel: bool, favorite: bool) -> Vec<menu::Tree<Message>>`
  - `ContextMenuAction::ToggleFavorite(usize)` and `Message::ToggleFavoriteAt(usize)`

This task changes UI construction only; the pure logic is covered by Task 1 tests. The verification is a clean build plus the manual check in Task 4.

- [ ] **Step 1: Add the builder** to `impl AppletMenu` in `applet_menu.rs`:

```rust
    /// Right-click menu of an app entry: launch, pin to panel, favorite, then
    /// the app's own desktop actions.
    pub fn build_app_context_menu(
        app: &ApplicationEntry,
        app_index: usize,
        pinned_to_panel: bool,
        favorite: bool,
    ) -> Vec<menu::Tree<Message>> {
        let mut buttons: Vec<menu::Item<ContextMenuAction, _>> = vec![
            menu::Item::Button(fl!("launch"), None, ContextMenuAction::LaunchApplication(app_index)),
            menu::Item::CheckBox(
                fl!("pin-to-panel"),
                None,
                pinned_to_panel,
                ContextMenuAction::PinToPanel(app_index, pinned_to_panel),
            ),
            menu::Item::CheckBox(
                fl!("add-to-favorites"),
                None,
                favorite,
                ContextMenuAction::ToggleFavorite(app_index),
            ),
        ];

        let actions: Vec<menu::Item<ContextMenuAction, _>> = app
            .desktop_actions
            .iter()
            .enumerate()
            .map(|(action_index, action)| {
                menu::Item::Button(
                    action.name.to_string(),
                    None,
                    ContextMenuAction::LaunchApplicationWithAction(app_index, action_index),
                )
            })
            .collect();
        if !actions.is_empty() {
            buttons.push(menu::Item::Divider);
            buttons.extend(actions);
        }

        menu::items(&std::collections::HashMap::new(), buttons)
    }
```

Add `use crate::model::application_entry::ApplicationEntry;` to the imports. Extend `ContextMenuAction` with `ToggleFavorite(usize)` and its `message()` arm `ContextMenuAction::ToggleFavorite(index) => Message::ToggleFavoriteAt(*index)`.

- [ ] **Step 2: Use it in `applet.rs`** (delete both duplicated builders):

In `UpdateAvailableApplications`, the loop body becomes:

```rust
                for (app_index, app) in self.available_applications.iter().enumerate() {
                    let trees = crate::applet_menu::AppletMenu::build_app_context_menu(
                        app,
                        app_index,
                        crate::logic::apps::is_app_in_favorites(app, &self.app_list_config),
                        self.config.pinned_apps.contains(&app.id),
                    );
                    self.context_menus.insert(app.id.clone(), trees);
                }
```

In `PinToAppTrayIndex`, keep the `app_list_config` add/remove, and replace the whole rebuild block (from `let new_is_favorites` to the `insert`) with:

```rust
                        let trees = crate::applet_menu::AppletMenu::build_app_context_menu(
                            &app,
                            app_index,
                            !favorites,
                            self.config.pinned_apps.contains(&app.id),
                        );
                        self.context_menus.insert(app.id.clone(), trees);
```

Add `ToggleFavoriteAt(usize)` to `Message` and this arm:

```rust
            Message::ToggleFavoriteAt(app_index) => {
                let Some(app) = self.available_applications.get(app_index).cloned() else {
                    return Task::none();
                };
                crate::model::favorites::toggle_pinned(&mut self.config.pinned_apps, &app.id);
                if let Some(handler) = AppletConfig::config_handler() {
                    if let Err(err) = self.config.write_entry(&handler) {
                        log::error!("failed to save pinned apps: {err}");
                    }
                }
                let trees = crate::applet_menu::AppletMenu::build_app_context_menu(
                    &app,
                    app_index,
                    crate::logic::apps::is_app_in_favorites(&app, &self.app_list_config),
                    self.config.pinned_apps.contains(&app.id),
                );
                self.context_menus.insert(app.id.clone(), trees);
                self.refresh_favorites_view()
            }
```

(`refresh_favorites_view` reloads the list when Favorites/All is selected; `UpdateAvailableApplications` then rebuilds every menu, including this one. Other categories keep their list and the rebuilt menu above.)

- [ ] **Step 3: Build, full suite, commit**

Run: `cargo build -p cosmic-ext-classic-menu-plus-applet 2>&1 | grep -E "^(error|warning: unused)" -A8 | head -30; cargo test -p cosmic-ext-classic-menu-plus-applet 2>&1 | tail -4`
Expected: no errors, no new unused warnings from these files, 30 passed.

```bash
git add applet/src
git commit -m "feat(favorites): add-to-favorites checkbox and single context-menu builder"
```

---

### Task 4: Settings page "Favorites" and final checks

**Files:**
- Modify: `settings/src/app.rs`, `settings/i18n/en/cosmic_ext_classic_menu_plus_settings.ftl`, `settings/i18n/ru/cosmic_ext_classic_menu_plus_settings.ftl`

**Interfaces:**
- Consumes: Task 1 `move_pinned`, `MoveDirection`, `AppletConfig::pinned_apps`; existing `logic::apps::load_apps`, `IconHandle`; existing `write_config` helper and `Page`/nav in `settings/src/app.rs`.
- Produces: `Page::Favorites`, `Message::FavoriteMoved(String, MoveDirection)`, `Message::FavoriteRemoved(String)`.

- [ ] **Step 1: Strings.** `en` settings ftl: `favorites = Favorites`, `favorites-empty = No favorite apps yet. Right-click an app in the menu and choose "Add to favorites".`, `favorite-remove = Remove`. `ru`: `favorites = Избранное`, `favorites-empty = Избранных приложений пока нет. Нажмите правой кнопкой на приложение в меню и выберите «В избранное».`, `favorite-remove = Убрать`.

- [ ] **Step 2: Page and state.**
  - `enum Page` gets `Favorites`; in `init`, after the power-buttons entry insert `.insert(|e| e.text(fl!("favorites")).icon(icon::from_name("starred-symbolic")).data(Page::Favorites))` (match the exact builder chain used by the other entries).
  - New field `apps: Vec<Arc<ApplicationEntry>>` on the app struct (import `std::sync::Arc` and `cosmic_ext_classic_menu_plus_applet::model::application_entry::{ApplicationEntry, IconHandle}`), initialised empty. It is filled in `on_nav_select` when the selected page is `Favorites` and `self.apps.is_empty()`: `self.apps = cosmic_ext_classic_menu_plus_applet::logic::apps::load_apps();` (cached; synchronous is acceptable, it runs once and only when the page is opened).
  - Messages `FavoriteMoved(String, MoveDirection)` and `FavoriteRemoved(String)`. `update` arms:
    ```rust
    Message::FavoriteMoved(id, direction) => {
        move_pinned(&mut self.config.pinned_apps, &id, direction);
        self.write_config("pinned apps");
        Task::none()
    }
    Message::FavoriteRemoved(id) => {
        self.config.pinned_apps.retain(|p| p != &id);
        self.write_config("pinned apps");
        Task::none()
    }
    ```
  - `view` match: `Some(Page::Favorites) => self.favorites_section().into(),`.

- [ ] **Step 3: The page.** Add next to `power_buttons_section`:

```rust
    fn favorites_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let space_xxs = cosmic::theme::active().cosmic().space_xxs();
        let pinned = &self.config.pinned_apps;
        let mut section = cosmic::widget::settings::section().title(fl!("favorites"));
        if pinned.is_empty() {
            return section.add(text::body(fl!("favorites-empty")));
        }
        for (index, id) in pinned.iter().enumerate() {
            let app = self.apps.iter().find(|a| &a.id == id);
            let name = app.map_or_else(|| id.clone(), |a| a.name.clone());
            let icon: Element<'_, Message> = match app.and_then(|a| a.icon.as_ref()) {
                Some(IconHandle::SvgHandle(h)) => cosmic::widget::svg(h.clone())
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .into(),
                Some(IconHandle::RasterHandle(h)) => cosmic::widget::image(h.clone())
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .into(),
                None => cosmic::widget::Space::new().width(24).height(24).into(),
            };
            let up = cosmic::widget::button::icon(icon::from_name("go-up-symbolic")).on_press_maybe(
                (index > 0).then(|| Message::FavoriteMoved(id.clone(), MoveDirection::Up)),
            );
            let down = cosmic::widget::button::icon(icon::from_name("go-down-symbolic"))
                .on_press_maybe((index + 1 < pinned.len()).then(|| {
                    Message::FavoriteMoved(id.clone(), MoveDirection::Down)
                }));
            let remove = cosmic::widget::button::standard(fl!("favorite-remove"))
                .on_press(Message::FavoriteRemoved(id.clone()));
            let controls = cosmic::iced::widget::row![up, down, remove]
                .spacing(space_xxs)
                .align_y(Alignment::Center);
            section = section.add(cosmic::widget::settings::item_row(vec![
                icon,
                text::body(name).width(Length::Fill).into(),
                controls.into(),
            ]));
        }
        section
    }
```

Adjust widget paths only if the compiler demands (the crate already uses these builders on the power-buttons page; copy its exact `item_row`/`settings::item` form if `item_row` differs). Import `move_pinned` from `...::model::favorites`.

- [ ] **Step 4: Build and test**

Run: `cargo build --release -p cosmic-ext-classic-menu-plus-settings 2>&1 | grep -E "^(error|warning: unused)" -A8 | head -30; cargo test 2>&1 | grep -E "test result|FAILED"`
Expected: no errors; all suites pass (applet 30).

- [ ] **Step 5: Release build of both crates**

Run: `just build-release 2>&1 | tail -3`
Expected: finishes without errors.

- [ ] **Step 6: Commit**

```bash
git add settings
git commit -m "feat(settings): Favorites page with reorder and remove"
```

- [ ] **Step 7: Manual check list for the user** (do not tick `PLAN.md`; report the list in the final message):
  1. `! sudo just install`, `! killall cosmic-panel`.
  2. No favorites yet: the menu opens on "All applications"; there is no Favorites category.
  3. Right-click an app: the menu has "Add to favorites" (checked state reflects reality). Tick it: "Favorites" appears first in the categories, with a divider after the three service categories.
  4. Close and reopen the menu: it opens on Favorites. Pin several apps: order equals pinning order.
  5. Untick the last favorite while Favorites is selected: the view switches to "All applications", the category disappears.
  6. Settings, Favorites page: ↑/↓ reorder and "Remove" work and apply to an open menu without restart.
  7. Type in the search field with Favorites selected: results come from all apps; clearing the search returns to "All applications" (known and accepted).
  8. Arrow keys after a pin/unpin: selection starts from the top again, no stale highlight.
  9. Config sanity: `pinned_apps` holding an id of a removed app does not break the menu.
