// SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL
// SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami
import org.kde.komplete

Kirigami.ApplicationWindow {
    id: root

    title: installer.displayName
    minimumWidth: Kirigami.Units.gridUnit * 24
    minimumHeight: Kirigami.Units.gridUnit * 12
    width: minimumWidth
    height: minimumHeight

    Installer {
        id: installer
        onFinished: success => {
            if (success) {
                installer.completeSuccess()
            } else {
                errorDialog.open()
            }
        }
    }

    // "Download and install it now?"
    Kirigami.PromptDialog {
        id: askDialog
        title: i18n("%1 Required", installer.displayName)
        subtitle: i18n("This file requires %1, but it is not installed.\n\nDownload and install it now?", installer.displayName)
        standardButtons: Kirigami.Dialog.Yes | Kirigami.Dialog.No
        showCloseButton: false
        closePolicy: Controls.Popup.NoAutoClose
        onAccepted: {
            askDialog.close()
            installer.start()
        }
        onRejected: installer.decline()
    }

    Kirigami.PromptDialog {
        id: errorDialog
        title: i18n("Installation Failed")
        subtitle: i18n("The installation of %1 was cancelled or failed.", installer.displayName)
        standardButtons: Kirigami.Dialog.Ok
        showCloseButton: false
        closePolicy: Controls.Popup.NoAutoClose
        onAccepted: installer.quitError()
        onRejected: installer.quitError()
    }

    pageStack.initialPage: Kirigami.Page {
        title: installer.displayName

        ColumnLayout {
            anchors {
                left: parent.left
                right: parent.right
                verticalCenter: parent.verticalCenter
            }
            spacing: Kirigami.Units.largeSpacing

            RowLayout {
                spacing: Kirigami.Units.largeSpacing
                Layout.fillWidth: true

                Kirigami.Icon {
                    source: installer.icon
                    Layout.preferredWidth: Kirigami.Units.iconSizes.large
                    Layout.preferredHeight: Kirigami.Units.iconSizes.large
                }

                Controls.Label {
                    text: i18n("Installing %1...", installer.displayName)
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }
            }

            Controls.ProgressBar {
                from: 0
                to: 100
                value: installer.progress
                Layout.fillWidth: true
            }

            Controls.Button {
                text: i18n("Cancel")
                icon.name: "dialog-cancel"
                Layout.alignment: Qt.AlignRight
                onClicked: installer.cancel()
            }
        }
    }

    Component.onCompleted: askDialog.open()
}
