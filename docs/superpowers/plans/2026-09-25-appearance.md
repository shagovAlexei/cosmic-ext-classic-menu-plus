# Appearance (popup size, icon size, list density) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the user set popup width/height, app icon size and list density (Compact/Normal), and move the settings app to a `nav_bar` with pages General / Appearance / Power buttons.

**Architecture:** A new pure module `applet/src/model/appearance.rs` owns the constants, clamping and the single `item_height` formula. Four additive fields in `AppletConfig` feed it. The applet reads them in `applet_menu.rs` (popup size), `widgets/virtualized_app_list.rs` (rendering) and `applet.rs` (arrow-key scroll math) through one `Applet::list_item_height()`. The settings crate gets a `nav_bar::Model` and an Appearance page.

**Tech Stack:** Rust, libcosmic (`spin_button`, `dropdown`, `nav_bar`), cosmic-config, Fluent (`fl!`).

**Spec:** `docs/PLAN.md`, section "3. Размеры и иконки" and stage 3 TODO.

## Global Constraints

- Merge-friendly with upstream: additive config fields, no reformatting of untouched code (do not run `cargo fmt` over whole files).
- New config fields must have `Default` values so existing installs keep loading.
- Ranges: popup width 500–1200, popup height 400–1200, icon size 16–64.
- Defaults reproduce today's look: width 600, height 700, icon size 24 (= COSMIC default `space_l`), density `Normal`.
- Item height is computed in exactly one place (`appearance::item_height`); rendering and scroll math both use it.
- i18n: every new string in `en` and `ru`.
- Do not tick boxes in `docs/PLAN.md` (the user ticks after manual verification). Do not push or open a PR; the user decides.
- libcosmic here is built with `a11y`, so `spin_button(label, name, value, step, min, max, on_press)` takes an extra `name` argument.

## Review Focus

- Hand-edited out-of-range config (`popup_width = 0` or `99999`, `app_icon_size = 0`): clamped on use. Pinned by Task 1 clamp tests.
- Huge icon with Compact density: the row grows so the icon is not clipped. Pinned by Task 1 `item_height` tests.
- Default settings render exactly as before (`Normal`, icon 24, `space_xl` 32 gives height 32). Pinned by Task 1 test.
- `Compact` hides comments even when the theme is spacious. Pinned by Task 1 `show_comment` test.
- A popup taller than the screen at height 1200 is not prevented by us; covered by the manual check in Task 4.

---

### Task 1: Appearance model and config fields

**Files:**
- Create: `applet/src/model/appearance.rs`
- Modify: `applet/src/model/mod.rs`, `applet/src/config.rs`

**Interfaces:**
- Produces: `appearance::{ListDensity, POPUP_WIDTH_RANGE, POPUP_HEIGHT_RANGE, ICON_SIZE_RANGE, DEFAULT_POPUP_WIDTH, DEFAULT_POPUP_HEIGHT, DEFAULT_ICON_SIZE, clamp_popup_width(u32)->u32, clamp_popup_height(u32)->u32, clamp_icon_size(u16)->u16, item_height(ListDensity, icon_size: u16, space_l: u16, space_xl: u16)->f32, show_comment(ListDensity, space_xl: u16)->bool}`; `AppletConfig.{popup_width: u32, popup_height: u32, app_icon_size: u16, list_density: ListDensity}`.

- [ ] **Step 1: Create the module with tests only (functions stubbed with `todo!()`)**

Create `applet/src/model/appearance.rs`:

```rust
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
    todo!()
}

pub fn clamp_popup_height(value: u32) -> u32 {
    todo!()
}

pub fn clamp_icon_size(value: u16) -> u16 {
    todo!()
}

/// Row height of the app list. The only source of truth: rendering and
/// scroll math must both call this.
pub fn item_height(density: ListDensity, icon_size: u16, space_l: u16, space_xl: u16) -> f32 {
    todo!()
}

/// The comment line is shown only in `Normal` density, and only when the row
/// is tall enough (same rule as before this feature).
pub fn show_comment(density: ListDensity, space_xl: u16) -> bool {
    todo!()
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
```

Add `pub mod appearance;` to `applet/src/model/mod.rs` (new last line; the file has no trailing newline, so put it on its own line).

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet --lib appearance 2>&1 | tail -20`
Expected: 7 tests FAIL with `not yet implemented`.

- [ ] **Step 3: Implement**

Replace the five `todo!()` bodies:

```rust
pub fn clamp_popup_width(value: u32) -> u32 {
    value.clamp(POPUP_WIDTH_RANGE.0, POPUP_WIDTH_RANGE.1)
}

pub fn clamp_popup_height(value: u32) -> u32 {
    value.clamp(POPUP_HEIGHT_RANGE.0, POPUP_HEIGHT_RANGE.1)
}

pub fn clamp_icon_size(value: u16) -> u16 {
    value.clamp(ICON_SIZE_RANGE.0, ICON_SIZE_RANGE.1)
}

pub fn item_height(density: ListDensity, icon_size: u16, space_l: u16, space_xl: u16) -> f32 {
    let icon_size = clamp_icon_size(icon_size);
    let height = match density {
        ListDensity::Normal => space_xl.max(icon_size + 8),
        ListDensity::Compact => space_l.max(icon_size + 4),
    };
    f32::from(height)
}

pub fn show_comment(density: ListDensity, space_xl: u16) -> bool {
    density == ListDensity::Normal && space_xl >= 40
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet --lib appearance 2>&1 | tail -15`
Expected: 7 passed.

- [ ] **Step 5: Add config fields**

In `applet/src/config.rs` add to the imports `use crate::model::appearance::{ListDensity, DEFAULT_ICON_SIZE, DEFAULT_POPUP_HEIGHT, DEFAULT_POPUP_WIDTH};`, add to `AppletConfig` after `power_buttons`:

```rust
    pub popup_width: u32,
    pub popup_height: u32,
    pub app_icon_size: u16,
    pub list_density: ListDensity,
```

and to `Default` after `power_buttons: ...,`:

```rust
            popup_width: DEFAULT_POPUP_WIDTH,
            popup_height: DEFAULT_POPUP_HEIGHT,
            app_icon_size: DEFAULT_ICON_SIZE,
            list_density: ListDensity::default(),
```

- [ ] **Step 6: Run the whole applet suite and build both crates**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet --lib 2>&1 | tail -6; cargo build 2>&1 | grep -E "^error|Finished"`
Expected: all tests pass (17 = 10 old + 7 new), `Finished`.

- [ ] **Step 7: Commit**

```bash
git add applet/src/model/appearance.rs applet/src/model/mod.rs applet/src/config.rs docs/superpowers/plans/2026-09-25-appearance.md
git commit -m "feat(config): appearance model, clamping and item_height

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

---

### Task 2: Applet uses the settings (popup size, list rendering, scroll math)

**Files:**
- Modify: `applet/src/applet_menu.rs:44-47,99-111`, `applet/src/applet.rs` (two `item_height` sites in `select_prev_app`/`select_next_app`, plus one new method), `applet/src/widgets/virtualized_app_list.rs`

**Interfaces:**
- Consumes: everything Task 1 produces.
- Produces: `Applet::list_item_height(&self) -> f32`.

UI code; there is no unit-test harness for it. Verification is that the existing suite stays green, the build passes, and the manual check in Task 4.

- [ ] **Step 1: Popup size from config**

In `applet/src/applet_menu.rs` change the constants to the clamp ranges:

```rust
    pub const POPUP_MAX_WIDTH: f32 = 1200.0;
    pub const POPUP_MIN_WIDTH: f32 = 500.0;
    pub const POPUP_MAX_HEIGHT: f32 = 1200.0;
    pub const POPUP_MIN_HEIGHT: f32 = 400.0;
```

and replace the two fixed sizes in `.popup_container(...)`:

```rust
                menu_layout
                    .width(Length::Fixed(
                        appearance::clamp_popup_width(applet.config.popup_width) as f32,
                    ))
                    .height(Length::Fixed(
                        appearance::clamp_popup_height(applet.config.popup_height) as f32,
                    )),
```

Add `use crate::model::appearance;` to the imports. If `POPUP_MAX_HEIGHT` is used elsewhere, `grep -n POPUP_ applet/src` and keep those uses compiling.

- [ ] **Step 2: One item-height method on `Applet`**

In `applet/src/applet.rs`, in an `impl Applet` block (the one holding `select_prev_app`), add:

```rust
    /// Row height of the app list; shared by rendering and scroll math.
    pub fn list_item_height(&self) -> f32 {
        let spacing = cosmic::theme::active().cosmic().spacing;
        crate::model::appearance::item_height(
            self.config.list_density,
            self.config.app_icon_size,
            spacing.space_l,
            spacing.space_xl,
        )
    }
```

In both `select_prev_app` and `select_next_app` replace

```rust
            let spacing = cosmic::theme::active().cosmic().spacing;
            let item_height = spacing.space_xl as f32;
```

with `let item_height = self.list_item_height();`.

- [ ] **Step 3: Rendering in `virtualized_app_list.rs`**

In `view`: remove `space_xl` from the destructured `Spacing`, and replace `let item_height = space_xl as f32;` with `let item_height = applet.list_item_height();`. Change both spacer heights to `Length::Fixed`:

```rust
            let spacer_height = render_start as f32 * item_height;
            items.push(cosmic::widget::Space::new().width(Length::Fill).height(Length::Fixed(spacer_height)).into());
```

```rust
            let remaining_height = (total_items - render_end) as f32 * item_height;
            items.push(cosmic::widget::Space::new().width(Length::Fill).height(Length::Fixed(remaining_height)).into());
```

Pass the height down: `Self::create_app_button(applet, original_index, app, item_height)`.

In `create_app_button` add parameter `item_height: f32`, stop destructuring `space_xl`/`space_l` from the theme except `space_xl` for the comment rule:

```rust
        let space_xl = theme::active().cosmic().spacing.space_xl;
        let show_comment = appearance::show_comment(applet.config.list_density, space_xl);
        let icon_size = appearance::clamp_icon_size(applet.config.app_icon_size);
```

use `Self::create_icon_widget(app, icon_size)`, and `.height(Length::Fixed(item_height))` instead of `.height(space_xl)`. Add `use crate::model::appearance;`. The `create_icon_widget` signature (`space_l: u16`) already takes a size; leave it, only rename the argument in the call.

- [ ] **Step 4: Build and run the suite**

Run: `cargo build 2>&1 | grep -E "^error|Finished" -A8; cargo test -p cosmic-ext-classic-menu-plus-applet --lib 2>&1 | tail -4`
Expected: `Finished`, 17 passed.

- [ ] **Step 5: Commit**

```bash
git add applet/src
git commit -m "feat(applet): popup size, icon size and density from config

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

---

### Task 3: Settings app on a nav_bar

**Files:**
- Modify: `settings/src/app.rs`, `settings/i18n/en/cosmic_ext_classic_menu_plus_settings.ftl`, `settings/i18n/ru/cosmic_ext_classic_menu_plus_settings.ftl`

**Interfaces:**
- Produces: `enum Page { General, Appearance, PowerButtons }`, `AppModel.nav: nav_bar::Model`, method `appearance_section(&self) -> Section<'_, Message>` (an empty titled section for now; Task 4 fills it).

- [ ] **Step 1: i18n title**

Append to `en/...settings.ftl`: `appearance = Appearance`; to `ru/...settings.ftl`: `appearance = Внешний вид`.

- [ ] **Step 2: Model, nav and page enum**

In `settings/src/app.rs`: import `nav_bar` (`use cosmic::widget::{button, icon, menu, nav_bar, menu::{ItemWidth, ItemHeight}};`), add to `AppModel` the field `nav: nav_bar::Model,`, and above `AppModel` add:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    General,
    Appearance,
    PowerButtons,
}
```

In `init`, before `let app = AppModel {`:

```rust
        let mut nav = nav_bar::Model::default();
        nav.insert()
            .text(fl!("general"))
            .icon(icon::from_name("preferences-system-symbolic"))
            .data(Page::General)
            .activate();
        nav.insert()
            .text(fl!("appearance"))
            .icon(icon::from_name("preferences-desktop-theme-symbolic"))
            .data(Page::Appearance);
        nav.insert()
            .text(fl!("power-buttons"))
            .icon(icon::from_name("system-shutdown-symbolic"))
            .data(Page::PowerButtons);
```

add `nav,` to the struct literal, and inside `impl cosmic::Application for AppModel` (next to `core_mut`):

```rust
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        Task::none()
    }
```

If the compiler wants a different `Task` type for `on_nav_select`, use exactly what the trait declares (`cosmic::app::Task<Self::Message>`).

- [ ] **Step 3: Pick the page in `view`**

In `view`, change `let settings_container = cosmic::widget::settings::view_column(vec![cosmic::widget::settings::section()` to `let general_section = cosmic::widget::settings::section()`, and the tail

```rust
                .into(),
                self.power_buttons_section().into()]);

        cosmic::widget::scrollable(settings_container.padding([5, 10])).into()
```

to

```rust
                ;

        let page: Element<'_, Message> = match self.nav.active_data::<Page>() {
            Some(Page::Appearance) => self.appearance_section().into(),
            Some(Page::PowerButtons) => self.power_buttons_section().into(),
            _ => general_section.into(),
        };
        let settings_container = cosmic::widget::settings::view_column(vec![page]);

        cosmic::widget::scrollable(settings_container.padding([5, 10])).into()
```

Add next to `power_buttons_section`:

```rust
    fn appearance_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        cosmic::widget::settings::section().title(fl!("appearance"))
    }
```

- [ ] **Step 4: Build**

Run: `cargo build -p cosmic-ext-classic-menu-plus-settings 2>&1 | grep -E "^error|Finished" -A10`
Expected: `Finished`. Fix type errors if any (the `.into()` chain in the `add(...)` block must still compile).

- [ ] **Step 5: Commit**

```bash
git add settings
git commit -m "refactor(settings): nav_bar with General, Appearance and Power buttons pages

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

---

### Task 4: Appearance page controls

**Files:**
- Modify: `settings/src/app.rs`, both settings `.ftl` files

**Interfaces:**
- Consumes: Task 1 constants/clamps, Task 3 `appearance_section`.
- Produces: messages `PopupWidthChanged(u32)`, `PopupHeightChanged(u32)`, `AppIconSizeChanged(u16)`, `ListDensityChanged(usize)`.

- [ ] **Step 1: i18n keys**

`en`:
```
popup-width = Menu width
popup-height = Menu height
app-icon-size = App icon size
list-density = List density
density-compact = Compact
density-normal = Normal
```
`ru`:
```
popup-width = Ширина меню
popup-height = Высота меню
app-icon-size = Размер иконок приложений
list-density = Плотность списка
density-compact = Компактная
density-normal = Обычная
```

- [ ] **Step 2: Messages and update arms**

Import `cosmic_ext_classic_menu_plus_applet::model::appearance::{self, ListDensity}`. Add to `Message`: `PopupWidthChanged(u32), PopupHeightChanged(u32), AppIconSizeChanged(u16), ListDensityChanged(usize),`. Add arms before `Message::ToggleContextPage`:

```rust
            Message::PopupWidthChanged(value) => {
                self.config.popup_width = appearance::clamp_popup_width(value);
                self.write_config("popup width");
                Task::none()
            }
            Message::PopupHeightChanged(value) => {
                self.config.popup_height = appearance::clamp_popup_height(value);
                self.write_config("popup height");
                Task::none()
            }
            Message::AppIconSizeChanged(value) => {
                self.config.app_icon_size = appearance::clamp_icon_size(value);
                self.write_config("app icon size");
                Task::none()
            }
            Message::ListDensityChanged(index) => {
                self.config.list_density = match index {
                    0 => ListDensity::Compact,
                    _ => ListDensity::Normal,
                };
                self.write_config("list density");
                Task::none()
            }
```

and a helper in `impl AppModel`:

```rust
    fn write_config(&self, what: &str) {
        if let Err(err) = self
            .config
            .write_entry(AppletConfig::config_handler().as_ref().unwrap())
        {
            log::error!("failed to write {what}: {err}");
        }
    }
```

- [ ] **Step 3: The page**

Replace `appearance_section`:

```rust
    fn appearance_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let width = appearance::clamp_popup_width(self.config.popup_width);
        let height = appearance::clamp_popup_height(self.config.popup_height);
        let icon_size = appearance::clamp_icon_size(self.config.app_icon_size);
        let density = match self.config.list_density {
            ListDensity::Compact => 0,
            ListDensity::Normal => 1,
        };

        cosmic::widget::settings::section()
            .title(fl!("appearance"))
            .add(cosmic::widget::settings::item(
                fl!("popup-width"),
                cosmic::widget::spin_button(
                    width.to_string(),
                    fl!("popup-width"),
                    width,
                    10,
                    appearance::POPUP_WIDTH_RANGE.0,
                    appearance::POPUP_WIDTH_RANGE.1,
                    Message::PopupWidthChanged,
                ),
            ))
            .add(cosmic::widget::settings::item(
                fl!("popup-height"),
                cosmic::widget::spin_button(
                    height.to_string(),
                    fl!("popup-height"),
                    height,
                    10,
                    appearance::POPUP_HEIGHT_RANGE.0,
                    appearance::POPUP_HEIGHT_RANGE.1,
                    Message::PopupHeightChanged,
                ),
            ))
            .add(cosmic::widget::settings::item(
                fl!("app-icon-size"),
                cosmic::widget::spin_button(
                    icon_size.to_string(),
                    fl!("app-icon-size"),
                    icon_size,
                    2,
                    appearance::ICON_SIZE_RANGE.0,
                    appearance::ICON_SIZE_RANGE.1,
                    Message::AppIconSizeChanged,
                ),
            ))
            .add(cosmic::widget::settings::item(
                fl!("list-density"),
                cosmic::widget::dropdown(
                    vec![fl!("density-compact"), fl!("density-normal")],
                    Some(density),
                    Message::ListDensityChanged,
                ),
            ))
    }
```

- [ ] **Step 4: Build both crates in release and run the suite**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet --lib 2>&1 | tail -4; just build-release 2>&1 | grep -E "^error|Finished"`
Expected: 17 passed; two `Finished`.

- [ ] **Step 5: Commit**

```bash
git add settings
git commit -m "feat(settings): Appearance page with popup size, icon size and density

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

- [ ] **Step 6: Manual check (the user runs it; no ticks in PLAN.md before that)**

1. `! ./target/release/cosmic-ext-classic-menu-plus-settings`: nav on the left with three pages; each page opens; the window scrolls if needed.
2. `! sudo just install`, `! killall cosmic-panel`.
3. Width 500 and 1200, height 400 and 1200: the popup follows live without a restart; the power row and categories stay inside; note if 1200 does not fit the screen.
4. Icon size 16 and 64 in both densities: no clipped icons.
5. `Compact`: rows are shorter, no comment lines. `Normal`: as before.
6. Arrow Up/Down through the list in both densities: the selected row stays visible, no drift after many presses; scrolling with the mouse shows no blank gaps.
7. "Reset to defaults" returns 600×700, icon 24, `Normal`.
