{ lib
, fetchFromGitHub
, rustPlatform
, pkg-config
, just
, stdenv
, glib
, gtk3
, libcosmicAppHook
}:

rustPlatform.buildRustPackage rec {
  pname = "cosmic-ext-classic-menu-plus";
  version = "0.0.11"; # Update this based on the latest release or tag

  src = fetchFromGitHub {
    owner = "championpeak87";
    repo = "cosmic-classic-menu";
    rev = "master"; # Or a specific tag like "v0.1.0"
    hash = "sha256-xiM9O37lZEv8Jfc3cBp31zKRXmUa+Xy9oipjAeFdPjE="; # See note below
  };

  # This is required for Rust projects that don't have a vendor folder
  # You can use 'lib.fakeHash' initially to get the correct hash from the error message
  cargoHash = "sha256-xflF6v6pDtEHydC5YM+mHBjf+GFgDXgIzo4ntPQac7w=";

  nativeBuildInputs = [
    pkg-config
    just
    libcosmicAppHook
  ];

  buildInputs = [
    glib
    gtk3
  ];

  # COSMIC applets usually use 'just' for specific install tasks
  # but buildRustPackage handles the cargo build automatically.
  # If the applet needs specific RON files moved to /share/cosmic, 
  # you might need a postInstall hook.  
  dontCargoBuild = true;
  buildPhase = ''
    runHook preBuild
    just build-release
    runHook postBuild
  '';

  # Override the default cargo build to use 'just'
  justFlags = [
    "--set"
    "prefix"
    (placeholder "out")
    "--set"
    "bin-src"
    "target/release/cosmic-ext-classic-menu-plus-applet"
    "--set"
    "settings-bin-src"
    "target/release/cosmic-ext-classic-menu-plus-settings"
  ];
  
  meta = with lib; {
    description = "A classic-style application menu for the COSMIC Desktop";
    homepage = "https://github.com/shagovAlexei/cosmic-ext-classic-menu-plus";
    license = licenses.gpl3Only;
    maintainers = [ ];
    platforms = platforms.linux;
  };
}
