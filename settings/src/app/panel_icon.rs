// SPDX-License-Identifier: GPL-3.0-only

//! The panel icon chooser: one dropdown entry per logo with a preview, and a
//! monochrome switch picking the `-symbolic` variant when the logo has one.

use std::path::{Path, PathBuf};

use cosmic::Element;
use cosmic::iced::Length;
use cosmic::widget::icon;

use super::{AppModel, Message};
use crate::fl;

/// A logo with its colour and monochrome files; either may be missing.
#[derive(Debug, Default, PartialEq)]
pub(super) struct Choice {
    base: String,
    color: Option<PathBuf>,
    mono: Option<PathBuf>,
}

impl Choice {
    fn contains(&self, path: &Path) -> bool {
        self.color.as_deref() == Some(path) || self.mono.as_deref() == Some(path)
    }

    /// The file to use: the requested variant if this logo has it.
    fn path(&self, mono: bool) -> &Path {
        let (wanted, other) = if mono {
            (&self.mono, &self.color)
        } else {
            (&self.color, &self.mono)
        };
        wanted
            .as_deref()
            .or(other.as_deref())
            .unwrap_or(Path::new(""))
    }
}

/// Groups icon files into logos by name, `x.svg` and `x-symbolic.svg` together.
pub(super) fn choices(paths: &[String]) -> Vec<Choice> {
    let mut choices: Vec<Choice> = Vec::new();
    for path in paths.iter().map(PathBuf::from) {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let (base, mono) = match stem.strip_suffix("-symbolic") {
            Some(base) => (base.to_string(), true),
            None => (stem.to_string(), false),
        };
        let index = match choices.iter().position(|c| c.base == base) {
            Some(index) => index,
            None => {
                choices.push(Choice {
                    base,
                    ..Choice::default()
                });
                choices.len() - 1
            }
        };
        let slot = if mono {
            &mut choices[index].mono
        } else {
            &mut choices[index].color
        };
        *slot = Some(path);
    }
    choices
}

/// Human name of a logo file base ("pop-os-logo" -> "Pop!_OS").
fn label(base: &str) -> String {
    let name = base
        .trim_end_matches("-logo")
        .trim_end_matches("_logo")
        .replace("-logo-", "-");
    let known = match name.as_str() {
        "classic-menu-plus" => "Classic Menu Plus",
        "default" => "Classic Menu",
        "almalinux" => "AlmaLinux",
        "asahilinux" => "Asahi Linux",
        "cachy" => "CachyOS",
        "cosmic" => "COSMIC",
        "cosmic-black" => "COSMIC (black)",
        "endeavouros" => "EndeavourOS",
        "mx" => "MX Linux",
        "nixos" => "NixOS",
        "opensuse" => "openSUSE",
        "pop-os" => "Pop!_OS",
        "postmarket" => "postmarketOS",
        "pureos" => "PureOS",
        "redhat" => "Red Hat",
        "rockylinux" => "Rocky Linux",
        "steam-deck-le" => "Steam Deck (LE)",
        "tux" => "Tux",
        "ublue" => "Universal Blue",
        _ => "",
    };
    if !known.is_empty() {
        return known.to_string();
    }
    name.split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl AppModel {
    /// Bundled logos, plus the current icon when it is a custom file.
    fn panel_icon_choices(&self) -> Vec<Choice> {
        let mut choices = choices(&self.panel_icons);
        // this app's own icon first
        if let Some(index) = choices.iter().position(|c| c.base == "classic-menu-plus") {
            let own = choices.remove(index);
            choices.insert(0, own);
        }
        let current = Path::new(&self.config.button_icon);
        if !choices.iter().any(|c| c.contains(current)) && current.exists() {
            choices.push(Choice {
                base: current
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string(),
                color: Some(current.to_path_buf()),
                mono: None,
            });
        }
        choices
    }

    fn icon_is_mono(&self) -> bool {
        Path::new(&self.config.button_icon)
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.ends_with("-symbolic"))
    }

    /// Dropdown of logos with previews; its last entry opens a file chooser.
    pub(super) fn panel_icon_dropdown(&self) -> Element<'_, Message> {
        let choices = self.panel_icon_choices();
        let current = Path::new(&self.config.button_icon);
        let selected = choices.iter().position(|c| c.contains(current));
        let mono = self.icon_is_mono();

        let mut labels: Vec<String> = choices.iter().map(|c| label(&c.base)).collect();
        labels.push(fl!("select-custom-icon"));
        let mut icons: Vec<icon::Handle> = choices
            .iter()
            .map(|c| icon::from_path(c.path(mono).to_path_buf()))
            .collect();
        icons.push(icon::from_name("document-open-symbolic").handle());

        let paths: Vec<PathBuf> = choices.iter().map(|c| c.path(mono).to_path_buf()).collect();
        cosmic::widget::dropdown(labels, selected, move |index| match paths.get(index) {
            Some(path) => Message::ButtonIconChanged(path.clone()),
            None => Message::CustomIconSelected,
        })
        .icons(std::borrow::Cow::Owned(icons))
        .width(Length::Fixed(260.0))
        .into()
    }

    /// Monochrome switch: off when the selected logo has only one variant.
    pub(super) fn panel_icon_mono_toggle(&self) -> Element<'_, Message> {
        let current = Path::new(&self.config.button_icon);
        let mono = self.icon_is_mono();
        let other = self
            .panel_icon_choices()
            .into_iter()
            .find(|c| c.contains(current))
            .filter(|c| c.color.is_some() && c.mono.is_some())
            .map(|c| c.path(!mono).to_path_buf());

        cosmic::widget::toggler(mono)
            .on_toggle_maybe(other.map(|path| move |_| Message::ButtonIconChanged(path.clone())))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_variants_and_picks_the_requested_one() {
        let paths = [
            "/i/fedora-logo-symbolic.svg",
            "/i/fedora-logo.svg",
            "/i/mx-logo-symbolic.svg",
        ]
        .map(String::from);
        let got = choices(&paths);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].path(false), Path::new("/i/fedora-logo.svg"));
        assert_eq!(got[0].path(true), Path::new("/i/fedora-logo-symbolic.svg"));
        // only a monochrome file: used either way
        assert_eq!(got[1].path(false), Path::new("/i/mx-logo-symbolic.svg"));
    }

    #[test]
    fn labels_are_readable() {
        assert_eq!(label("pop-os-logo"), "Pop!_OS");
        assert_eq!(label("endeavouros_logo"), "EndeavourOS");
        assert_eq!(label("cosmic-logo-black"), "COSMIC (black)");
        assert_eq!(label("steam-deck-logo"), "Steam Deck");
        assert_eq!(label("my_custom-icon"), "My Custom Icon");
    }
}
