# cosmic-ext-classic-menu-plus

Classic Menu is a customizable application launcher for the COSMIC™ desktop environment. It provides a classic style menu for launching applications, accessing system tools, and managing power options.

![Classic Menu Screenshot](screenshots/cosmic-ext-classic-menu-applet.png)

## Features

- Classic-style application menu
- Search functionality with fuzzy matching and typo tolerance
- Favorites: pin apps from their right-click menu, shown first and open by default once you have any
- Categories: standard freedesktop categories plus your own (custom name and icon), move apps between categories or hide them from the menu, reorder and hide categories
- Recently used applications
- Power options: logout, suspend, hibernate, lock, reboot, shutdown, with a confirmation step for hibernate; pick which buttons to show and in what order
- Appearance settings: popup width/height, application icon size, compact/normal list density
- System tools (settings, system monitor, disk management)
- A standalone settings app (`cosmic-ext-classic-menu-plus-settings`) for everything above — see `screenshots/cosmic-ext-classic-menu-settings.png`

## Known issues

- Popup is not in focus when opened, search field and arrow key navigation may not work properly unless the popup is in focus by clicking on it

## Installation 

Clone the repository:

```bash
git clone https://github.com/shagovAlexei/cosmic-ext-classic-menu-plus cosmic-ext-classic-menu-plus
cd cosmic-ext-classic-menu-plus
```

Build and install the project:

```bash
just build-release
sudo just install
```

## Flatpak

This project includes a Flatpak manifest at `flatpak/io.github.shagovAlexei.cosmic-ext-classic-menu-plus.json` and a `cargo-sources.json` helper. Use Flatpak and flatpak-builder to build, install, and run the app locally.

Prerequisites

- flatpak
- flatpak-builder
- Access to the runtimes referenced by the manifest (for example Flathub and/or System76's Flatpak remotes). If a runtime isn't available you may need to enable Flathub or add the appropriate remote.

Build and install (user)

From the repository root:

```bash
# build and install into the current user
flatpak-builder --install --user --force-clean build-dir flatpak/io.github.shagovAlexei.cosmic-ext-classic-menu-plus.json
```

Run without installing

```bash
# build the project and run the app inside the build sandbox
flatpak-builder --run build-dir flatpak/io.github.shagovAlexei.cosmic-ext-classic-menu-plus.json io.github.shagovAlexei.cosmic-ext-classic-menu-plus
```

Create a reusable bundle and install

```bash
# create a local flatpak repo and build the app into it
flatpak-builder --repo=repo build-dir flatpak/io.github.shagovAlexei.cosmic-ext-classic-menu-plus.json --force-clean

# create a single-file bundle
flatpak build-bundle repo io.github.shagovAlexei.cosmic-ext-classic-menu-plus.flatpak io.github.shagovAlexei.cosmic-ext-classic-menu-plus

# install the bundle for the current user
flatpak install --user io.github.shagovAlexei.cosmic-ext-classic-menu-plus.flatpak
```

Notes & troubleshooting

- If the build fails due to a missing runtime, add Flathub (or the remote that provides the runtime):

```bash
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
```

- The manifest uses `com.system76.Cosmic.BaseApp` as its base. Ensure that remote provides that runtime or change the manifest to a base available to you.


## Fedora / COPR

> **Note:** this section describes the **original** upstream project (`championpeak87/cosmic-ext-classic-menu`), not the Plus fork. There is no COPR package for the fork.

A Fedora COPR package is available for the original project in the `championpeak87/cosmic-ext-classic-menu` COPR repository. Use the `dnf copr` helper to enable the COPR and install the packaged RPM.

```bash
# enable the COPR repository (requires sudo)
sudo dnf copr enable championpeak87/cosmic-ext-classic-menu -y

# install the packaged application
sudo dnf install cosmic-ext-classic-menu
```

To update the package later:

```bash
sudo dnf upgrade --refresh cosmic-ext-classic-menu
```

To remove the package and disable the COPR repo:

```bash
sudo dnf remove cosmic-ext-classic-menu
sudo dnf copr disable championpeak87/cosmic-ext-classic-menu
```

More details and package versions are available on the COPR page:

https://copr.fedorainfracloud.org/coprs/championpeak87/cosmic-ext-classic-menu/

## Contributing

A [justfile](./justfile) is included with common recipes used by other COSMIC projects:

- `just build-debug` compiles with debug profile
- `just run` builds and runs the application
- `just check` runs clippy on the project to check for linter warnings
- `just check-json` can be used by IDEs that support LSP

## License

Code is distributed with the [GPL-3.0-only license][./LICENSE]

---

A fork of [championpeak87/cosmic-ext-classic-menu](https://github.com/championpeak87/cosmic-ext-classic-menu) with extra features: favorites, custom categories, hibernate, configurable power buttons, and appearance settings (popup size, icon size, list density). It uses its own app id, config and D-Bus name, so it can be installed next to the original.

