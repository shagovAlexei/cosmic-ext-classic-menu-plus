// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use cosmic::cosmic_theme::Spacing;
use cosmic::iced::widget::{column, row};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, ContentFit, Length};
use cosmic::widget::text;
use cosmic::widget::{ListColumn, container, scrollable};
use cosmic::{Element, theme};

use crate::applet::{Applet, Message};
use crate::model::appearance;
use crate::model::application_entry::ApplicationEntry;
use crate::widgets::ScrollAwareContextMenu;

/// A virtualized app list widget that only renders visible items for performance.
///
/// This widget improves performance when dealing with large application lists
/// by only rendering items that are currently visible in the viewport.
pub struct VirtualizedAppList;

impl VirtualizedAppList {
    /// Buffer items to render above/below viewport for smooth scrolling
    const RENDER_BUFFER: usize = 2;

    /// Creates a virtualized app list element
    ///
    /// # Arguments
    /// * `applet` - Reference to the applet containing app data and configuration
    ///
    /// # Returns
    /// A scrollable element containing only the visible app items
    pub fn view(applet: &Applet) -> Element<'_, Message> {
        let Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        if applet.uses_grid() {
            return Self::view_grid(applet);
        }

        let item_height = applet.list_item_height();
        let scroll_offset = applet.scroll_offset;
        let total_items = applet.available_applications.len();

        // Calculate which items should be rendered based on scroll position
        let visible_start = (scroll_offset / item_height).floor() as usize;
        let viewport_height = applet.list_viewport_height();
        let visible_count = ((viewport_height / item_height).ceil() as usize) + 1;

        // Add buffer for smooth scrolling
        let render_start = visible_start.saturating_sub(Self::RENDER_BUFFER);
        let render_end = (visible_start + visible_count + Self::RENDER_BUFFER).min(total_items);

        // Build items to render
        let mut items: Vec<Element<'_, Message>> = Vec::new();

        // Add spacer above visible items to maintain scroll position
        if render_start > 0 {
            let spacer_height = render_start as f32 * item_height;
            items.push(cosmic::widget::Space::new().width(Length::Fill).height(Length::Fixed(spacer_height)).into());
        }

        // Add visible and buffered items
        for (original_index, app) in applet
            .available_applications
            .iter()
            .enumerate()
            .skip(render_start)
            .take(render_end - render_start)
        {
            items.push(Self::create_app_button(
                applet,
                original_index,
                app,
                item_height,
            ));
        }

        // Add spacer below visible items
        if render_end < total_items {
            let remaining_height = (total_items - render_end) as f32 * item_height;
            items.push(cosmic::widget::Space::new().width(Length::Fill).height(Length::Fixed(remaining_height)).into());
        }

        // Build list column from items
        let app_list: ListColumn<Message> = items.into_iter().fold(
            cosmic::widget::list_column()
                .list_item_padding([0., space_xxs as f32]),
            |list, item| list.add(item),
        );

        scrollable(container(app_list))
            .height(Length::Fill)
            .width(Length::FillPortion(5))
            .id(applet.scrollable_id.clone())
            .on_scroll(|viewport| Message::ScrollUpdated(viewport))
            .into()
    }

    /// Grid of icon cells, virtualized by row like the list.
    fn view_grid(applet: &Applet) -> Element<'_, Message> {
        let space_s = theme::active().cosmic().spacing.space_s;
        let columns = applet.grid_columns();
        let row_height = applet.row_height();
        let total_rows = applet.available_applications.len().div_ceil(columns);

        let visible_start = (applet.scroll_offset / row_height).floor() as usize;
        let visible_count = (applet.list_viewport_height() / row_height).ceil() as usize + 1;
        let render_start = visible_start.saturating_sub(Self::RENDER_BUFFER);
        let render_end = (visible_start + visible_count + Self::RENDER_BUFFER).min(total_rows);

        let spacer = |rows: usize| -> Element<'_, Message> {
            cosmic::widget::Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(rows as f32 * row_height))
                .into()
        };

        let mut rows: Vec<Element<'_, Message>> = Vec::new();
        if render_start > 0 {
            rows.push(spacer(render_start));
        }
        for row_index in render_start..render_end {
            let first = row_index * columns;
            let mut cells: Vec<Element<'_, Message>> = applet.available_applications
                [first..(first + columns).min(applet.available_applications.len())]
                .iter()
                .enumerate()
                .map(|(offset, app)| Self::create_grid_cell(applet, first + offset, app, row_height))
                .collect();
            // pad the last row so its cells keep the same width
            while cells.len() < columns {
                cells.push(cosmic::widget::Space::new().width(Length::Fill).into());
            }
            rows.push(cosmic::widget::row::with_children(cells).into());
        }
        if render_end < total_rows {
            rows.push(spacer(total_rows - render_end));
        }

        scrollable(container(cosmic::widget::column::with_children(rows)).padding([0, space_s]))
            .height(Length::Fill)
            .width(Length::FillPortion(5))
            .id(applet.scrollable_id.clone())
            .on_scroll(|viewport| Message::ScrollUpdated(viewport))
            .into()
    }

    /// An app cell of the grid: icon above a one-line name.
    fn create_grid_cell<'a>(
        applet: &'a Applet,
        index: usize,
        app: &'a Arc<ApplicationEntry>,
        cell_height: f32,
    ) -> Element<'a, Message> {
        use cosmic::iced::core::text::EllipsizeHeightLimit;
        use cosmic::iced::widget::text::{Ellipsize, Wrapping};

        let icon_size = appearance::grid_icon_size(applet.list_icon_size());
        let name = text::body(&app.name)
            .wrapping(Wrapping::None)
            .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
            .align_x(Alignment::Center)
            .width(Length::Fill);

        let button = cosmic::widget::button::custom(
            column![Self::create_icon_widget(app, icon_size), name]
                .spacing(4)
                .align_x(Alignment::Center),
        )
        .padding(8)
        .on_press(Message::ApplicationSelected(app.clone()))
        .class(if applet.selected_item_index == Some(index) {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::AppletMenu
        })
        .width(Length::Fill)
        .height(Length::Fixed(cell_height));

        Self::with_context_menu(applet, app, button)
    }

    fn with_context_menu<'a>(
        applet: &'a Applet,
        app: &'a Arc<ApplicationEntry>,
        button: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        // libcosmic's context_menu panics on `None`
        let Some(menu) = Self::create_context_menu(applet, app) else {
            return button.into();
        };
        let widget = cosmic::widget::context_menu(button, Some(menu))
            .close_on_escape(true)
            .on_surface_action(Message::ContextMenuAction)
            .window_id(applet.popup.unwrap_or_else(|| Id::NONE));

        ScrollAwareContextMenu::new(widget, applet.scroll_offset).into()
    }

    /// Creates an individual app button with context menu
    ///
    /// # Arguments
    /// * `applet` - Reference to the applet
    /// * `index` - The index of the app in the list
    /// * `app` - The application entry
    /// * `item_height` - Row height from `Applet::list_item_height`
    ///
    /// # Returns
    /// An element containing a button with context menu
    fn create_app_button<'a>(
        applet: &'a Applet,
        index: usize,
        app: &'a Arc<ApplicationEntry>,
        item_height: f32,
    ) -> Element<'a, Message> {
        let space_xl = theme::active().cosmic().spacing.space_xl;
        let show_comment = appearance::show_comment(applet.config.list_density, space_xl);
        let icon_size = applet.list_icon_size();

        let button = cosmic::widget::button::custom(
            row![
                Self::create_icon_widget(app, icon_size),
                cosmic::widget::Space::new().width(5).height(Length::Fill),
                if show_comment {
                    column![
                        text(&app.name),
                        text(app.comment.as_deref().unwrap_or_default()).size(8.0),
                    ]
                    .padding([0, 0])
                } else {
                    column![text(&app.name)].padding([0, 0])
                },
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::ApplicationSelected(app.clone()))
        .class(
            if applet.selected_item_index.is_some() && index == applet.selected_item_index.unwrap()
            {
                cosmic::theme::Button::Suggested
            } else {
                cosmic::theme::Button::AppletMenu
            },
        )
        .width(Length::Fill)
        .height(Length::Fixed(item_height));

        Self::with_context_menu(applet, app, button)
    }

    /// Creates the icon widget for an application
    ///
    /// # Arguments
    /// * `app` - The application entry
    /// * `icon_size` - The icon dimensions in pixels
    ///
    /// # Returns
    /// A container element with the app icon
    fn create_icon_widget(app: &Arc<ApplicationEntry>, icon_size: u16) -> Element<'_, Message> {
        let default_icon = crate::model::application_entry::IconHandle::default();
        let icon_handle = app.icon.as_ref().unwrap_or(&default_icon);

        match icon_handle {
            crate::model::application_entry::IconHandle::SvgHandle(handle) => container(
                cosmic::widget::svg(handle.clone())
                    .width(Length::Fixed(icon_size.into()))
                    .height(Length::Fixed(icon_size.into()))
                    .content_fit(ContentFit::Contain),
            )
            .into(),
            crate::model::application_entry::IconHandle::RasterHandle(handle) => container(
                cosmic::widget::image(handle.clone())
                    .width(Length::Fixed(icon_size.into()))
                    .height(Length::Fixed(icon_size.into()))
                    .content_fit(ContentFit::Contain),
            )
            .into(),
        }
    }

    /// Creates the context menu for an application
    ///
    /// # Arguments
    /// * `applet` - Reference to the applet
    /// * `app` - The application entry
    ///
    /// # Returns
    /// An optional context menu with app actions
    fn create_context_menu<'a>(
        applet: &'a Applet,
        app: &'a Arc<ApplicationEntry>,
    ) -> Option<Vec<cosmic::widget::menu::Tree<Message>>> {
        // Use cached menu trees if available (built in Applet when apps are updated)
        applet.context_menus.get(&app.id).cloned()
    }
}
