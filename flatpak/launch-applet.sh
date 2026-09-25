#!/bin/sh

# Set variables
export PATH=$PATH:$(flatpak-spawn --host /bin/sh -l -c 'echo $PATH')
export XDG_DATA_DIRS=$XDG_DATA_DIRS:$(flatpak-spawn --host /bin/sh -l -c 'echo $XDG_DATA_DIRS')

# Workaround to expand $HOME variable defined in flatpak manifest file
export XDG_DATA_DIRS=$(envsubst <<< $XDG_DATA_DIRS | tr ':' '\n' | sort | uniq | grep -v '\$' | tr '\n' ':')

exec cosmic-ext-classic-menu-plus-applet "$@"