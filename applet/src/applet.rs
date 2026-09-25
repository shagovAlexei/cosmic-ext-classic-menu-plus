// SPDX-License-Identifier: GPL-3.0-only

use cached::Cached;
use cosmic::app::{Core, Task};
use cosmic::applet::cosmic_panel_config::PanelAnchor;
use cosmic::cctk::sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{
    Anchor, Gravity,
};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::iced::event::listen_raw;
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::widget::operation::AbsoluteOffset;
use cosmic::iced::widget::scrollable::{RelativeOffset, Viewport};
use cosmic::iced::{
    Alignment,
    platform_specific::shell::commands::popup::{destroy_popup, get_popup},
    widget::{column, row},
    window::Id,
};
use cosmic::iced::{Subscription, keyboard};
use cosmic::surface::Action;
use cosmic::{Application, Element};
use cosmic_app_list_config::AppListConfig;
use std::process;
use std::sync::Arc;

use crate::applet_button::AppletButton;
use crate::applet_menu::AppletMenu;
use crate::config::{AppletButtonStyle, AppletConfig, RecentApplication};
use crate::fl;
use crate::logic::apps::{Event, desktop_files};
use crate::model::application_category::ApplicationCategory;
use crate::model::application_entry::{ApplicationEntry, DesktopAction};
use crate::model::popup_type::PopupType;
use crate::model::power_action::PowerAction;
use crate::model::system_tool::SystemTool;
use crate::model::user::User;

pub const APP_ID: &str = "io.github.shagovAlexei.cosmic-ext-classic-menu-plus";

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct Applet {
    /// Application state which is managed by the COSMIC runtime.
    pub core: Core,
    /// The popup id.
    pub popup: Option<Id>,
    /// The configuration that is used to store the application settings.
    pub config: AppletConfig,
    /// The search field that is used to filter the applications.
    pub search_field: String,
    /// The list of available applications that are displayed in the menu.
    pub available_applications: Vec<Arc<ApplicationEntry>>,
    /// The list of available categories that are displayed in the menu.
    pub available_categories: Vec<ApplicationCategory>,
    /// The popup type that is used to determine which popup to display.
    pub popup_type: PopupType,
    /// The selected category that is used to filter the applications.
    pub selected_category: Option<ApplicationCategory>,
    /// Currently logged user
    pub current_user: Option<User>,
    /// Currently selected item
    pub selected_item_index: Option<usize>,
    /// Scrollable ID for keyboard navigation
    pub scrollable_id: cosmic::widget::Id,
    /// List of pinned apps
    pub app_list_config: AppListConfig,
    /// Cached context menus for applications (built once when apps are loaded)
    pub context_menus: std::collections::HashMap<String, Vec<cosmic::widget::menu::Tree<Message>>>,
    /// Categories offered in the per-app "Move to..." panel.
    pub move_targets: Vec<ApplicationCategory>,
    /// Index of the app whose "Move to..." category panel is shown in place
    /// of the categories pane.
    pub moving_app: Option<usize>,
    /// Scroll offset for virtualization (pixels from top)
    pub scroll_offset: f32,
    /// Viewport height for virtualization and selection scroll behavior.
    pub scroll_viewport_height: f32,
    /// Whether logind reports hibernation as available.
    pub can_hibernate: bool,
    /// Power action waiting for in-popup confirmation.
    pub pending_confirmation: Option<(PowerAction, std::time::Instant)>,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup(PopupType),
    PopupClosed(Id),
    SearchFieldInput(String),
    SearchCleared,
    PowerOptionSelected(PowerAction),
    HibernateSupport(bool),
    ConfirmPowerAction,
    CancelPowerAction,
    ApplicationSelected(Arc<ApplicationEntry>),
    CategorySelected(ApplicationCategory),
    LaunchTool(SystemTool),
    Zbus(Result<(), zbus::Error>),
    UpdateLoggedUser(Result<User, zbus::Error>),
    FileEvent(Event),
    UpdateConfig(AppletConfig),
    UpdateAvailableApplications(Vec<Arc<ApplicationEntry>>),
    UpdateAvailableCategories(Vec<ApplicationCategory>),
    SelectPreviousApp,
    SelectNextApp,
    LaunchSelectedApplication,
    SuperKeyPressed,
    AppListConfigUpdated(AppListConfig),
    ContextMenuAction(Action),
    LaunchApplicationAt(usize),
    LaunchApplicationWithActionAt(usize, usize),
    PinToAppTrayIndex(usize, bool),
    ToggleFavoriteAt(usize),
    ChooseCategoryFor(usize),
    CancelChooseCategory,
    MoveApplicationToCategory(usize, usize),
    ResetApplicationCategory(usize),
    HideApplicationAt(usize),
    ScrollUpdated(Viewport),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for Applet {
    type Executor = cosmic::executor::multi::Executor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Task` type is used to send messages to your application. `Task::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = AppletConfig::config();
        let move_targets = crate::logic::categories::move_targets(&config);
        let window = Applet {
            core,
            search_field: "".to_owned(),
            popup_type: PopupType::MainMenu,
            selected_category: Some(ApplicationCategory::ALL),
            config,
            current_user: None,
            selected_item_index: None,
            scrollable_id: cosmic::widget::Id::unique(),
            app_list_config: Default::default(),
            available_applications: Vec::new(),
            available_categories: Vec::new(),
            popup: None,
            context_menus: std::collections::HashMap::new(),
            move_targets,
            moving_app: None,
            scroll_offset: 0.0,
            scroll_viewport_height: 0.0,
            can_hibernate: false,
            pending_confirmation: None,
        };

        // fetch current user asynchronously
        let fetch_current_user_task =
            Task::perform(crate::model::user::get_current_user(), |result| {
                cosmic::Action::App(Message::UpdateLoggedUser(result))
            });

        let fetch_all_apps_task = Task::perform(
            tokio::task::spawn_blocking(|| crate::logic::apps::load_apps()),
            |res| cosmic::Action::App(Message::UpdateAvailableApplications(res.unwrap())),
        );

        let categories_config = window.config.clone();
        let fetch_available_categories_task = Task::perform(
            tokio::task::spawn_blocking(move || {
                crate::logic::apps::load_app_categories(&categories_config)
            }),
            |res| cosmic::Action::App(Message::UpdateAvailableCategories(res.unwrap())),
        );

        let fetch_can_hibernate_task =
            Task::perform(crate::power_options::can_hibernate(), |available| {
                cosmic::Action::App(Message::HibernateSupport(available))
            });

        (
            window,
            Task::batch(vec![
                fetch_current_user_task,
                fetch_can_hibernate_task,
                fetch_all_apps_task,
                fetch_available_categories_task,
            ]),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<'_, Message> {
        let applet_button_style = &self.config.applet_button_style;
        let panel_type = &self.core.applet.panel_type;
        let size = &self.core.applet.size;

        match applet_button_style {
            AppletButtonStyle::IconOnly => AppletButton::view_icon_only(&self),
            AppletButtonStyle::LabelOnly => AppletButton::view_label_only(&self),
            AppletButtonStyle::IconAndLabel => AppletButton::view_icon_and_label(&self),
            AppletButtonStyle::Auto => match panel_type {
                cosmic::applet::PanelType::Panel => match size {
                    cosmic::applet::Size::Hardcoded(hardcoded_size) => {
                        if hardcoded_size.0
                            < cosmic::applet::cosmic_panel_config::PanelSize::M
                                .get_applet_icon_size(false) as u16
                        {
                            AppletButton::view_label_only(&self)
                        } else {
                            AppletButton::view_icon_only(&self)
                        }
                    }
                    cosmic::applet::Size::PanelSize(panel_size) => match panel_size {
                        cosmic::applet::cosmic_panel_config::PanelSize::XS
                        | cosmic::applet::cosmic_panel_config::PanelSize::S => {
                            AppletButton::view_label_only(&self)
                        }
                        cosmic::applet::cosmic_panel_config::PanelSize::M
                        | cosmic::applet::cosmic_panel_config::PanelSize::L
                        | cosmic::applet::cosmic_panel_config::PanelSize::XL
                        | cosmic::applet::cosmic_panel_config::PanelSize::Custom(_) => {
                            AppletButton::view_icon_only(&self)
                        }
                    },
                },
                cosmic::applet::PanelType::Dock | cosmic::applet::PanelType::Other(_) => {
                    AppletButton::view_icon_only(&self)
                }
            },
        }
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        match self.popup_type {
            PopupType::MainMenu => self.view_main_menu(),
            PopupType::ContextMenu => self.view_context_menu(),
        }
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Tasks may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup(popup_type) => self.toggle_popup(popup_type),
            Message::PopupClosed(id) => self.close_popup(id),
            Message::SearchFieldInput(input) => {
                self.moving_app = None;
                self.update_search_field(input)
            }
            Message::SearchCleared => {
                self.moving_app = None;
                self.clear_search()
            }
            Message::PowerOptionSelected(action) => self.perform_power_action(action),
            Message::HibernateSupport(available) => {
                self.can_hibernate = available;
                Task::none()
            }
            Message::ConfirmPowerAction => match self.pending_confirmation {
                Some((_, shown))
                    if !crate::model::power_action::confirmation_ready(
                        shown,
                        std::time::Instant::now(),
                    ) =>
                {
                    Task::none()
                }
                Some((action, _)) => {
                    self.pending_confirmation = None;
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
            Message::ApplicationSelected(app) => self.launch_application(app, None),
            Message::CategorySelected(category) => {
                self.moving_app = None;
                self.select_category(category)
            }
            Message::LaunchTool(tool) => self.launch_tool(tool),
            Message::Zbus(result) => self.handle_zbus_result(result),
            Message::UpdateLoggedUser(user) => {
                self.current_user = user.ok();
                Task::none()
            }
            Message::FileEvent(event) => self.handle_event(event),
            Message::UpdateConfig(config) => {
                let pinned_changed = self.config.pinned_apps != config.pinned_apps;
                let categories_changed = self.config.custom_categories != config.custom_categories
                    || self.config.app_category_overrides != config.app_category_overrides
                    || self.config.hidden_apps != config.hidden_apps
                    || self.config.hidden_categories != config.hidden_categories
                    || self.config.category_order != config.category_order;
                self.config = config;

                if pinned_changed || categories_changed {
                    self.rebuild_context_menus();
                }

                if (pinned_changed || categories_changed) && self.popup.is_some() {
                    self.refresh_menu_view()
                } else {
                    Task::none()
                }
            }
            Message::UpdateAvailableApplications(items) => {
                self.moving_app = None;
                self.available_applications = items;
                self.rebuild_context_menus();

                Task::none()
            }
            Message::UpdateAvailableCategories(items) => {
                self.available_categories = items;

                Task::none()
            }
            Message::SelectPreviousApp => self.select_previous_app(),
            Message::SelectNextApp => self.select_next_app(),
            Message::LaunchSelectedApplication => {
                dbg!(self.selected_item_index);
                if let Some(index) = self.selected_item_index {
                    let selected_application =
                        self.available_applications.get(index).unwrap().clone();

                    return self.launch_application(selected_application, None);
                }

                Task::none()
            }
            Message::SuperKeyPressed => self.toggle_popup(PopupType::MainMenu),
            Message::LaunchApplicationAt(index) => {
                if let Some(app) = self.available_applications.get(index).cloned() {
                    return self.launch_application(app, None);
                }

                Task::none()
            }
            Message::LaunchApplicationWithActionAt(app_index, action_index) => {
                if let Some(app) = self.available_applications.get(app_index).cloned() {
                    if let Some(action) = app.desktop_actions.get(action_index).cloned() {
                        return self.launch_application(app, Some(action));
                    }
                }

                Task::none()
            }
            Message::PinToAppTrayIndex(app_index, favorites) => {
                if let Some(app) = self.available_applications.get(app_index).cloned() {
                    let pinned_id = app.id.clone();
                    if let Some(app_list_helper) =
                        Config::new(cosmic_app_list_config::APP_ID, AppListConfig::VERSION).ok()
                    {
                        if favorites {
                            // currently favorites==true indicates it is pinned; request unpin
                            self.app_list_config
                                .remove_pinned(&pinned_id, &app_list_helper);
                        } else {
                            self.app_list_config.add_pinned(pinned_id, &app_list_helper);
                        }

                        self.rebuild_app_context_menu(app_index, &app);
                    }
                }

                Task::none()
            }
            Message::ToggleFavoriteAt(app_index) => {
                let Some(app) = self.available_applications.get(app_index).cloned() else {
                    return Task::none();
                };
                crate::model::favorites::toggle_pinned(&mut self.config.pinned_apps, &app.id);
                self.save_config();
                self.rebuild_app_context_menu(app_index, &app);
                self.refresh_menu_view()
            }
            Message::ChooseCategoryFor(app_index) => {
                self.moving_app = Some(app_index);
                Task::none()
            }
            Message::CancelChooseCategory => {
                self.moving_app = None;
                Task::none()
            }
            Message::MoveApplicationToCategory(app_index, target_index) => {
                let Some(app) = self.available_applications.get(app_index).cloned() else {
                    return Task::none();
                };
                let Some(target) = self.move_targets.get(target_index).cloned() else {
                    return Task::none();
                };
                self.config
                    .app_category_overrides
                    .insert(app.id.clone(), target.key.into_owned());
                self.save_config();
                self.rebuild_app_context_menu(app_index, &app);
                self.refresh_menu_view()
            }
            Message::ResetApplicationCategory(app_index) => {
                let Some(app) = self.available_applications.get(app_index).cloned() else {
                    return Task::none();
                };
                self.config.app_category_overrides.remove(&app.id);
                self.save_config();
                self.rebuild_app_context_menu(app_index, &app);
                self.refresh_menu_view()
            }
            Message::HideApplicationAt(app_index) => {
                let Some(app) = self.available_applications.get(app_index).cloned() else {
                    return Task::none();
                };
                if !self.config.hidden_apps.contains(&app.id) {
                    self.config.hidden_apps.push(app.id.clone());
                }
                self.save_config();
                self.refresh_menu_view()
            }
            Message::AppListConfigUpdated(app_list_config) => {
                self.app_list_config = app_list_config;

                Task::none()
            }
            Message::ContextMenuAction(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
            Message::ScrollUpdated(viewport) => {
                self.scroll_offset = viewport.absolute_offset().y;
                self.scroll_viewport_height = viewport.bounds().height;
                Task::none()
            }
        }
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            desktop_files().map(Message::FileEvent),
            listen_raw(|event, _, _| {
                return match event {
                    cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(key),
                        ..
                    }) => match key {
                        Named::ArrowUp => Some(Message::SelectPreviousApp),
                        Named::ArrowDown => Some(Message::SelectNextApp),
                        Named::Enter => Some(Message::LaunchSelectedApplication),

                        _ => None,
                    },

                    _ => None,
                };
            }),
            // Watch for application configuration changes.
            self.core
                .watch_config::<AppletConfig>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
            // DBUS subscription
            crate::dbus::dbus_service_subscription().map(|msg| msg),
            self.core
                .watch_config::<cosmic_app_list_config::AppListConfig>(
                    cosmic_app_list_config::APP_ID,
                )
                .map(|config| Message::AppListConfigUpdated(config.config)),
        ])
    }
}

impl Applet {
    /// Icon size of the app list (0 in config means "follow the theme").
    pub fn list_icon_size(&self) -> u16 {
        let spacing = cosmic::theme::active().cosmic().spacing;
        crate::model::appearance::resolve_icon_size(self.config.app_icon_size, spacing.space_l)
    }

    /// Row height of the app list; shared by rendering and scroll math.
    pub fn list_item_height(&self) -> f32 {
        let spacing = cosmic::theme::active().cosmic().spacing;
        crate::model::appearance::item_height(
            self.config.list_density,
            self.list_icon_size(),
            spacing.space_l,
            spacing.space_xl,
        )
    }

    pub fn handle_event(&mut self, event: Event) -> Task<Message> {
        match event {
            Event::Changed => {
                // Invalidate the cache
                log::debug!("App list has been updated, invalidating cache and loading new list!");
                crate::logic::apps::APPS_CACHE.lock().unwrap().cache_reset();

                Task::none()
            }
        }
    }

    fn toggle_popup(&mut self, popup_type: PopupType) -> Task<Message> {
        // reset popup state
        self.search_field.clear();
        let category = crate::logic::categories::default_category(
            crate::logic::apps::has_favorites(&self.config),
        );
        self.selected_category = Some(category.clone());
        self.available_applications =
            crate::logic::apps::get_apps_of_category(category.clone(), &self.config);
        self.selected_item_index = None;
        self.pending_confirmation = None;
        self.moving_app = None;

        let mut tasks = vec![];
        self.popup_type = popup_type;
        if self.popup_type == PopupType::MainMenu {
            let config = self.config.clone();
            tasks.push(Task::perform(
                tokio::task::spawn_blocking(move || {
                    crate::logic::apps::get_apps_of_category(category, &config)
                }),
                |res| cosmic::action::app(Message::UpdateAvailableApplications(res.unwrap())),
            ));
            let config = self.config.clone();
            tasks.push(Task::perform(
                tokio::task::spawn_blocking(move || {
                    crate::logic::apps::load_app_categories(&config)
                }),
                |res| cosmic::action::app(Message::UpdateAvailableCategories(res.unwrap())),
            ));
        }

        if let Some(p) = self.popup.take() {
            tasks.push(destroy_popup(p));
            Task::batch(tasks)
        } else {
            let new_id = Id::unique();
            self.popup.replace(new_id);
            let mut popup_settings = self.core.applet.get_popup_settings(
                self.core.main_window_id().unwrap(),
                new_id,
                None,
                None,
                None,
            );
            let (anchor, gravity) = match self.core.applet.anchor {
                PanelAnchor::Left => (Anchor::TopRight, Gravity::BottomRight),
                PanelAnchor::Right => (Anchor::TopLeft, Gravity::BottomLeft),
                PanelAnchor::Top => (Anchor::BottomLeft, Gravity::BottomRight),
                PanelAnchor::Bottom => (Anchor::TopLeft, Gravity::TopRight),
            };
            popup_settings.positioner.anchor = anchor;
            popup_settings.positioner.gravity = gravity;

            tasks.push(get_popup(popup_settings));
            Task::batch(tasks)
        }
    }

    /// Reload categories and, if one is still selected, its app list. Called
    /// after anything affecting category resolution changed: `pinned_apps`,
    /// custom categories, overrides, hidden apps/categories or their order.
    fn refresh_menu_view(&mut self) -> Task<Message> {
        let has_favorites = crate::logic::apps::has_favorites(&self.config);
        let available = crate::logic::apps::load_app_categories(&self.config);
        self.selected_category = crate::logic::categories::category_after_change(
            self.selected_category.take(),
            &available,
            has_favorites,
        );
        self.available_categories = available;
        self.selected_item_index = None;

        if let Some(category) = self.selected_category.clone() {
            let config = self.config.clone();
            Task::perform(
                tokio::task::spawn_blocking(move || {
                    crate::logic::apps::get_apps_of_category(category, &config)
                }),
                |res| cosmic::action::app(Message::UpdateAvailableApplications(res.unwrap())),
            )
        } else {
            Task::none()
        }
    }

    /// Persist the config; used by handlers that only touch it (recent
    /// applications go through `update_recent_applications` instead).
    fn save_config(&self) {
        if let Some(handler) = AppletConfig::config_handler() {
            if let Err(err) = self.config.write_entry(&handler) {
                log::error!("failed to save config: {err}");
            }
        }
    }

    /// Rebuild the cached context menu for every app currently shown; used
    /// after loading a new app list or after a config change that affects
    /// every menu (categories, hidden apps/categories, overrides).
    fn rebuild_context_menus(&mut self) {
        self.move_targets = crate::logic::categories::move_targets(&self.config);
        self.context_menus.clear();
        for (index, app) in self.available_applications.clone().into_iter().enumerate() {
            self.rebuild_app_context_menu(index, &app);
        }
    }

    /// Rebuild the cached context menu for a single app; used after an
    /// action that only affects that app (pin, favorite, move, hide).
    fn rebuild_app_context_menu(&mut self, app_index: usize, app: &Arc<ApplicationEntry>) {
        let pinned_to_panel = crate::logic::apps::is_app_in_favorites(app, &self.app_list_config);
        let favorite = self.config.pinned_apps.contains(&app.id);
        let current_target = crate::logic::categories::current_override(&app.id, &self.config)
            .and_then(|key| self.move_targets.iter().position(|c| c.key.as_ref() == key));
        let state = crate::applet_menu::AppMenuState {
            pinned_to_panel,
            favorite,
            move_targets: &self.move_targets,
            current_target,
        };
        let trees = crate::applet_menu::AppletMenu::build_app_context_menu(app, app_index, state);
        self.context_menus.insert(app.id.clone(), trees);
    }

    fn close_popup(&mut self, id: Id) -> Task<Message> {
        if self.popup.as_ref() == Some(&id) {
            self.popup = None;
            self.pending_confirmation = None;
            self.moving_app = None;
        }

        Task::none()
    }

    fn clear_search(&mut self) -> Task<Message> {
        self.selected_category = Some(ApplicationCategory::ALL);
        self.search_field = "".to_string();

        let config = self.config.clone();
        Task::perform(
            tokio::task::spawn_blocking(move || {
                crate::logic::apps::get_apps_of_category(ApplicationCategory::ALL, &config)
            }),
            |res| cosmic::action::app(Message::UpdateAvailableApplications(res.unwrap())),
        )
    }

    fn update_search_field(&mut self, input: String) -> Task<Message> {
        self.selected_category = None;
        self.selected_item_index = None;

        self.search_field = input.clone();
        if self.search_field.is_empty() {
            return self.clear_search();
        }

        let config = self.config.clone();
        Task::batch([
            // reset scroll position
            cosmic::iced::widget::operation::snap_to(
                self.scrollable_id.clone(),
                RelativeOffset { x: 0., y: 0. },
            ),
            Task::perform(
                tokio::task::spawn_blocking(move || {
                    crate::logic::apps::load_filtered_apps(input, &config)
                }),
                |res| cosmic::Action::App(Message::UpdateAvailableApplications(res.unwrap())),
            ),
        ])
    }

    fn perform_power_action(&mut self, action: PowerAction) -> Task<Message> {
        if action.needs_confirmation() {
            self.pending_confirmation = Some((action, std::time::Instant::now()));
            return Task::none();
        }

        let is_flatpak = std::env::var("FLATPAK_ID").is_ok();

        if action == PowerAction::Lock || action == PowerAction::Suspend {
            return action.perform();
        }

        let app_exec = match action {
            PowerAction::Logout => "cosmic-osd log-out",
            PowerAction::Reboot => "cosmic-osd restart",
            PowerAction::Shutdown => "cosmic-osd shutdown",
            _ => "",
        };
        let (main_exec, args) = if is_flatpak {
            (
                "flatpak-spawn",
                vec!["--host", "/bin/sh", "-l", "-c", app_exec],
            )
        } else {
            let mut parts = app_exec.split_whitespace();
            let exec = parts.next().unwrap_or("");
            let args: Vec<&str> = parts.collect();

            (exec, args)
        };

        // non sandboxed env
        if let Err(_) = process::Command::new(main_exec).args(args).spawn() {
            return action.perform();
        }

        if let Some(p) = self.popup.take() {
            return destroy_popup(p);
        }

        Task::none()
    }

    fn launch_application(
        &mut self,
        app: Arc<ApplicationEntry>,
        action: Option<DesktopAction>,
    ) -> Task<Message> {
        let mut app_exec = if action.is_some() {
            action
                .unwrap()
                .exec
                .clone()
                .split_whitespace()
                .filter(|arg| !arg.starts_with('%'))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            app.exec
                .clone()
                .unwrap()
                .split_whitespace()
                .filter(|arg| !arg.starts_with('%'))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let env_vars: Vec<(String, String)> = std::env::vars().collect();
        let app_id = Some(app.id.clone());
        let mut is_terminal = app.is_terminal;

        let is_flatpak = std::env::var("FLATPAK_ID").is_ok();

        if is_flatpak {
            if is_terminal {
                // For flatpaks handle terminal applications manually
                // not through libcosmic implementation
                let term = cosmic_settings_config::shortcuts::context()
                    .ok()
                    .and_then(|config| {
                        cosmic_settings_config::shortcuts::system_actions(&config)
                            .get(&cosmic_settings_config::shortcuts::action::System::Terminal)
                            .cloned()
                    })
                    .unwrap_or_else(|| String::from("cosmic-term"));

                app_exec = format!("{term} -- {}", app_exec);
                is_terminal = false;
            }

            app_exec = format!("flatpak-spawn --host /bin/sh -l -c '{}'", app_exec);
        }

        tokio::spawn(async move {
            cosmic::desktop::spawn_desktop_exec(app_exec, env_vars, app_id.as_deref(), is_terminal)
                .await;
        });

        self.update_recent_applications(app);

        if let Some(p) = self.popup.take() {
            return destroy_popup(p);
        }
        Task::none()
    }

    fn update_recent_applications(&mut self, app: Arc<ApplicationEntry>) {
        let current_recent_application = self
            .config
            .recent_applications
            .iter_mut()
            .find(|x| x.app_id == app.id);
        if let Some(recent_app) = current_recent_application {
            if recent_app.launch_count < u32::MAX {
                recent_app.launch_count += 1;
            }
        } else {
            self.config.recent_applications.push(RecentApplication {
                app_id: app.id.clone(),
                launch_count: 1,
            });
        }

        self.config
            .write_entry(AppletConfig::config_handler().as_ref().unwrap())
            .expect("Failed to write recent applications config");
    }

    fn select_category(&mut self, category: ApplicationCategory) -> Task<Message> {
        self.search_field.clear();
        self.selected_category = Some(category.clone());
        self.selected_item_index = None;

        let config = self.config.clone();
        Task::batch([
            // reset scroll position
            cosmic::iced::widget::operation::snap_to(
                self.scrollable_id.clone(),
                RelativeOffset { x: 0., y: 0. },
            ),
            Task::perform(
                tokio::task::spawn_blocking(move || {
                    crate::logic::apps::get_apps_of_category(category, &config)
                }),
                |res| cosmic::Action::App(Message::UpdateAvailableApplications(res.unwrap())),
            ),
        ])
    }

    fn launch_tool(&mut self, tool: SystemTool) -> Task<Message> {
        tool.perform();
        if let Some(p) = self.popup.take() {
            return destroy_popup(p);
        }
        Task::none()
    }

    fn handle_zbus_result(&self, result: Result<(), zbus::Error>) -> Task<Message> {
        if let Err(e) = result {
            log::error!("cosmic-ext-classic-menu-plus ERROR: '{}'", e);
        }

        Task::none()
    }

    fn view_main_menu(&self) -> Element<'_, Message> {
        // TODO: Implement grid view
        AppletMenu::view_main_menu_list(&self)
    }

    fn view_context_menu(&self) -> Element<'_, Message> {
        let context_menu = column![
            cosmic::applet::menu_button(
                row![cosmic::widget::text::body(fl!("settings")),].align_y(Alignment::Center)
            )
            .class(cosmic::theme::Button::AppletMenu)
            .on_press(Message::LaunchTool(SystemTool::APPLET_SETTINGS)),
            cosmic::applet::padded_control(cosmic::widget::divider::horizontal::default()),
            cosmic::applet::menu_button(
                row![cosmic::widget::text::body(fl!("settings-label")),].align_y(Alignment::Center)
            )
            .class(cosmic::theme::Button::AppletMenu)
            .on_press(Message::LaunchTool(SystemTool::SYSTEM_SETTINGS)),
            cosmic::applet::menu_button(
                row![cosmic::widget::text::body(fl!("system-monitor-label")),]
                    .align_y(Alignment::Center)
            )
            .class(cosmic::theme::Button::AppletMenu)
            .on_press(Message::LaunchTool(SystemTool::SYSTEM_MONITOR)),
            cosmic::applet::menu_button(
                row![cosmic::widget::text::body(fl!("disks-label")),].align_y(Alignment::Center)
            )
            .class(cosmic::theme::Button::AppletMenu)
            .on_press(Message::LaunchTool(SystemTool::DISK_MANAGEMENT)),
        ]
        .padding([8, 0]);

        self.core.applet.popup_container(context_menu).into()
    }

    fn select_previous_app(&mut self) -> cosmic::Task<cosmic::Action<Message>> {
        if self.selected_item_index.is_none() {
            return Task::none();
        }

        if let Some(index) = self.selected_item_index {
            if index > 0 {
                self.selected_item_index = Some(index - 1);
            }
        }

        if let Some(index) = self.selected_item_index {
            let item_height = self.list_item_height();
            let viewport_height = self.scroll_viewport_height.max(item_height);
            let visible_top = self.scroll_offset;
            let visible_bottom = visible_top + viewport_height;

            let selected_top = index as f32 * item_height;
            let selected_bottom = selected_top + item_height;

            if selected_top >= visible_top && selected_bottom <= visible_bottom {
                return Task::none();
            }

            let target_offset = if selected_top < visible_top {
                selected_top
            } else {
                selected_bottom - viewport_height
            };

            return Task::batch([cosmic::iced::widget::operation::scroll_to(
                self.scrollable_id.clone(),
                AbsoluteOffset {
                    x: 0.,
                    y: target_offset,
                },
            )]);
        }

        Task::none()
    }

    fn select_next_app(&mut self) -> cosmic::Task<cosmic::Action<Message>> {
        if self.selected_item_index.is_none() && !self.available_applications.is_empty() {
            self.selected_item_index = Some(0);
        } else if let Some(index) = self.selected_item_index {
            if index < self.available_applications.len() - 1 {
                self.selected_item_index = Some(index + 1);
            }
        }

        if let Some(index) = self.selected_item_index {
            let item_height = self.list_item_height();
            let viewport_height = self.scroll_viewport_height.max(item_height);
            let visible_top = self.scroll_offset;
            let visible_bottom = visible_top + viewport_height;

            let selected_top = index as f32 * item_height;
            let selected_bottom = selected_top + item_height;

            if selected_top >= visible_top && selected_bottom <= visible_bottom {
                return Task::none();
            }

            let target_offset = if selected_top < visible_top {
                selected_top
            } else {
                selected_bottom - viewport_height
            };

            dbg!(target_offset);

            return Task::batch([cosmic::iced::widget::operation::scroll_to(
                self.scrollable_id.clone(),
                AbsoluteOffset {
                    x: 0.,
                    y: target_offset + 16.0,
                },
            )]);
        }

        Task::none()
    }
}
