<!--
SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>
SPDX-License-Identifier: CC0-1.0
-->
# komplete

## Layout

```
src/komplete-run              # launcher (bash)
src/komplete-install/         # GUI installer (Rust + Kirigami)
    src/main.rs               #   entry point, CLI fast path, QML engine
    src/config.rs             #   config parsing, flatpak helpers, notifications
    src/installer.rs          #   Installer QObject exposed to QML
    src/qml/Main.qml          #   Kirigami UI
trees/<name>/usr/...          # one integration per directory
tools/new-tree.sh             # scaffolding for new trees
```

## Build & install

```bash
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release
cmake --build build
sudo cmake --install build
```

