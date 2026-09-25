// SPDX-License-Identifier: {{ license }}

//! The "Categories" settings page: reordering/hiding categories, creating
//! and editing custom ones, and the hidden/moved applications sub-lists.

use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{icon, text};
use cosmic_ext_classic_menu_plus_applet::logic::categories as cat;
use cosmic_ext_classic_menu_plus_applet::model::application_category::{
    ApplicationCategory, CATEGORY_ICONS, CUSTOM_KEY_PREFIX, CategoryIcon, CategoryKind,
    NAMED_ICON_CHOICES,
};
use cosmic_ext_classic_menu_plus_applet::model::power_action::MoveDirection;

use super::{AppModel, Message};
use crate::fl;

impl AppModel {
    pub(super) fn categories_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let space_xxs = cosmic::theme::active().cosmic().space_xxs();
        let rows = cat::editor_rows(&self.config);
        let rest_len = rows
            .iter()
            .filter(|(c, _)| c.kind != CategoryKind::Permanent)
            .count();

        let mut section = cosmic::widget::settings::section().title(fl!("categories"));
        let mut rest_index = 0usize;

        for (category, visible) in rows {
            let is_permanent = category.kind == CategoryKind::Permanent;
            let is_all = category == ApplicationCategory::ALL;
            let is_custom = category.kind == CategoryKind::Custom;
            let key = category.key.to_string();

            let icon_el = Self::render_category_icon(&category.icon, 20);

            let name_el: Element<'_, Message> = if is_custom {
                let id = key.trim_start_matches(CUSTOM_KEY_PREFIX).to_string();
                cosmic::widget::text_input(
                    fl!("category-name-placeholder"),
                    category.get_display_name(),
                )
                .on_input(move |value| Message::CategoryRenamed(id.clone(), value))
                .width(Length::Fill)
                .into()
            } else {
                text::body(category.get_display_name())
                    .width(Length::Fill)
                    .into()
            };

            let mut controls = cosmic::iced::widget::row![]
                .spacing(space_xxs)
                .align_y(Alignment::Center);

            if is_permanent {
                controls = controls.push(
                    cosmic::widget::Space::new()
                        .width(2 * 32)
                        .height(Length::Shrink),
                );
            } else {
                let up_key = key.clone();
                let down_key = key.clone();
                let up = cosmic::widget::button::icon(icon::from_name("go-up-symbolic"))
                    .on_press_maybe(
                        (rest_index > 0)
                            .then_some(Message::CategoryMoved(up_key, MoveDirection::Up)),
                    );
                let down = cosmic::widget::button::icon(icon::from_name("go-down-symbolic"))
                    .on_press_maybe(
                        (rest_index + 1 < rest_len)
                            .then_some(Message::CategoryMoved(down_key, MoveDirection::Down)),
                    );
                controls = controls.push(up).push(down);
                rest_index += 1;
            }

            if is_all {
                controls = controls.push(
                    cosmic::widget::Space::new()
                        .width(40)
                        .height(Length::Shrink),
                );
            } else {
                let toggle_key = key.clone();
                controls = controls
                    .push(cosmic::widget::toggler(visible).on_toggle(move |_| {
                        Message::CategoryVisibilityToggled(toggle_key.clone())
                    }));
            }

            if is_custom {
                let id = key.trim_start_matches(CUSTOM_KEY_PREFIX).to_string();
                let icon_id = id.clone();
                controls = controls.push(
                    cosmic::widget::button::icon(icon::from_name("image-x-generic-symbolic"))
                        .on_press(Message::OpenCategoryIconPicker(icon_id)),
                );
                controls = controls.push(
                    cosmic::widget::button::icon(icon::from_name("edit-delete-symbolic"))
                        .on_press(Message::CategoryRemoved(id)),
                );
            }

            section = section.add(cosmic::widget::settings::item_row(vec![
                icon_el,
                name_el,
                controls.into(),
            ]));
        }

        section
    }

    pub(super) fn hidden_apps_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let mut section = cosmic::widget::settings::section().title(fl!("hidden-apps"));
        if self.config.hidden_apps.is_empty() {
            return section.add(text::body(fl!("hidden-apps-empty")));
        }
        for id in &self.config.hidden_apps {
            let app = self.apps.iter().find(|a| &a.id == id);
            let name = app.map_or_else(|| id.clone(), |a| a.name.clone());
            let unhide = cosmic::widget::button::standard(fl!("unhide"))
                .on_press(Message::AppUnhidden(id.clone()));
            section = section.add(cosmic::widget::settings::item_row(vec![
                Self::app_row_icon(app),
                text::body(name).width(Length::Fill).into(),
                unhide.into(),
            ]));
        }
        section
    }

    pub(super) fn moved_apps_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let mut section = cosmic::widget::settings::section().title(fl!("moved-apps"));
        if self.config.app_category_overrides.is_empty() {
            return section.add(text::body(fl!("moved-apps-empty")));
        }
        for (app_id, target_key) in &self.config.app_category_overrides {
            let app = self.apps.iter().find(|a| &a.id == app_id);
            let name = app.map_or_else(|| app_id.clone(), |a| a.name.clone());
            let target_name = Self::category_display_name(target_key, self);
            let reset = cosmic::widget::button::standard(fl!("reset-category"))
                .on_press(Message::AppCategoryReset(app_id.clone()));
            section = section.add(cosmic::widget::settings::item_row(vec![
                Self::app_row_icon(app),
                text::body(format!("{name} \u{2192} {target_name}"))
                    .width(Length::Fill)
                    .into(),
                reset.into(),
            ]));
        }
        section
    }

    /// The icon picker for a custom category's icon; the bundled icons plus
    /// a handful of named ones from the icon theme.
    pub(super) fn category_icon_picker(&self) -> Element<'_, Message> {
        let Some(id) = self.editing_category_icon.clone() else {
            return cosmic::widget::Space::new().width(0).height(0).into();
        };
        let current = self
            .config
            .custom_categories
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.icon.clone())
            .unwrap_or_default();

        let theme = cosmic::theme::active();
        let theme = theme.cosmic();
        let names: Vec<&str> = CATEGORY_ICONS
            .iter()
            .map(|(name, _)| *name)
            .chain(NAMED_ICON_CHOICES.iter().copied())
            .collect();

        let mut grid = cosmic::iced::widget::Column::new().spacing(theme.space_xs());
        for chunk in names.chunks(5) {
            let mut row = cosmic::iced::widget::Row::new().spacing(theme.space_xs());
            for name in chunk {
                let selected = current == *name;
                let icon_el = Self::render_category_icon(&CategoryIcon::from_name(name), 32);
                let id = id.clone();
                let name = (*name).to_string();
                row = row.push(
                    cosmic::widget::button::custom(icon_el)
                        .selected(selected)
                        .padding(theme.space_xs())
                        .on_press(Message::CategoryIconSelected(id, name)),
                );
            }
            grid = grid.push(row);
        }

        cosmic::widget::scrollable(grid).into()
    }

    fn render_category_icon(icon_kind: &CategoryIcon, size: u16) -> Element<'static, Message> {
        match icon_kind {
            CategoryIcon::Bundled(bytes) => cosmic::widget::icon::from_svg_bytes(*bytes)
                .symbolic(true)
                .icon()
                .size(size)
                .into(),
            CategoryIcon::Named(name) => icon::from_name(name.as_str())
                .symbolic(true)
                .icon()
                .size(size)
                .into(),
        }
    }

    /// Display name of a category by its config key: a standard category's
    /// translated name, a custom category's name, or the raw key if it was
    /// since deleted.
    fn category_display_name(key: &str, app: &AppModel) -> String {
        if let Some(category) = ApplicationCategory::STANDARD
            .iter()
            .find(|c| c.mime_name == key)
        {
            return category.get_display_name();
        }
        if let Some(id) = key.strip_prefix(CUSTOM_KEY_PREFIX) {
            if let Some(custom) = app.config.custom_categories.iter().find(|c| c.id == id) {
                return custom.name.clone();
            }
        }
        key.to_string()
    }
}
