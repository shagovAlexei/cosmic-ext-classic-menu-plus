use crate::applet::Message;
use logind_zbus::manager::IsSupported;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

/// Clicks on the confirm button are ignored for this long after it appears, so a
/// double-click on the power icon cannot confirm by accident.
pub const CONFIRM_GUARD: std::time::Duration = std::time::Duration::from_millis(400);

pub fn confirmation_ready(shown: std::time::Instant, now: std::time::Instant) -> bool {
    now.saturating_duration_since(shown) >= CONFIRM_GUARD
}

/// Interpret logind's `CanHibernate` answer; errors mean "not available".
pub fn hibernate_available_from(res: zbus::Result<IsSupported>) -> bool {
    matches!(res, Ok(IsSupported::Yes | IsSupported::Challenge))
}

#[cfg(test)]
mod tests {
    use super::*;
    use logind_zbus::manager::IsSupported;

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
    fn confirmation_ignores_clicks_right_after_it_appears() {
        let shown = std::time::Instant::now();
        assert!(!confirmation_ready(shown, shown));
        assert!(!confirmation_ready(shown, shown + CONFIRM_GUARD / 2));
        assert!(confirmation_ready(shown, shown + CONFIRM_GUARD));
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
