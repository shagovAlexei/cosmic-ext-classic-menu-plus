# Power Buttons Config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the user choose which power buttons the menu shows and in what order, from the settings app.

**Architecture:** A new `AppletConfig.power_buttons: Vec<PowerAction>` holds the visible buttons in display order. All list logic (filtering, editor rows, toggling, moving) lives as pure functions next to `PowerAction` and is unit-tested. The applet renders `PowerAction::visible(&config.power_buttons, can_hibernate)`. The settings app gets a "Power buttons" section built on the same helpers. cosmic-config live-reload already propagates changes to the open menu.

**Tech Stack:** Rust 2024, libcosmic (`cosmic-config`, `toggler`, `settings::section`), serde, Fluent i18n.

**Spec:** `docs/PLAN.md` section "2. Выбор и порядок кнопок питания" and "Этап 2" TODO list.

## Global Constraints

- New config field only; `#[version = 1]` stays, existing config files must keep loading (missing key → default).
- Default value is `PowerAction::DEFAULT_ORDER` (Logout, Suspend, Hibernate, Lock, Reboot, Shutdown), i.e. the menu looks exactly as before.
- `power_buttons` holds only visible buttons, in display order. Hidden buttons are simply absent.
- Hibernate is additionally filtered by `can_hibernate` at render time, never removed from the config.
- If no button is visible, the power row is not rendered (no empty padded gap).
- "Reset to defaults" in settings must also restore `power_buttons` (it uses `AppletConfig::default()`, so this is automatic).
- Every new string goes into `settings/i18n/en/…` and `settings/i18n/ru/…` (`cosmic_ext_classic_menu_plus_settings.ftl`).
- Branch `feat/power-buttons-config`, created from `master`.
- Commit messages end with `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.

## Review Focus

1. Config containing duplicates or Hibernate while unsupported must render each button once and hide Hibernate (`visible` tests in Task 1).
2. Disabling every button must not leave an empty padded row (Task 1, manual check in Task 2).
3. Re-enabling a button places it at the end; moving the first item up or the last enabled item down is a no-op and its arrow is disabled (Task 1 tests, Task 2 UI).
4. An existing user config without the new key must still load and show the default six buttons (manual check in Task 2: delete the key file, reopen).
5. "Reset to defaults" restores all six buttons in default order (manual check in Task 2).

---

## File Structure

- Modify `applet/src/model/power_action.rs`: serde derives, new `visible` signature, `editor_rows`, `toggle_power_button`, `move_power_button`, `MoveDirection`, tests.
- Modify `applet/src/config.rs`: `power_buttons` field + default.
- Modify `applet/src/applet_menu.rs`: use config list; hide empty row.
- Modify `settings/src/app.rs`: messages, update arms, "Power buttons" section.
- Modify `settings/i18n/en/cosmic_ext_classic_menu_plus_settings.ftl` and `settings/i18n/ru/cosmic_ext_classic_menu_plus_settings.ftl`.
- Modify `docs/PLAN.md`: tick Stage 2 boxes (after manual verification by the user).

---

### Task 1: Pure list logic, config field, applet rendering

**Files:**
- Modify: `applet/src/model/power_action.rs`
- Modify: `applet/src/config.rs`
- Modify: `applet/src/applet_menu.rs` (`create_power_menu`)
- Test: inline `#[cfg(test)] mod tests` in `applet/src/model/power_action.rs`

**Interfaces:**
- Produces:
  - `PowerAction: Serialize + Deserialize`
  - `PowerAction::visible(configured: &[PowerAction], can_hibernate: bool) -> Vec<PowerAction>` (signature change; the old one took only `can_hibernate`)
  - `pub fn editor_rows(configured: &[PowerAction]) -> Vec<(PowerAction, bool)>`
  - `pub fn toggle_power_button(configured: &mut Vec<PowerAction>, action: PowerAction)`
  - `pub enum MoveDirection { Up, Down }` (`Clone, Copy, Debug, PartialEq, Eq`)
  - `pub fn move_power_button(configured: &mut Vec<PowerAction>, action: PowerAction, dir: MoveDirection)`
  - `AppletConfig.power_buttons: Vec<PowerAction>`

- [ ] **Step 1: Branch**

```bash
git checkout master && git pull origin master && git checkout -b feat/power-buttons-config
```

- [ ] **Step 2: Write the failing tests.** In `applet/src/model/power_action.rs` replace the two existing `visible_*` tests with the versions below and add the new ones inside `mod tests`:

```rust
    fn defaults() -> Vec<PowerAction> {
        PowerAction::DEFAULT_ORDER.to_vec()
    }

    #[test]
    fn visible_default_keeps_order_and_hibernate_when_supported() {
        assert_eq!(PowerAction::visible(&defaults(), true), defaults());
    }

    #[test]
    fn visible_omits_hibernate_when_unsupported() {
        let actions = PowerAction::visible(&defaults(), false);
        assert!(!actions.contains(&PowerAction::Hibernate));
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn visible_respects_custom_order_and_subset() {
        let configured = [PowerAction::Shutdown, PowerAction::Lock];
        assert_eq!(
            PowerAction::visible(&configured, true),
            vec![PowerAction::Shutdown, PowerAction::Lock]
        );
    }

    #[test]
    fn visible_drops_duplicates_and_handles_empty() {
        let configured = [PowerAction::Lock, PowerAction::Lock, PowerAction::Reboot];
        assert_eq!(
            PowerAction::visible(&configured, true),
            vec![PowerAction::Lock, PowerAction::Reboot]
        );
        assert!(PowerAction::visible(&[], true).is_empty());
    }

    #[test]
    fn editor_rows_lists_enabled_first_then_the_rest_in_default_order() {
        let rows = editor_rows(&[PowerAction::Shutdown, PowerAction::Lock]);
        assert_eq!(
            rows,
            vec![
                (PowerAction::Shutdown, true),
                (PowerAction::Lock, true),
                (PowerAction::Logout, false),
                (PowerAction::Suspend, false),
                (PowerAction::Hibernate, false),
                (PowerAction::Reboot, false),
            ]
        );
    }

    #[test]
    fn toggle_removes_present_action_and_appends_absent_one() {
        let mut list = vec![PowerAction::Logout, PowerAction::Lock];
        toggle_power_button(&mut list, PowerAction::Logout);
        assert_eq!(list, vec![PowerAction::Lock]);
        toggle_power_button(&mut list, PowerAction::Logout);
        assert_eq!(list, vec![PowerAction::Lock, PowerAction::Logout]);
    }

    #[test]
    fn move_swaps_neighbours_and_ignores_edges() {
        let mut list = vec![PowerAction::Logout, PowerAction::Lock, PowerAction::Reboot];
        move_power_button(&mut list, PowerAction::Lock, MoveDirection::Up);
        assert_eq!(
            list,
            vec![PowerAction::Lock, PowerAction::Logout, PowerAction::Reboot]
        );
        move_power_button(&mut list, PowerAction::Lock, MoveDirection::Up);
        move_power_button(&mut list, PowerAction::Reboot, MoveDirection::Down);
        move_power_button(&mut list, PowerAction::Shutdown, MoveDirection::Up);
        assert_eq!(
            list,
            vec![PowerAction::Lock, PowerAction::Logout, PowerAction::Reboot]
        );
        move_power_button(&mut list, PowerAction::Lock, MoveDirection::Down);
        assert_eq!(
            list,
            vec![PowerAction::Logout, PowerAction::Lock, PowerAction::Reboot]
        );
    }
```

Also change the existing `only_hibernate_needs_in_popup_confirmation` test: nothing to change there (it iterates `DEFAULT_ORDER`).

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet --lib power_action`
Expected: compile errors: `visible` takes 1 argument, no `editor_rows` / `toggle_power_button` / `move_power_button` / `MoveDirection`.

- [ ] **Step 4: Implement.** In `applet/src/model/power_action.rs`:

Add `use serde::{Deserialize, Serialize};` at the top and change the derive on the enum to `#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]`.

Replace `visible` with:

```rust
    /// Buttons to show in the power row: the configured ones, in order, without
    /// duplicates, minus Hibernate when the system cannot hibernate.
    pub fn visible(configured: &[PowerAction], can_hibernate: bool) -> Vec<PowerAction> {
        let mut out: Vec<PowerAction> = Vec::new();
        for &action in configured {
            let hidden = action == PowerAction::Hibernate && !can_hibernate;
            if !hidden && !out.contains(&action) {
                out.push(action);
            }
        }
        out
    }
```

Add after the `impl PowerAction` block (before `CONFIRM_GUARD`):

```rust
/// Rows for the settings editor: enabled actions first (in display order),
/// then the remaining actions, disabled, in default order.
pub fn editor_rows(configured: &[PowerAction]) -> Vec<(PowerAction, bool)> {
    let mut rows: Vec<(PowerAction, bool)> = Vec::new();
    for &action in configured {
        if !rows.iter().any(|(a, _)| *a == action) {
            rows.push((action, true));
        }
    }
    for action in PowerAction::DEFAULT_ORDER {
        if !rows.iter().any(|(a, _)| *a == action) {
            rows.push((action, false));
        }
    }
    rows
}

/// Hide the action if it is shown, otherwise show it at the end.
pub fn toggle_power_button(configured: &mut Vec<PowerAction>, action: PowerAction) {
    let before = configured.len();
    configured.retain(|a| *a != action);
    if configured.len() == before {
        configured.push(action);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

/// Swap the action with its neighbour; no-op at the edges or if it is not shown.
pub fn move_power_button(
    configured: &mut Vec<PowerAction>,
    action: PowerAction,
    direction: MoveDirection,
) {
    let Some(index) = configured.iter().position(|a| *a == action) else {
        return;
    };
    let target = match direction {
        MoveDirection::Up => index.checked_sub(1),
        MoveDirection::Down => (index + 1 < configured.len()).then_some(index + 1),
    };
    if let Some(target) = target {
        configured.swap(index, target);
    }
}
```

In `applet/src/config.rs` add `use crate::model::power_action::PowerAction;`, the field `pub power_buttons: Vec<PowerAction>,` at the end of `AppletConfig` (after `recent_applications`), and in `Default`: `power_buttons: PowerAction::DEFAULT_ORDER.to_vec(),`.

In `applet/src/applet_menu.rs`, `create_power_menu`: replace the `PowerAction::visible(applet.can_hibernate)` call so the icon-row branch reads:

```rust
        } else {
            let actions = PowerAction::visible(&applet.config.power_buttons, applet.can_hibernate);
            if actions.is_empty() {
                return cosmic::widget::Space::new().width(0).height(0).into();
            }
            cosmic::widget::row::with_children(
                actions
                    .into_iter()
```

(keep the rest of the branch unchanged).

- [ ] **Step 5: Run the tests to verify they pass, then build everything**

Run: `cargo test -p cosmic-ext-classic-menu-plus-applet --lib && cargo build --workspace`
Expected: all tests PASS (5 old + new ones), build OK. If `CosmicConfigEntry` complains about `Vec<PowerAction>`, check that the derives on `PowerAction` are in place.

- [ ] **Step 6: Commit**

```bash
git add applet
git commit -m "feat(power): configurable power button list

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

---

### Task 2: Settings section, i18n, verification

**Files:**
- Modify: `settings/src/app.rs`
- Modify: `settings/i18n/en/cosmic_ext_classic_menu_plus_settings.ftl`, `settings/i18n/ru/cosmic_ext_classic_menu_plus_settings.ftl`
- Modify: `docs/PLAN.md`

**Interfaces:**
- Consumes: `editor_rows`, `toggle_power_button`, `move_power_button`, `MoveDirection`, `AppletConfig.power_buttons` (Task 1)

The settings UI is not unit-testable (needs a running COSMIC app); all its logic is in the tested helpers.

- [ ] **Step 1: i18n strings.** Append to the `en` file:

```
power-buttons = Power buttons
power-logout = Log out
power-suspend = Suspend
power-hibernate = Hibernate
power-lock = Lock screen
power-reboot = Restart
power-shutdown = Shut down
move-up = Move up
move-down = Move down
```

Append to the `ru` file:

```
power-buttons = Кнопки питания
power-logout = Выйти
power-suspend = Сон
power-hibernate = Гибернация
power-lock = Заблокировать экран
power-reboot = Перезагрузить
power-shutdown = Выключить
move-up = Вверх
move-down = Вниз
```

- [ ] **Step 2: Messages and update arms.** In `settings/src/app.rs` extend the imports:

```rust
use cosmic_ext_classic_menu_plus_applet::model::power_action::{
    editor_rows, move_power_button, toggle_power_button, MoveDirection, PowerAction,
};
```

Add to `enum Message`:

```rust
    PowerButtonToggled(PowerAction),
    PowerButtonMoved(PowerAction, MoveDirection),
```

Add to `update`, before `Message::ToggleContextPage`:

```rust
            Message::PowerButtonToggled(action) => {
                toggle_power_button(&mut self.config.power_buttons, action);

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write power buttons config");

                Task::none()
            }
            Message::PowerButtonMoved(action, direction) => {
                move_power_button(&mut self.config.power_buttons, action, direction);

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write power buttons config");

                Task::none()
            }
```

- [ ] **Step 3: The section.** Add to `impl AppModel` (next to `icon_picker`):

```rust
    fn power_action_label(action: PowerAction) -> String {
        match action {
            PowerAction::Logout => fl!("power-logout"),
            PowerAction::Suspend => fl!("power-suspend"),
            PowerAction::Hibernate => fl!("power-hibernate"),
            PowerAction::Lock => fl!("power-lock"),
            PowerAction::Reboot => fl!("power-reboot"),
            PowerAction::Shutdown => fl!("power-shutdown"),
        }
    }

    fn power_buttons_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let space_xxs = cosmic::theme::active().cosmic().space_xxs();
        let rows = editor_rows(&self.config.power_buttons);
        let enabled_count = rows.iter().filter(|(_, enabled)| *enabled).count();

        let mut section = cosmic::widget::settings::section().title(fl!("power-buttons"));
        for (index, (action, enabled)) in rows.into_iter().enumerate() {
            let up = cosmic::widget::button::icon(icon::from_name("go-up-symbolic"))
                .on_press_maybe(
                    (enabled && index > 0)
                        .then_some(Message::PowerButtonMoved(action, MoveDirection::Up)),
                );
            let down = cosmic::widget::button::icon(icon::from_name("go-down-symbolic"))
                .on_press_maybe(
                    (enabled && index + 1 < enabled_count)
                        .then_some(Message::PowerButtonMoved(action, MoveDirection::Down)),
                );
            let controls = cosmic::iced::widget::row![
                up,
                down,
                cosmic::widget::toggler(enabled)
                    .on_toggle(move |_| Message::PowerButtonToggled(action)),
            ]
            .spacing(space_xxs)
            .align_y(Alignment::Center);

            section = section.add(cosmic::widget::settings::item(
                Self::power_action_label(action),
                controls,
            ));
        }
        section
    }
```

In `view`, add the section to the `view_column` vector: change `cosmic::widget::settings::view_column(vec![cosmic::widget::settings::section()....into()])` so the vector has two items: the existing general section `.into()` followed by `self.power_buttons_section().into()`.

(If `on_press_maybe` is not available on the icon button builder, use `let up = b; if cond { up.on_press(msg) } else { up }` instead. If `Section` has no public type path, return `Element<'_, Message>` via `.into()`.)

- [ ] **Step 4: Build and run all tests**

Run: `cargo build --workspace && cargo test -p cosmic-ext-classic-menu-plus-applet --lib`
Expected: build OK, all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add settings docs
git commit -m "feat(settings): power buttons section (show/hide, reorder)

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

- [ ] **Step 6: Manual verification (done by the user, needs `sudo just install`).**
  1. Settings → "Кнопки питания": six rows, all on, arrows enabled except the first row's ↑ and the last row's ↓.
  2. Turn off "Заблокировать экран": the lock icon disappears from the open menu without restarting it.
  3. Move "Гибернация" up: the snowflake moves in the menu.
  4. Turn a button back on: it appears at the end.
  5. Turn everything off: the power row disappears, no empty gap under the categories.
  6. "Reset to defaults": all six buttons return in the default order.
  7. Delete the file `~/.config/cosmic/io.github.shagovAlexei.cosmic-ext-classic-menu-plus/v1/power_buttons`, reopen the menu: six default buttons.
  8. Hibernate turned on in settings but unsupported by the system: still hidden in the menu (cannot be tested on the user's machine; covered by unit test).

- [ ] **Step 7: Update `docs/PLAN.md`** ticking Stage 2 boxes once the user confirms Step 6, then commit.
