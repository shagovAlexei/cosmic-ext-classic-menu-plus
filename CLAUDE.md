# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Classic-style application launcher applet for the COSMIC desktop, written in Rust on top of `libcosmic` (iced fork). This repo is a fork of `championpeak87/cosmic-ext-classic-menu` (remote `upstream`), renamed to `cosmic-ext-classic-menu-plus` so it can be installed alongside the original; our remote is `origin` (`shagovAlexei/cosmic-ext-classic-menu-plus`). The rename touched the crate/binary names, `APP_ID`, config path, D-Bus name and packaging files, so merging `upstream` will conflict there: resolve by keeping the `-plus` names. Keep changes merge-friendly with upstream: additive config fields, no gratuitous reformatting of upstream code. Fork roadmap and TODO list: `docs/PLAN.md`.

## Commands

```bash
just build-debug           # cargo build for both crates (applet + settings)
just build-release         # release build
sudo just install          # install binaries, .desktop, metainfo, icons into /usr
just check                 # cargo fmt + clippy --all-features -W clippy::pedantic
just run-logs -p cosmic-ext-classic-menu-plus-applet   # run with RUST_LOG/backtrace
cargo build -p cosmic-ext-classic-menu-plus-applet     # single crate
cargo run -p cosmic-ext-classic-menu-plus-settings     # run the settings app standalone
cargo test -p cosmic-ext-classic-menu-plus-applet <name>  # single test (no tests exist yet)
```

The applet binary only works when launched by `cosmic-panel`. To try changes: `just build-release && sudo just install`, then restart the panel (`killall cosmic-panel`, it respawns) or remove/re-add the applet. The settings app runs standalone.

Logging goes through `simple_logger::init_with_env()`, so use `RUST_LOG=debug`.

## Architecture

Cargo workspace with two crates:

- **`applet/`** (`cosmic-ext-classic-menu-plus-applet`): the panel applet. It is also a **library** (`lib.rs` re-exports all modules), so the settings crate can reuse `config::AppletConfig` and `applet::APP_ID`.
- **`settings/`** (`cosmic-ext-classic-menu-plus-settings`): a standalone windowed COSMIC app (`settings/src/app.rs`) that edits the same config. It is launched from the applet's right-click menu via `SystemTool::APPLET_SETTINGS`.

### Config is the contract between the two crates

`applet/src/config.rs` holds `AppletConfig`, a `#[derive(CosmicConfigEntry)]` struct with `#[version = 1]` and id `io.github.shagovAlexei.cosmic-ext-classic-menu-plus`. It is stored by cosmic-config under `~/.config/cosmic/io.github.shagovAlexei.cosmic-ext-classic-menu-plus/v1/`, one file per field. The settings app writes fields with `config.write_entry(&AppletConfig::config_handler())`. The applet watches with `core.watch_config::<AppletConfig>` → `Message::UpdateConfig`, so changes apply live. The applet also writes to config itself (`recent_applications` launch counts). New fields need a `Default` value so existing installs keep loading. Missing keys fall back via `get_entry(...).unwrap_or_else(|(_, c)| c)`.

The applet also reads and writes **`cosmic_app_list_config::AppListConfig`** (the dock/app-tray config) for the "Pin to panel" context-menu action.

### Applet (Elm-style `cosmic::Application`)

- `applet.rs`: the `Applet` state struct, the big `Message` enum, `update()`, `subscription()`, and popup management. Everything routes through here. It is large (~35 KB). Prefer putting new logic into `logic/`/`model/` helpers and keeping `update()` arms thin.
- `applet_button.rs`: the panel button (icon/label/auto depending on `AppletButtonStyle` and panel size).
- `applet_menu.rs`: builds the main popup view. It contains the user widget, search field, app list, categories pane, and the power-button row (`create_power_menu`), plus the `ContextMenuAction` enum for right-click menus on app entries. Popup size constants (`POPUP_*`) and the hardcoded width of 600 live here.
- `widgets/virtualized_app_list.rs`: renders only the visible rows of the app list. Item height is assumed to be `space_xl` from the theme, so scroll math breaks if row height changes without updating this. Scroll position arrives via `Message::ScrollUpdated`.
- There are two popup kinds (`model/popup_type.rs`). `MainMenu` is opened by left click or Super key. `ContextMenu` is opened by right click on the panel button (links to settings, system settings, monitor, disks). Per-app right-click menus are separate: `cosmic::widget::menu` trees cached in `Applet::context_menus` keyed by app id, shown through `Message::ContextMenuAction(surface Action)`.

### App & category data flow

- `logic/apps.rs::load_apps()` loads desktop entries via `cosmic::desktop::load_applications` and memoizes them in the `#[cached]` `APPS_CACHE`. A `notify` watcher on XDG app dirs (`desktop_files()` subscription) resets the cache on change.
- Search is `load_filtered_apps()`: skim fuzzy matching over name/generic name/comment, with diacritics stripped.
- Categories are **compile-time constants** in `model/application_category.rs` (`&'static` name, embedded SVG bytes, freedesktop `mime_name`). `load_app_categories()` shows the permanent ones (All, Recently used) plus those used by at least one app. `get_apps_of_category()` filters by `mime_name`. The categories pane inserts a divider after index 2 (after the permanent ones). Upstream has an unmerged draft (`upstream/feat/custom-categories`) that turns these fields into `Cow` to allow runtime categories.
- Recently used apps come from `AppletConfig.recent_applications`, sorted by launch count, top 15.

### Power actions & D-Bus

- `model/power_action.rs` (`PowerAction` enum) and `power_options.rs` (async logind calls via `logind-zbus`: reboot, power_off, suspend, session lock; logout via `cosmic_session.rs` or GNOME `session_manager.rs`).
- `Applet::perform_power_action` calls Lock/Suspend directly. Logout/Reboot/Shutdown first try `cosmic-osd <action>` (confirmation dialog), wrapped in `flatpak-spawn --host` under Flatpak, and fall back to the logind call. Results come back as `Message::Zbus`.
- `dbus/` exposes the session bus service `io.github.shagovAlexei.CosmicExtClassicMenuPlus` with method `TogglePopupSignal` → `Message::SuperKeyPressed`. This is how a keyboard shortcut opens the menu.

### Icons & i18n

- UI icons are bundled SVGs in `res/icons/bundled/`, embedded with `include_bytes!` and rendered `.symbolic(true)`. Applet-button icons are installed to `/usr/share/cosmic/<APPID>/applet-buttons/`.
- Fluent localization: `applet/i18n/<lang>/cosmic_ext_classic_menu_plus_applet.ftl` and `settings/i18n/<lang>/cosmic_ext_classic_menu_plus_settings.ftl`, used via the `fl!("key")` macro (compile-time checked against `en`). New strings must be added to `en` at least. Add `ru` too for this fork.

### Packaging

`justfile` + `res/packaging.just` (install paths), `flatpak/` manifest (with `cargo-sources.json`, which must be regenerated when dependencies change), `rpm/` spec, `package.nix`. APP_ID `io.github.shagovAlexei.cosmic-ext-classic-menu-plus` is used across all of them and in the config path, so don't change it.

## Known issues (from README)

- The per-app context menu is misaligned when the list is scrolled.
- The popup is not focused when opened, so search and arrow keys may not work until it's clicked.
