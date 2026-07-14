<!--
SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>
SPDX-License-Identifier: CC0-1.0
-->
# komplete

## Layout

```
src/komplete-run       # launcher (bash)
src/komplete-install   # GUI installer
trees/<name>/usr/...   # one integration per directory
tools/new-tree.sh      # scaffolding for new trees
```

## Build & install

```bash
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr
cmake --build build
sudo cmake --install build
```

All trees are installed by default; select with `-DKOMPLETE_TREES="kjar;wine"`.
Result: shims in `/usr/bin`, framework in `/usr/lib/komplete`, configs in
`/usr/share/komplete`, desktop files and binfmt rules in their usual places.

Runtime dependencies: `flatpak`, `pyside6`, `libnotify`,
`desktop-file-utils`.

