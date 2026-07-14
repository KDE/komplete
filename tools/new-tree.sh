#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>
# SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL

#
# Scaffold a per-project komplete integration tree under trees/.
#
# Usage:
#   new-tree.sh --name kjar \
#       --app-id org.kde.kjar \
#       --display-name "Java Support" \
#       --icon application-x-java-archive \
#       --remote-name kjar-nightly \
#       --remote-url https://cdn.kde.org/flatpak/kjar-nightly/kjar-nightly.flatpakrepo \
#       --post-install "flatpak run org.kde.kjar --generate-wrappers" \
#       --cmd java --cmd javac \
#       --mime application/java-archive --mime-exec "java -jar %f" \
#       --binfmt jar --binfmt-cmd "java -jar" \
#       [--output-dir trees]
#


set -euo pipefail

NAME= APP_ID= DISPLAY_NAME= ICON=application-x-executable
REMOTE_NAME= REMOTE_URL= POST_INSTALL=
MIME= MIME_EXEC= BINFMT_EXT= BINFMT_CMD=
OUTPUT_DIR=trees
CMDS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --name)          NAME="$2"; shift 2 ;;
        --app-id)        APP_ID="$2"; shift 2 ;;
        --display-name)  DISPLAY_NAME="$2"; shift 2 ;;
        --icon)          ICON="$2"; shift 2 ;;
        --remote-name)   REMOTE_NAME="$2"; shift 2 ;;
        --remote-url)    REMOTE_URL="$2"; shift 2 ;;
        --post-install)  POST_INSTALL="$2"; shift 2 ;;
        --cmd)           CMDS+=("$2"); shift 2 ;;
        --mime)          MIME="$2"; shift 2 ;;
        --mime-exec)     MIME_EXEC="$2"; shift 2 ;;
        --binfmt)        BINFMT_EXT="$2"; shift 2 ;;
        --binfmt-cmd)    BINFMT_CMD="$2"; shift 2 ;;
        --output-dir)    OUTPUT_DIR="$2"; shift 2 ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

[[ -n "$NAME" && -n "$APP_ID" && -n "$REMOTE_NAME" && -n "$REMOTE_URL" ]] || {
    echo "Missing required option (--name/--app-id/--remote-name/--remote-url)" >&2
    exit 1
}
DISPLAY_NAME="${DISPLAY_NAME:-$NAME}"

ROOT="$OUTPUT_DIR/$NAME"
mkdir -p "$ROOT/usr/bin" "$ROOT/usr/share/komplete"

# komplete config
{
    # REUSE-IgnoreStart
    echo "# SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL"
    echo "# SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>"
    # REUSE-IgnoreEnd
    echo "[App]"
    echo "Id=$APP_ID"
    echo "Name=$DISPLAY_NAME"
    echo "Icon=$ICON"
    echo
    echo "[Remote]"
    echo "Name=$REMOTE_NAME"
    echo "Url=$REMOTE_URL"
    if [[ -n "$POST_INSTALL" ]]; then
        echo
        echo "[Install]"
        echo "PostInstall=$POST_INSTALL"
    fi
} > "$ROOT/usr/share/komplete/$NAME.conf"

# command shims
for cmd in "${CMDS[@]}"; do
    # REUSE-IgnoreStart
    cat > "$ROOT/usr/bin/$cmd" << EOF
#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL

# SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>
exec /usr/lib/komplete/komplete-run $NAME $cmd "\$@"
EOF
    # REUSE-IgnoreEnd
    chmod 755 "$ROOT/usr/bin/$cmd"
done

# optional MIME integration
if [[ -n "$MIME" ]]; then
    [[ -n "$MIME_EXEC" ]] || { echo "--mime requires --mime-exec" >&2; exit 1; }
    mkdir -p "$ROOT/usr/share/applications"
    cat > "$ROOT/usr/share/applications/$APP_ID.desktop" << EOF
[Desktop Entry]
Name=$DISPLAY_NAME
Comment=Open this file type with $DISPLAY_NAME
Exec=$MIME_EXEC
Icon=$ICON
Terminal=false
Type=Application
NoDisplay=true
MimeType=$MIME;
Categories=Development;
EOF
fi

# optional binfmt rule
if [[ -n "$BINFMT_EXT" ]]; then
    [[ -n "$BINFMT_CMD" ]] || { echo "--binfmt requires --binfmt-cmd" >&2; exit 1; }
    mkdir -p "$ROOT/usr/lib/binfmt.d" "$ROOT/usr/lib/komplete"
    # REUSE-IgnoreStart
    cat > "$ROOT/usr/lib/komplete/$NAME-binfmt" << EOF
#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL

# SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>
exec $BINFMT_CMD "\$@"
EOF
    # REUSE-IgnoreEnd
    chmod 755 "$ROOT/usr/lib/komplete/$NAME-binfmt"
    printf ':OnDemand-%s:E::%s::/usr/lib/komplete/%s-binfmt:F\n' \
        "$NAME" "$BINFMT_EXT" "$NAME" > "$ROOT/usr/lib/binfmt.d/$NAME.conf"
fi

echo "Created tree: $ROOT"
echo "It will be installed by CMake on the next build."
