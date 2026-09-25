// SPDX-License-Identifier: {{ license }}

mod app;
mod i18n;

fn main() -> cosmic::iced::Result {
    // Initialize logging
    simple_logger::init_with_env().unwrap();
    log::info!("Starting Classic Menu Settings");

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);
    // The applet crate's `fl!()` (used by e.g. ApplicationCategory::get_display_name)
    // has its own localizer and needs to be initialized separately.
    cosmic_ext_classic_menu_plus_applet::i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default()
        .size_limits(cosmic::iced::Limits::NONE.height(600.0).width(900.0))
        .resizable(Some(0.0));

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, ())
}
