#! /usr/bin/env bash
# SPDX-FileCopyrightText: None
# SPDX-License-Identifier: CC0-1.0

$XGETTEXT --from-code=UTF-8 --language=Rust \
    --keyword=gettext:1 --keyword=tr1:1 \
    $(find src/komplete-install/src -name '*.rs') \
    --output=$podir/komplete.pot

$XGETTEXT --from-code=UTF-8 --language=JavaScript --join-existing --kde \
    --keyword=i18n:1 --keyword=i18nc:1c,2 \
    --keyword=i18np:1,2 --keyword=i18ncp:1c,2,3 \
    $(find src/komplete-install/src/qml -name '*.qml') \
    --output=$podir/komplete.pot
