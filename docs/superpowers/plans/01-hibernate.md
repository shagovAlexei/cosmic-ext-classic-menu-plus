# Hibernate Button Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "Hibernate" button to the menu's power row. It calls logind `Hibernate` over D-Bus after an in-popup confirmation and is hidden when the system can't hibernate.

**Architecture:** `PowerAction` gains a `Hibernate` variant plus three pure helpers: which actions to show, which need confirmation, and whether a logind `CanHibernate` answer means "available". These helpers get unit tests. `Applet` stores `can_hibernate: bool` (fetched once at init) and `pending_confirmation: Option<PowerAction>`. When a confirmation is pending, `AppletMenu::create_power_menu` renders "question + Cancel/Confirm" instead of the icon row.

**Tech Stack:** Rust 2024, libcosmic (iced), `logind-zbus` 5.3.2 (`ManagerProxy::hibernate(bool)`, `ManagerProxy::can_hibernate() -> IsSupported`), Fluent i18n via `fl!`.

**Spec:** `docs/PLAN.md` section "4. Гибернация" and "9.1" TODO list.

## Global Constraints

- `APP_ID` stays `com.championpeak87.cosmic-ext-classic-menu`; do not rename anything upstream-owned.
- No `AppletConfig` changes in this stage (config-driven button list is stage 9.2).
- Hibernate goes through logind D-Bus directly (no `cosmic-osd`, no `flatpak-spawn`), like Suspend/Lock.
- Available iff `CanHibernate` is `yes` or `challenge`; `na`, `no` or a D-Bus error → hidden.
- Default button order: Logout, Suspend, Hibernate, Lock, Reboot, Shutdown.
- Every new user-visible string is added to both `applet/i18n/en/…ftl` and `applet/i18n/ru/…ftl`.
- Work on branch `feat/hibernate` created from `docs/fork-plan`.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. Pending confirmation must not survive a popup close/reopen: reopening the menu shows normal icons (pinned by the `toggle_popup`/`close_popup` resets in Task 2; checked manually in Task 3).
2. The 6-button row must fit in the categories pane (~3/8 of a 600px popup) without clipping or wrapping (manual check in Task 3).
3. A D-Bus failure while querying `CanHibernate` must hide the button rather than show a broken one (`hibernate_available_from(Err)` test in Task 1).
4. Clicking any other power button while a confirmation is pending must not trigger hibernate (the confirm row replaces the icons, so they can't be clicked; checked manually in Task 3).
5. The confirmation question must stay readable at ~210px width in Russian (text wraps above the buttons; manual check in Task 3).

---

## File Structure

- Modify `applet/src/model/power_action.rs`: `Hibernate` variant, `perform()` arm, pure helpers + unit tests.
- Modify `applet/src/power_options.rs`: `hibernate()` and `can_hibernate()` async fns.
- Modify `applet/src/applet.rs`: new state fields, messages, init task, confirmation flow, resets.
- Modify `applet/src/applet_menu.rs`: data-driven icon row, hibernate icon constant, confirmation row.
- Create `res/icons/bundled/system-hibernate-symbolic.svg`: an original 16×16 snowflake glyph matching the style of the other bundled icons.
- Modify `applet/i18n/en/cosmic_ext_classic_menu_applet.ftl` and `applet/i18n/ru/cosmic_ext_classic_menu_applet.ftl`.
- Modify `docs/PLAN.md`: tick stage 9.1 boxes.

---

### Task 1: PowerAction::Hibernate, logind calls, pure helpers

**Files:**
- Modify: `applet/src/model/power_action.rs`
- Modify: `applet/src/power_options.rs`
- Test: inline `#[cfg(test)] mod tests` in `applet/src/model/power_action.rs`

**Interfaces:**
- Produces:
  - `PowerAction::Hibernate` (the enum also gains `Copy, Eq` derives)
  - `pub const PowerAction::DEFAULT_ORDER: [PowerAction; 6]`
  - `pub fn PowerAction::visible(can_hibernate: bool) -> Vec<PowerAction>`
  - `pub fn PowerAction::needs_confirmation(self) -> bool`
  - `pub fn hibernate_available_from(res: zbus::Result<logind_zbus::manager::IsSupported>) -> bool` (in `power_action.rs`)
  - `pub async fn power_options::hibernate() -> zbus::Result<()>`
  - `pub async fn power_options::can_hibernate() -> bool`

- [ ] **Step 1: Create the branch**

```bash
git checkout docs/fork-plan && git checkout -b feat/hibernate
```

- [ ] **Step 2: Write the failing tests** at the end of `applet/src/model/power_action.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use logind_zbus::manager::IsSupported;

    #[test]
    fn visible_includes_hibernate_after_suspend_when_supported() {
        assert_eq!(
            PowerAction::visible(true),
            vec![
                PowerAction::Logout,
                PowerAction::Suspend,
                PowerAction::Hibernate,
                PowerAction::Lock,
                PowerAction::Reboot,
                PowerAction::Shutdown,
            ]
        );
    }

    #[test]
    fn visible_omits_hibernate_when_unsupported() {
        let actions = PowerAction::visible(false);
        assert!(!actions.contains(&PowerAction::Hibernate));
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn only_hibernate_needs_in_popup_confirmation() {
        for action in PowerAction::DEFAULT_ORDER {
            assert_eq!(
                action.needs_confirmation(),
                action == PowerAction::Hibernate,
                "{action:?}"
            );
        }
    }

    #[test]
    fn hibernate_available_only_for_yes_or_challenge() {
        assert!(hibernate_available_from(Ok(IsSupported::Yes)));
        assert!(hibernate_available_from(Ok(IsSupported::Challenge)));
        assert!(!hibernate_available_from(Ok(IsSupported::No)));
        assert!(!hibernate_available_from(Ok(IsSupported::NA)));
        assert!(!hibernate_available_from(Err(zbus::Error::Failure(
            "no logind".into()
        ))));
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p cosmic-ext-classic-menu-applet --lib power_action`
Expected: compile errors: no variant `Hibernate`, no function `visible` / `needs_confirmation` / `hibernate_available_from`.

- [ ] **Step 4: Implement.** Replace the body of `applet/src/model/power_action.rs` above the tests with:

```rust
use crate::applet::Message;
use logind_zbus::manager::IsSupported;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown,
    Logout,
    Lock,
    Reboot,
    Suspend,
    Hibernate,
}

impl PowerAction {
    /// Order of the buttons in the power row.
    pub const DEFAULT_ORDER: [PowerAction; 6] = [
        PowerAction::Logout,
        PowerAction::Suspend,
        PowerAction::Hibernate,
        PowerAction::Lock,
        PowerAction::Reboot,
        PowerAction::Shutdown,
    ];

    /// Buttons to show in the power row.
    pub fn visible(can_hibernate: bool) -> Vec<PowerAction> {
        Self::DEFAULT_ORDER
            .into_iter()
            .filter(|a| can_hibernate || *a != PowerAction::Hibernate)
            .collect()
    }

    /// Actions confirmed inside the popup (others use cosmic-osd or need none).
    pub fn needs_confirmation(self) -> bool {
        self == PowerAction::Hibernate
    }

    pub fn perform(self) -> cosmic::iced::Task<cosmic::Action<Message>> {
        let msg = |m| cosmic::Action::App(Message::Zbus(m));
        match self {
            PowerAction::Lock => cosmic::iced::Task::perform(crate::power_options::lock(), msg),
            PowerAction::Logout => {
                cosmic::iced::Task::perform(crate::power_options::log_out(), msg)
            }
            PowerAction::Reboot => {
                cosmic::iced::Task::perform(crate::power_options::restart(), msg)
            }
            PowerAction::Shutdown => {
                cosmic::iced::Task::perform(crate::power_options::shutdown(), msg)
            }
            PowerAction::Suspend => {
                cosmic::iced::Task::perform(crate::power_options::suspend(), msg)
            }
            PowerAction::Hibernate => {
                cosmic::iced::Task::perform(crate::power_options::hibernate(), msg)
            }
        }
    }
}

/// Interpret logind's `CanHibernate` answer; errors mean "not available".
pub fn hibernate_available_from(res: zbus::Result<IsSupported>) -> bool {
    matches!(res, Ok(IsSupported::Yes | IsSupported::Challenge))
}
```

Add to `applet/src/power_options.rs` after `suspend()`:

```rust
pub async fn hibernate() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    let manager_proxy = ManagerProxy::new(&connection).await?;
    manager_proxy.hibernate(true).await
}

pub async fn can_hibernate() -> bool {
    let res = async {
        let connection = Connection::system().await?;
        let manager_proxy = ManagerProxy::new(&connection).await?;
        manager_proxy.can_hibernate().await
    }
    .await;
    if let Err(e) = &res {
        log::warn!("CanHibernate query failed: {e}");
    }
    crate::model::power_action::hibernate_available_from(res)
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p cosmic-ext-classic-menu-applet --lib power_action`
Expected: 4 tests PASS. `cargo build` may warn that `Hibernate` is not yet handled in `applet.rs::perform_power_action`'s `match` (it has a `_ => ""` arm, so it compiles).

- [ ] **Step 6: Commit**

```bash
git add applet/src/model/power_action.rs applet/src/power_options.rs
git commit -m "feat(power): add Hibernate action via logind

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Applet state, messages and confirmation flow

**Files:**
- Modify: `applet/src/applet.rs`: struct `Applet` (~line 43), `Message` (~line 80), `init` (~line 136), `update` (~line 242), `toggle_popup` (~line 492), `close_popup` (~line 539), `perform_power_action` (~line 579)

**Interfaces:**
- Consumes: `PowerAction::needs_confirmation`, `power_options::can_hibernate` (Task 1)
- Produces:
  - `Applet.can_hibernate: bool`, `Applet.pending_confirmation: Option<PowerAction>`
  - `Message::HibernateSupport(bool)`, `Message::ConfirmPowerAction`, `Message::CancelPowerAction`

No unit tests: `Applet` needs a live `cosmic::Core`. The decision logic is already covered in Task 1, and this flow is verified manually in Task 3.

- [ ] **Step 1: Add state fields** to `pub struct Applet`, after `scroll_viewport_height`:

```rust
    /// Whether logind reports hibernation as available.
    pub can_hibernate: bool,
    /// Power action waiting for in-popup confirmation.
    pub pending_confirmation: Option<PowerAction>,
```

In `init`, add to the `Applet { ... }` literal:

```rust
            can_hibernate: false,
            pending_confirmation: None,
```

- [ ] **Step 2: Add messages** to `pub enum Message` after `PowerOptionSelected(PowerAction),`:

```rust
    HibernateSupport(bool),
    ConfirmPowerAction,
    CancelPowerAction,
```

- [ ] **Step 3: Query support at init.** In `init`, before the final tuple, add:

```rust
        let fetch_can_hibernate_task =
            Task::perform(crate::power_options::can_hibernate(), |available| {
                cosmic::Action::App(Message::HibernateSupport(available))
            });
```

and add `fetch_can_hibernate_task,` to the `Task::batch(vec![...])`.

- [ ] **Step 4: Handle messages** in `update`, after the `Message::PowerOptionSelected` arm:

```rust
            Message::HibernateSupport(available) => {
                self.can_hibernate = available;
                Task::none()
            }
            Message::ConfirmPowerAction => match self.pending_confirmation.take() {
                Some(action) => {
                    let mut tasks = vec![action.perform()];
                    if let Some(p) = self.popup.take() {
                        tasks.push(destroy_popup(p));
                    }
                    Task::batch(tasks)
                }
                None => Task::none(),
            },
            Message::CancelPowerAction => {
                self.pending_confirmation = None;
                Task::none()
            }
```

- [ ] **Step 5: Route confirmable actions.** At the top of `perform_power_action`, before `let is_flatpak`:

```rust
        if action.needs_confirmation() {
            self.pending_confirmation = Some(action);
            return Task::none();
        }
```

- [ ] **Step 6: Reset on popup open/close.** In `toggle_popup`, after `self.selected_item_index = None;` add `self.pending_confirmation = None;`. In `close_popup`, inside the `if`, add `self.pending_confirmation = None;`.

- [ ] **Step 7: Build**

Run: `cargo build -p cosmic-ext-classic-menu-applet`
Expected: builds. If there's a "non-exhaustive patterns" error, check `PowerAction` matches elsewhere (`grep -rn "PowerAction::" applet/src`).

- [ ] **Step 8: Commit**

```bash
git add applet/src/applet.rs
git commit -m "feat(power): in-popup confirmation state for hibernate

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Power row UI, icon, i18n, manual verification

**Files:**
- Create: `res/icons/bundled/system-hibernate-symbolic.svg`
- Modify: `applet/src/applet_menu.rs`: icon constants (~line 48) and `create_power_menu` (~line 110)
- Modify: `applet/i18n/en/cosmic_ext_classic_menu_applet.ftl`, `applet/i18n/ru/cosmic_ext_classic_menu_applet.ftl`
- Modify: `docs/PLAN.md`

**Interfaces:**
- Consumes: `PowerAction::visible`, `Applet.can_hibernate`, `Applet.pending_confirmation`, `Message::{ConfirmPowerAction, CancelPowerAction, PowerOptionSelected}`

- [ ] **Step 1: Create the icon** `res/icons/bundled/system-hibernate-symbolic.svg` (an original snowflake: three bars with V-branches at each end, same `#232323` fill as the sibling icons so `.symbolic(true)` recolors it):

```svg
<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<defs>
<g id="arm" fill="#232323">
<rect x="7.25" y="1" width="1.5" height="14" rx="0.75"/>
<rect x="7.25" y="1.2" width="1.5" height="2.8" rx="0.75" transform="rotate(-45 8 4)"/>
<rect x="7.25" y="1.2" width="1.5" height="2.8" rx="0.75" transform="rotate(45 8 4)"/>
<rect x="7.25" y="12" width="1.5" height="2.8" rx="0.75" transform="rotate(-45 8 12)"/>
<rect x="7.25" y="12" width="1.5" height="2.8" rx="0.75" transform="rotate(45 8 12)"/>
</g>
</defs>
<use href="#arm"/>
<use href="#arm" transform="rotate(60 8 8)"/>
<use href="#arm" transform="rotate(-60 8 8)"/>
</svg>
```

- [ ] **Step 2: Add i18n strings.** Append to `applet/i18n/en/cosmic_ext_classic_menu_applet.ftl`:

```
# power actions
hibernate-confirm-question=Hibernate now?
hibernate-confirm-accept=Hibernate
cancel=Cancel
```

Append to `applet/i18n/ru/cosmic_ext_classic_menu_applet.ftl`:

```
# действия питания
hibernate-confirm-question=Перейти в гибернацию?
hibernate-confirm-accept=Гибернация
cancel=Отмена
```

- [ ] **Step 3: Rewrite `create_power_menu`.** In `applet/src/applet_menu.rs`, add after `SYSTEM_SUSPEND_SYMBOLIC_ICON`:

```rust
    const SYSTEM_HIBERNATE_SYMBOLIC_ICON: &[u8] =
        include_bytes!("../../res/icons/bundled/system-hibernate-symbolic.svg");
```

Add an icon lookup inside `impl AppletMenu`:

```rust
    fn power_action_icon(action: PowerAction) -> &'static [u8] {
        match action {
            PowerAction::Logout => AppletMenu::SYSTEM_LOGOUT_SYMBOLIC_ICON,
            PowerAction::Suspend => AppletMenu::SYSTEM_SUSPEND_SYMBOLIC_ICON,
            PowerAction::Hibernate => AppletMenu::SYSTEM_HIBERNATE_SYMBOLIC_ICON,
            PowerAction::Lock => AppletMenu::SYSTEM_LOCKSCREEN_SYMBOLIC_ICON,
            PowerAction::Reboot => AppletMenu::SYSTEM_REBOOT_SYMBOLIC_ICON,
            PowerAction::Shutdown => AppletMenu::SYSTEM_SHUTDOWN_SYMBOLIC_ICON,
        }
    }
```

Replace the whole `fn create_power_menu(_applet: &Applet)` with:

```rust
    fn create_power_menu(applet: &Applet) -> Element<'_, Message> {
        let Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let content: Element<Message> = if applet.pending_confirmation.is_some() {
            column![
                text(fl!("hibernate-confirm-question")).align_x(Alignment::Center),
                row![
                    cosmic::widget::button::standard(fl!("cancel"))
                        .on_press(Message::CancelPowerAction),
                    cosmic::widget::button::suggested(fl!("hibernate-confirm-accept"))
                        .on_press(Message::ConfirmPowerAction),
                ]
                .spacing(space_xxs)
                .align_y(Alignment::Center),
            ]
            .spacing(space_xxs)
            .align_x(Alignment::Center)
            .into()
        } else {
            cosmic::widget::row::with_children(
                PowerAction::visible(applet.can_hibernate)
                    .into_iter()
                    .map(|action| {
                        cosmic::widget::button::icon(
                            cosmic::widget::icon::from_svg_bytes(
                                AppletMenu::power_action_icon(action),
                            )
                            .symbolic(true),
                        )
                        .on_press(Message::PowerOptionSelected(action))
                        .into()
                    })
                    .collect(),
            )
            .align_y(Alignment::Center)
            .into()
        };

        container(content)
            .width(Length::Fill)
            .padding([20, 0])
            .align_x(Alignment::Center)
            .into()
    }
```

- [ ] **Step 4: Build and run all tests**

Run: `cargo build -p cosmic-ext-classic-menu-applet && cargo test -p cosmic-ext-classic-menu-applet --lib`
Expected: build OK, 4 tests PASS. If `text(...).align_x` doesn't exist for this libcosmic version, drop the `.align_x` call on the text (the column already centers it).

- [ ] **Step 5: Install and restart the panel**

```bash
just build-release && sudo just install && killall cosmic-panel
```

The panel respawns automatically; wait ~3 s.

- [ ] **Step 6: Manual verification checklist** (each item must be observed, not assumed):
  1. The power row shows 6 icons in order: logout, suspend (moon), hibernate (snowflake), lock, reboot, shutdown. No clipping or wrapping in the categories pane.
  2. The snowflake is recolored like the neighbors in both light and dark theme.
  3. Click the snowflake: the row turns into "Перейти в гибернацию?" + [Отмена] [Гибернация]. The text is readable and not cut off.
  4. Click [Отмена]: the icons return and nothing happens.
  5. Click the snowflake, close the menu (Esc or click outside), reopen: the icons are shown, not the question.
  6. Click the snowflake, then [Гибернация]: the popup closes and the machine hibernates. After resume, the session is intact.
  7. Other buttons are unaffected: lock and suspend act immediately, logout/reboot/shutdown still show the cosmic-osd dialog (press cancel there).

- [ ] **Step 7: Update `docs/PLAN.md`.** Tick every stage 9.1 checkbox that was verified in Step 6, except "Убрать `applet-hibernate` с панели" (the user does that).

- [ ] **Step 8: Commit**

```bash
git add res/icons/bundled/system-hibernate-symbolic.svg applet/src/applet_menu.rs applet/i18n/en/cosmic_ext_classic_menu_applet.ftl applet/i18n/ru/cosmic_ext_classic_menu_applet.ftl docs/PLAN.md
git commit -m "feat(power): hibernate button with in-popup confirmation

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
