// SPDX-License-Identifier: {{ license }}

mod categories;
mod panel_icon;

use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::dialog::file_chooser::FileFilter;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{
    button, icon, menu,
    menu::{ItemHeight, ItemWidth},
    nav_bar,
};
use cosmic::{Element, widget::text};
use cosmic_ext_classic_menu_plus_applet::config::{
    AppletButtonStyle, AppletConfig, HorizontalPosition, MenuLayout, UserWidgetStyle,
    VerticalPosition,
};
use cosmic_ext_classic_menu_plus_applet::model::appearance::{self, ListDensity};
use cosmic_ext_classic_menu_plus_applet::model::application_entry::{ApplicationEntry, IconHandle};
use cosmic_ext_classic_menu_plus_applet::model::favorites::move_pinned;
use cosmic_ext_classic_menu_plus_applet::model::power_action::{
    MoveDirection, PowerAction, editor_rows, move_power_button, toggle_power_button,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    General,
    Appearance,
    PowerButtons,
    Favorites,
    Categories,
    About,
}

/// Stripe Payment Link: a hosted payment page, no server needed.
const DONATE_URL: &str = "https://donate.stripe.com/aFaeVe9Jjcxb9zA4Rh67S02";

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
    /// Installed apps, loaded when the Favorites or Categories page is first opened.
    apps: Vec<Arc<ApplicationEntry>>,
    /// Id of the custom category whose icon picker is open.
    editing_category_icon: Option<String>,
    /// Installed panel icon files, read once at startup.
    panel_icons: Vec<String>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    UpdateConfig(AppletConfig),
    ResetSettings,
    LaunchUrl(String),
    AppPositionChanged(HorizontalPosition),
    SearchFieldPositionChanged(VerticalPosition),
    AppletButtonStyleChanged(usize),
    UserWidgetChanged(usize),
    ButtonLabelChanged(String),
    ToggleContextPage(ContextPage),
    ButtonIconChanged(PathBuf),
    CustomIconSelected,
    PowerButtonToggled(PowerAction),
    PowerButtonMoved(PowerAction, MoveDirection),
    FavoriteMoved(String, MoveDirection),
    FavoriteRemoved(String),
    PopupWidthChanged(u32),
    PopupHeightChanged(u32),
    AppIconSizeChanged(u16),
    ListDensityChanged(usize),
    ResetAppearance,
    CategoryMoved(String, MoveDirection),
    CategoryVisibilityToggled(String),
    CategoryRenamed(String, String),
    /// A standard or permanent category's key and its new name.
    BuiltinCategoryRenamed(String, String),
    OpenCategoryIconPicker(String),
    CategoryIconSelected(String, String),
    CategoryAdded,
    CategoryRemoved(String),
    AppUnhidden(String),
    AppCategoryReset(String),
    LayoutChanged(MenuLayout),
    SidebarHiddenToggled(bool),
    PowerActionsToggled(bool),
    /// Category key, or empty for automatic.
    DefaultCategoryChanged(String),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    /// Open on the About page.
    type Flags = bool;

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
        let needs_apps = matches!(
            self.nav.active_data::<Page>(),
            Some(&(Page::Favorites | Page::Categories))
        );
        if needs_apps && self.apps.is_empty() {
            self.apps = cosmic_ext_classic_menu_plus_applet::logic::apps::load_apps();
        }
        Task::none()
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        mut core: cosmic::Core,
        open_about: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        core.window.show_maximize = false;

        let about = cosmic::widget::about::About::default()
            .name(fl!("app-title"))
            .icon(icon::from_name(Self::APP_ID))
            .version(env!("CARGO_PKG_VERSION"))
            .author("Shagov Alexei")
            .comments(fl!("about-comments"))
            .license("GPL-3.0-only")
            .license_url("https://www.gnu.org/licenses/gpl-3.0.html")
            .developers([("Shagov Alexei", "shagov.alexei@gmail.com")])
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
        nav.insert()
            .text(fl!("favorites"))
            .icon(icon::from_name("starred-symbolic"))
            .data(Page::Favorites);
        nav.insert()
            .text(fl!("categories"))
            .icon(icon::from_name("view-list-symbolic"))
            .data(Page::Categories);
        nav.insert()
            .text(fl!("about"))
            .icon(icon::from_name("help-about-symbolic"))
            .data(Page::About);
        if open_about {
            let about = nav.iter().last();
            if let Some(id) = about {
                nav.activate(id);
            }
        }

        // Construct the app model with the runtime's core.
        let app = AppModel {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: AppletConfig::config(),
            nav,
            apps: Vec::new(),
            editing_category_icon: None,
            panel_icons: Self::system_icon_names(),
        };

        (app, Task::none())
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        self.core()
            .watch_config::<AppletConfig>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))
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
                self.panel_icon_dropdown(),
            ))
            .add(cosmic::widget::settings::item(
                fl!("monochrome-icon"),
                self.panel_icon_mono_toggle(),
            ));

        let page: Element<'_, Message> = match self.nav.active_data::<Page>() {
            Some(Page::Appearance) => cosmic::widget::column::with_children(vec![
                self.appearance_section().into(),
                self.sidebar_section().into(),
                self.default_menu_section().into(),
                cosmic::widget::button::standard(fl!("default-settings"))
                    .on_press(Message::ResetAppearance)
                    .into(),
            ])
            .spacing(cosmic::theme::active().cosmic().space_s())
            .into(),
            Some(Page::PowerButtons) => self.power_buttons_section().into(),
            Some(Page::Favorites) => self.favorites_section().into(),
            Some(Page::Categories) => cosmic::widget::column::with_children(vec![
                self.categories_section().into(),
                cosmic::widget::button::standard(fl!("new-category"))
                    .on_press(Message::CategoryAdded)
                    .into(),
                self.hidden_apps_section().into(),
                self.moved_apps_section().into(),
            ])
            .spacing(cosmic::theme::active().cosmic().space_s())
            .into(),
            Some(Page::About) => self.about_page(),
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
            ContextPage::About => context_drawer::context_drawer(
                self.about_page(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::CategoryIcon => context_drawer::context_drawer(
                self.category_icon_picker(),
                Message::ToggleContextPage(ContextPage::CategoryIcon),
            )
            .title(fl!("category-icon")),
        })
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            // Changes made elsewhere (the applet writes favorites and recent apps).
            // Never write back here, or we would echo and clobber newer data.
            Message::UpdateConfig(config) => {
                self.config = config;
                Task::none()
            }
            Message::ResetSettings => {
                // Favorites and categories are curated by hand, so a reset keeps them.
                let pinned_apps = std::mem::take(&mut self.config.pinned_apps);
                let custom_categories = std::mem::take(&mut self.config.custom_categories);
                let app_category_overrides =
                    std::mem::take(&mut self.config.app_category_overrides);
                let hidden_apps = std::mem::take(&mut self.config.hidden_apps);
                let hidden_categories = std::mem::take(&mut self.config.hidden_categories);
                let category_order = std::mem::take(&mut self.config.category_order);
                self.config = AppletConfig {
                    pinned_apps,
                    custom_categories,
                    app_category_overrides,
                    hidden_apps,
                    hidden_categories,
                    category_order,
                    ..AppletConfig::default()
                };
                self.write_config("default settings");
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
                // One step below the minimum means "auto" (follow the theme).
                self.config.app_icon_size = if value < appearance::ICON_SIZE_RANGE.0 {
                    appearance::DEFAULT_ICON_SIZE
                } else {
                    appearance::clamp_icon_size(value)
                };
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
            Message::ResetAppearance => {
                self.config.popup_width = appearance::DEFAULT_POPUP_WIDTH;
                self.config.popup_height = appearance::DEFAULT_POPUP_HEIGHT;
                self.config.app_icon_size = appearance::DEFAULT_ICON_SIZE;
                self.config.list_density = ListDensity::default();
                self.config.layout = MenuLayout::default();
                self.write_config("appearance defaults");
                Task::none()
            }
            Message::CategoryMoved(key, direction) => {
                cosmic_ext_classic_menu_plus_applet::logic::categories::move_category(
                    &mut self.config,
                    &key,
                    direction,
                );
                self.write_config("category order");
                Task::none()
            }
            Message::CategoryVisibilityToggled(key) => {
                cosmic_ext_classic_menu_plus_applet::logic::categories::toggle_category_visibility(
                    &mut self.config,
                    &key,
                );
                self.write_config("category visibility");
                Task::none()
            }
            Message::CategoryRenamed(id, name) => {
                if let Some(category) = self
                    .config
                    .custom_categories
                    .iter_mut()
                    .find(|c| c.id == id)
                {
                    category.name = name;
                }
                self.write_config("category name");
                Task::none()
            }
            Message::BuiltinCategoryRenamed(key, name) => {
                if name.trim().is_empty() {
                    self.config.category_names.remove(&key);
                } else {
                    self.config.category_names.insert(key, name);
                }
                self.write_config("category name");
                Task::none()
            }
            Message::OpenCategoryIconPicker(id) => {
                self.editing_category_icon = Some(id);
                self.context_page = ContextPage::CategoryIcon;
                self.core.window.show_context = true;
                Task::none()
            }
            Message::CategoryIconSelected(id, icon_name) => {
                if let Some(category) = self
                    .config
                    .custom_categories
                    .iter_mut()
                    .find(|c| c.id == id)
                {
                    category.icon = icon_name;
                }
                self.core.window.show_context = false;
                self.editing_category_icon = None;
                self.write_config("category icon");
                Task::none()
            }
            Message::CategoryAdded => {
                let id = cosmic_ext_classic_menu_plus_applet::logic::categories::next_custom_id(
                    &self.config.custom_categories,
                );
                self.config.custom_categories.push(
                    cosmic_ext_classic_menu_plus_applet::model::application_category::CustomCategory {
                        id,
                        name: fl!("new-category"),
                        icon: "folder-symbolic".to_string(),
                    },
                );
                self.write_config("new category");
                Task::none()
            }
            Message::CategoryRemoved(id) => {
                cosmic_ext_classic_menu_plus_applet::logic::categories::remove_custom_category(
                    &mut self.config,
                    &id,
                );
                self.write_config("category removed");
                Task::none()
            }
            Message::AppUnhidden(id) => {
                self.config.hidden_apps.retain(|a| a != &id);
                self.write_config("unhidden app");
                Task::none()
            }
            Message::AppCategoryReset(id) => {
                self.config.app_category_overrides.remove(&id);
                self.write_config("reset app category");
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
            Message::LayoutChanged(layout) => {
                self.config.layout = layout;
                self.write_config("layout");
                Task::none()
            }
            Message::SidebarHiddenToggled(hidden) => {
                self.config.sidebar_hidden_by_default = hidden;
                self.write_config("sidebar hidden by default");
                Task::none()
            }
            Message::PowerActionsToggled(show) => {
                self.config.show_power_actions = show;
                self.write_config("show power actions");
                Task::none()
            }
            Message::DefaultCategoryChanged(key) => {
                self.config.default_category = key;
                self.write_config("default category");
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

    /// Icon of an installed app for a settings row; a blank placeholder if
    /// the app is no longer installed.
    pub(crate) fn app_row_icon(app: Option<&Arc<ApplicationEntry>>) -> Element<'static, Message> {
        match app.and_then(|a| a.icon.as_ref()) {
            Some(IconHandle::SvgHandle(h)) => cosmic::widget::svg(h.clone())
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .into(),
            Some(IconHandle::RasterHandle(h)) => cosmic::widget::image(h.clone())
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .into(),
            None => cosmic::widget::Space::new().width(24).height(24).into(),
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
        let auto_icon = self.config.app_icon_size == appearance::DEFAULT_ICON_SIZE;
        let auto_step = appearance::ICON_SIZE_RANGE.0 - 2;
        let icon_size = if auto_icon {
            auto_step
        } else {
            appearance::clamp_icon_size(self.config.app_icon_size)
        };
        let icon_label = if auto_icon {
            fl!("icon-size-auto")
        } else {
            icon_size.to_string()
        };
        let density = match self.config.list_density {
            ListDensity::Compact => 0,
            ListDensity::Normal => 1,
        };

        let layout_radio = |label: String, layout: MenuLayout| {
            cosmic::widget::Radio::new(
                text::body(label),
                layout,
                Some(self.config.layout),
                Message::LayoutChanged,
            )
        };
        let layout = cosmic::iced::widget::row![
            layout_radio(fl!("layout-grid"), MenuLayout::Grid),
            layout_radio(fl!("layout-list"), MenuLayout::List),
            layout_radio(fl!("layout-hybrid"), MenuLayout::Hybrid),
        ]
        .spacing(cosmic::theme::active().cosmic().space_m());

        cosmic::widget::settings::section()
            .title(fl!("appearance"))
            .add(
                cosmic::widget::settings::item::builder(fl!("layout"))
                    .description(fl!("layout-description"))
                    .control(layout),
            )
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
                    icon_label,
                    fl!("app-icon-size"),
                    icon_size,
                    2,
                    auto_step,
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
            let app_icon = Self::app_row_icon(app);
            let up = cosmic::widget::button::icon(icon::from_name("go-up-symbolic"))
                .on_press_maybe(
                    (index > 0).then(|| Message::FavoriteMoved(id.clone(), MoveDirection::Up)),
                );
            let down = cosmic::widget::button::icon(icon::from_name("go-down-symbolic"))
                .on_press_maybe(
                    (index + 1 < pinned.len())
                        .then(|| Message::FavoriteMoved(id.clone(), MoveDirection::Down)),
                );
            let remove = cosmic::widget::button::standard(fl!("favorite-remove"))
                .on_press(Message::FavoriteRemoved(id.clone()));
            let controls = cosmic::iced::widget::row![up, down, remove]
                .spacing(space_xxs)
                .align_y(Alignment::Center);
            section = section.add(cosmic::widget::settings::item_row(vec![
                app_icon,
                text::body(name).width(Length::Fill).into(),
                controls.into(),
            ]));
        }
        section
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
            let dir = format!(
                "{}/cosmic/{}/applet-buttons",
                data_dir,
                cosmic_ext_classic_menu_plus_applet::applet::APP_ID
            );
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

    /// Toggles for what the menu shows besides the app list.
    fn sidebar_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let shown = |key: &str| !self.config.hidden_categories.iter().any(|k| k == key);
        let category_toggle = |label: String, key: &'static str| {
            cosmic::widget::settings::item(
                label,
                cosmic::widget::toggler(shown(key))
                    .on_toggle(move |_| Message::CategoryVisibilityToggled(key.to_string())),
            )
        };

        cosmic::widget::settings::section()
            .title(fl!("sidebar"))
            .add(category_toggle(
                fl!("show-favorites"),
                "favorites",
            ))
            .add(category_toggle(
                fl!("show-recently-used"),
                "recently-used",
            ))
            .add(cosmic::widget::settings::item(
                fl!("show-power-actions"),
                cosmic::widget::toggler(self.config.show_power_actions)
                    .on_toggle(Message::PowerActionsToggled),
            ))
            .add(
                cosmic::widget::settings::item::builder(fl!("hide-sidebar"))
                    .description(fl!("hide-sidebar-description"))
                    .toggler(self.config.sidebar_hidden_by_default, Message::SidebarHiddenToggled),
            )
    }

    /// The category the menu opens on.
    fn default_menu_section(&self) -> cosmic::widget::settings::Section<'_, Message> {
        let (mut keys, mut labels) = (vec![String::new()], vec![fl!("default-category-auto")]);
        for (category, visible) in cosmic_ext_classic_menu_plus_applet::logic::categories::editor_rows(&self.config) {
            if !visible {
                continue;
            }
            let key = category.key.to_string();
            labels.push(
                self.config
                    .category_names
                    .get(&key)
                    .filter(|name| !name.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| category.get_display_name()),
            );
            keys.push(key);
        }
        let selected = keys.iter().position(|k| *k == self.config.default_category);

        cosmic::widget::settings::section()
            .title(fl!("default-menu"))
            .add(
                cosmic::widget::settings::item::builder(fl!("default-category"))
                    .description(fl!("default-category-description"))
                    .control(cosmic::widget::dropdown(labels, selected.or(Some(0)), move |index| {
                        Message::DefaultCategoryChanged(keys[index].clone())
                    })),
            )
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
            MenuAction::SetDefaultSettings => Message::ResetSettings,
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    CategoryIcon,
}

impl AppModel {
    /// Donate block over the about info, shown both as a page and in the header menu's drawer.
    fn about_page(&self) -> Element<'_, Message> {
        cosmic::widget::column::with_children(vec![
            donate(),
            cosmic::widget::about(&self.about, |url| Message::LaunchUrl(url.to_string())),
        ])
        .spacing(24)
        .into()
    }
}

fn donate() -> Element<'static, Message> {
    cosmic::widget::column::with_capacity(3)
        .spacing(8)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .push(text::heading(fl!("donate-title")))
        .push(text::body(fl!("donate-text")))
        .push(
            button::suggested(fl!("donate-title"))
                .on_press(Message::LaunchUrl(DONATE_URL.to_string())),
        )
        .into()
}
