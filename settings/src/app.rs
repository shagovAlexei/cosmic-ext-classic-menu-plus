// SPDX-License-Identifier: {{ license }}

use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::dialog::file_chooser::FileFilter;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, icon, menu, nav_bar, menu::{ItemWidth, ItemHeight}};
use cosmic::{iced::Background, widget::text, Element};
use cosmic_ext_classic_menu_plus_applet::config::{
    AppletButtonStyle, AppletConfig, HorizontalPosition, UserWidgetStyle,
    VerticalPosition,
};
use cosmic_ext_classic_menu_plus_applet::model::appearance::{self, ListDensity};
use cosmic_ext_classic_menu_plus_applet::model::power_action::{
    editor_rows, move_power_button, toggle_power_button, MoveDirection, PowerAction,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    General,
    Appearance,
    PowerButtons,
}

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The about page for this app.
    about: cosmic::widget::about::About,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config: AppletConfig,
    /// Navigation bar with the settings pages.
    nav: nav_bar::Model,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    UpdateConfig(AppletConfig),
    LaunchUrl(String),
    AppPositionChanged(HorizontalPosition),
    SearchFieldPositionChanged(VerticalPosition),
    AppletButtonStyleChanged(usize),
    UserWidgetChanged(usize),
    ButtonLabelChanged(String),
    ToggleContextPage(ContextPage),
    OpenIconPicker,
    ButtonIconChanged(PathBuf),
    CustomIconSelected,
    PowerButtonToggled(PowerAction),
    PowerButtonMoved(PowerAction, MoveDirection),
    PopupWidthChanged(u32),
    PopupHeightChanged(u32),
    AppIconSizeChanged(u16),
    ListDensityChanged(usize),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = cosmic_ext_classic_menu_plus_applet::applet::APP_ID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        Task::none()
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        mut core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        core.window.show_maximize = false;

        let about = cosmic::widget::about::About::default()
            .name(fl!("app-title"))
            .icon(icon::from_name(Self::APP_ID))
            .version(env!("CARGO_PKG_VERSION"))
            .license("GPL-3.0-only")
            .developers([("Kamil Lihan", "k.lihan@outlook.com")])
            .links([
                (
                    fl!("repository"),
                    "https://github.com/shagovAlexei/cosmic-ext-classic-menu-plus",
                ),
                (
                    fl!("support"),
                    "https://github.com/shagovAlexei/cosmic-ext-classic-menu-plus/issues",
                ),
            ]);

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

        // Construct the app model with the runtime's core.
        let app = AppModel {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: AppletConfig::config(),
            nav,
        };

        (app, Task::none())
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&'_ self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("settings")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(
                        fl!("default-settings"),
                        None,
                        MenuAction::SetDefaultSettings,
                    ),
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                ],
            ),
        )])
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(240));

        vec![menu_bar.into()]
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&'_ self) -> Element<'_, Self::Message> {
        let app_menu_position = cosmic::iced::widget::row![
            cosmic::widget::Radio::new(
                cosmic::widget::text::heading(fl!("left")),
                HorizontalPosition::Left,
                Some(self.config.app_menu_position),
                Message::AppPositionChanged
            ),
            cosmic::widget::Space::new().width(5).height(Length::Shrink),
            cosmic::widget::Radio::new(
                cosmic::widget::text::heading(fl!("right")),
                HorizontalPosition::Right,
                Some(self.config.app_menu_position),
                Message::AppPositionChanged
            )
        ];
        let search_field_position = cosmic::iced::widget::row![
            cosmic::widget::Space::new().width(Length::Fill).height(5),
            cosmic::widget::Radio::new(
                cosmic::widget::text::heading(fl!("top")),
                VerticalPosition::Top,
                Some(self.config.search_field_position),
                Message::SearchFieldPositionChanged
            ),
            cosmic::widget::Space::new().width(5).height(Length::Shrink),
            cosmic::widget::Radio::new(
                cosmic::widget::text::heading(fl!("bottom")),
                VerticalPosition::Bottom,
                Some(self.config.search_field_position),
                Message::SearchFieldPositionChanged
            )
        ];
        let applet_button_style = cosmic::iced::widget::row![
            cosmic::widget::Space::new().width(Length::Fill).height(5),
            cosmic::widget::dropdown(
                vec![
                    fl!("icon-only"),
                    fl!("label-only"),
                    fl!("icon-and-label"),
                    fl!("auto")
                ],
                Some(self.config.applet_button_style as usize),
                Message::AppletButtonStyleChanged
            )
        ];
        let user_widget = cosmic::iced::widget::row![
            cosmic::widget::Space::new().width(Length::Fill).height(5),
            cosmic::widget::dropdown(
                vec![
                    fl!("username-prefered"),
                    fl!("realname-prefered"),
                    fl!("none")
                ],
                Some(self.config.user_widget as usize),
                Message::UserWidgetChanged
            )
        ];
        let button_label = cosmic::iced::widget::row![
            cosmic::widget::Space::new().width(Length::Fill).height(5),
            cosmic::widget::text_input(fl!("button-label-placeholder"), &self.config.button_label)
                .on_input(Message::ButtonLabelChanged)
        ];
        let button_icon = cosmic::iced::widget::row![
            cosmic::widget::Space::new().width(Length::Fill).height(5),
            cosmic::widget::button::text(fl!("button-icon-placeholder"))
                .on_press(Message::OpenIconPicker) // 4. Open picker on click
        ];

        let general_section = cosmic::widget::settings::section()
                .title(fl!("general"))
                .add(cosmic::widget::settings::item(
                    fl!("app-menu-position"),
                    app_menu_position,
                ))
                .add(cosmic::widget::settings::item(
                    fl!("search-field-position"),
                    search_field_position,
                ))
                .add(cosmic::widget::settings::item(
                    fl!("applet-button-style"),
                    applet_button_style,
                ))
                .add(cosmic::widget::settings::item(
                    fl!("user-widget"),
                    user_widget,
                ))
                .add(cosmic::widget::settings::item(
                    fl!("button-label"),
                    button_label,
                ))
                .add(cosmic::widget::settings::item(
                    fl!("button-icon"),
                    button_icon,
                ))
                ;

        let page: Element<'_, Message> = match self.nav.active_data::<Page>() {
            Some(Page::Appearance) => self.appearance_section().into(),
            Some(Page::PowerButtons) => self.power_buttons_section().into(),
            _ => general_section.into(),
        };
        let settings_container = cosmic::widget::settings::view_column(vec![page]);

        cosmic::widget::scrollable(settings_container.padding([5, 10])).into()
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&'_ self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::IconPicker => context_drawer::context_drawer(
                self.icon_picker(), // 3. Show icon picker
                Message::ToggleContextPage(ContextPage::IconPicker),
            )
            .title(fl!("button-icon")),
        })
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write recent applications config");

                Task::none()
            }
            Message::LaunchUrl(url) => {
                match open::that_detached(&url) {
                    Ok(()) => {}
                    Err(err) => {
                        log::error!("failed to open {url:?}: {err}");
                    }
                }
                Task::none()
            }
            Message::AppPositionChanged(horizontal_position) => {
                log::info!("App position changed to: {:?}", horizontal_position);
                self.config.app_menu_position = horizontal_position;

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write recent applications config");

                Task::none()
            }
            Message::SearchFieldPositionChanged(vertical_position) => {
                log::info!("Search field position changed to: {:?}", vertical_position);
                self.config.search_field_position = vertical_position;

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write search field position config");

                Task::none()
            }
            Message::AppletButtonStyleChanged(applet_button_style) => {
                log::info!("Applet button style changed to: {:?}", applet_button_style);
                self.config.applet_button_style = match applet_button_style {
                    0 => AppletButtonStyle::IconOnly,
                    1 => AppletButtonStyle::LabelOnly,
                    2 => AppletButtonStyle::IconAndLabel,
                    3 => AppletButtonStyle::Auto,
                    _ => AppletButtonStyle::Auto,
                };

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write applet button style config");

                Task::none()
            }
            Message::UserWidgetChanged(user_widget_style) => {
                log::info!("User widget style changed to: {:?}", user_widget_style);
                self.config.user_widget = match user_widget_style {
                    0 => UserWidgetStyle::UsernamePrefered,
                    1 => UserWidgetStyle::RealNamePrefered,
                    2 => UserWidgetStyle::None,
                    _ => UserWidgetStyle::None,
                };

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write user widget style config");

                Task::none()
            }
            Message::ButtonLabelChanged(new_label) => {
                let mut new_label = new_label;
                if new_label.len() == 0 {
                    // If the field is empty, reset to default.
                    new_label = AppletConfig::default().button_label;
                }

                log::info!("Button label changed to: {:?}", new_label);
                self.config.button_label = new_label;

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write button label config");

                Task::none()
            }
            Message::ButtonIconChanged(new_icon) => {
                log::info!(
                    "Button icon changed to: {:?}",
                    new_icon.clone().to_string_lossy()
                );
                self.config.button_icon = new_icon.to_string_lossy().into_owned();

                self.config
                    .write_entry(AppletConfig::config_handler().as_ref().unwrap())
                    .expect("Failed to write button icon config");

                Task::none()
            }
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
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                Task::none()
            }
            Message::OpenIconPicker => {
                self.context_page = ContextPage::IconPicker;
                self.core.window.show_context = true;

                Task::none()
            }
            Message::CustomIconSelected => Task::perform(AppModel::pick_custom_icon(), |res| {
                if let Some(icon_pathbuf) = res {
                    // Icon exists and was selected
                    cosmic::action::app(Message::ButtonIconChanged(icon_pathbuf))
                } else {
                    // Icon selection was cancelled or failed, revert to default
                    cosmic::action::none()
                }
            }),
        }
    }
}

impl AppModel {
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

    fn write_config(&self, what: &str) {
        if let Err(err) = self
            .config
            .write_entry(AppletConfig::config_handler().as_ref().unwrap())
        {
            log::error!("failed to write {what}: {err}");
        }
    }

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

    /// Helper to find available system icons in standard locations.
    fn system_icon_names() -> Vec<String> {
        // Prefer runtime discovery using XDG_DATA_DIRS so the app works correctly
        // inside Flatpak (where icons live under /app/share) as well as on
        // a system installation (under /usr/share).
        let mut icons: Vec<String> = Vec::new();

        // Build a list of candidate data dirs from XDG_DATA_DIRS. If the
        // variable isn't set, fall back to common locations including /usr and
        // /app so we cover both host and Flatpak runtimes.
        let mut candidate_dirs: Vec<String> = Vec::new();
        if let Ok(xdg) = std::env::var("XDG_DATA_DIRS") {
            for part in xdg.split(':') {
                let part = part.trim_end_matches('/');
                if !part.is_empty() {
                    candidate_dirs.push(part.to_string());
                }
            }
        } else {
            candidate_dirs.push("/usr/share".to_string());
            candidate_dirs.push("/app/share".to_string());
        }

        for data_dir in candidate_dirs {
            let dir = format!("{}/cosmic/{}/applet-buttons", data_dir, cosmic_ext_classic_menu_plus_applet::applet::APP_ID);
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "svg" || ext == "png" {
                            if let Ok(abs_path) = path.canonicalize() {
                                icons.push(abs_path.to_string_lossy().into_owned());
                            }
                        }
                    }
                }
            }
        }

        icons.sort();
        icons.dedup();
        icons
    }

    async fn pick_custom_icon() -> Option<PathBuf> {
        if let Ok(result) = cosmic::dialog::file_chooser::open::Dialog::new()
            .title(fl!("select-custom-icon"))
            .accept_label(fl!("select"))
            .current_filter(FileFilter::new("icon-file").glob("*.svg").glob("*.png"))
            .open_file()
            .await
        {
            let icon_pathbuf: PathBuf = PathBuf::from(result.0.uris()[0].path());
            if icon_pathbuf.exists() {
                return Some(icon_pathbuf);
            } else {
                return None;
            }
        }

        None
    }

    pub fn icon_picker(&'_ self) -> Element<'_, Message> {
        let mut icons = Self::system_icon_names();
        let icons_per_row = 3;
        let theme = cosmic::theme::active();
        let theme = theme.cosmic();

        let mut grid = cosmic::iced::widget::Column::new().spacing(theme.space_xs());

        // handle custom icon selection
        let currently_selected_icon: PathBuf = PathBuf::from(&self.config.button_icon);
        if currently_selected_icon.exists()
            && !icons.contains(&currently_selected_icon.to_string_lossy().into_owned())
        {
            icons.insert(0, currently_selected_icon.to_string_lossy().into_owned());
        }

        // Add default set of icons
        for chunk in icons.chunks(icons_per_row) {
            let mut row = cosmic::iced::widget::Row::new().spacing(theme.space_xs());
            for icon_path in chunk {
                let icon_pathbuf: PathBuf = icon_path.into();
                let icon_name = icon_pathbuf
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                row = row.push(Self::button(
                    &icon_name,
                    icon_pathbuf,
                    self.config.button_icon == *icon_path,
                    Message::ButtonIconChanged,
                ));
            }
            grid = grid.push(row);
        }

        let custom_icon_button = cosmic::widget::button::standard(fl!("select-custom-icon"))
            .on_press(Message::CustomIconSelected);

        cosmic::iced::widget::column![custom_icon_button, grid]
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .spacing(theme.space_xs())
            .into()
    }

    fn button(
        name: &String,
        icon_pathbuf: PathBuf,
        selected: bool,
        callback: impl Fn(PathBuf) -> Message,
    ) -> Element<'static, Message> {
        const ICON_THUMB_SIZE: u16 = 32;
        const ICON_NAME_TRUNC: usize = 20;

        let theme = cosmic::theme::active();
        let theme = theme.cosmic();
        let background = Background::Color(theme.palette.neutral_4.into());

        cosmic::iced::widget::column::Column::new()
            .push(
                cosmic::widget::button::custom_image_button(
                    cosmic::widget::icon::from_path(icon_pathbuf.clone())
                        .icon()
                        .size(ICON_THUMB_SIZE),
                    None,
                )
                .on_press(callback(icon_pathbuf.clone()))
                .selected(selected)
                .padding(theme.space_xs())
                // Image button's style mostly works, but it needs a background to fit the design
                .class(button::ButtonClass::Custom {
                    active: Box::new(move |focused, theme| {
                        let mut appearance = <cosmic::theme::Theme as button::Catalog>::active(
                            theme,
                            focused,
                            selected,
                            &cosmic::theme::Button::Image,
                        );
                        appearance.background = Some(background);
                        appearance
                    }),
                    disabled: Box::new(move |theme| {
                        let mut appearance = <cosmic::theme::Theme as button::Catalog>::disabled(
                            theme,
                            &cosmic::theme::Button::Image,
                        );
                        appearance.background = Some(background);
                        appearance
                    }),
                    hovered: Box::new(move |focused, theme| {
                        let mut appearance = <cosmic::theme::Theme as button::Catalog>::hovered(
                            theme,
                            focused,
                            selected,
                            &cosmic::theme::Button::Image,
                        );
                        appearance.background = Some(background);
                        appearance
                    }),
                    pressed: Box::new(move |focused, theme| {
                        let mut appearance = <cosmic::theme::Theme as button::Catalog>::pressed(
                            theme,
                            focused,
                            selected,
                            &cosmic::theme::Button::Image,
                        );
                        appearance.background = Some(background);
                        appearance
                    }),
                }),
            )
            .push(
                text::body(if name.len() > ICON_NAME_TRUNC {
                    format!("{name:.ICON_NAME_TRUNC$}...")
                } else {
                    name.into()
                })
                .width(Length::Fixed((ICON_THUMB_SIZE * 3) as _)),
            )
            .spacing(theme.space_xxs())
            .into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    SetDefaultSettings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::SetDefaultSettings => {
                Message::UpdateConfig(AppletConfig::default())
            }
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    IconPicker, // 1. Add new variant
}
