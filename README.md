<!--
SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>
SPDX-License-Identifier: CC0-1.0
-->
# komplete

komplete adds on-demand installation for tools, MIME
handlers, and binfmt interpreters that are shipped as Flatpaks rather
than baked into the base image. You define an integration ("tree") for a
tool once; komplete then installs a shim under `/usr` for it. Run that
command (or open a file it's registered to handle) before the Flatpak is
installed, and instead of "command not found" you get a Kirigami prompt —
"$THING is incomplete; download it from the internet?" — that adds the
Flatpak remote and installs it for you, then runs the command. After
that first install, the shim just calls the Flatpak directly.

`komplete-run` is the shim's target: it checks whether the tree's Flatpak
is installed and, if not, hands off to the `komplete-install` GUI before
running the real command. The `kjar` tree in this repo is the example —
it makes `java`/`javac` and `.jar` files work out of the box by
on-demand-installing a JDK Flatpak the first time they're used.

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
