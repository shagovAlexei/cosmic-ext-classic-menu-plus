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
