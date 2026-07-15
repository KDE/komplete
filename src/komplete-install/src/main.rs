// SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL
// SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>

//! Usage: komplete-install <config-name>

mod config;
mod installer;

use std::env;

use cxx_kde_frameworks::ki18n::{self, KLocalizedString};
use cxx_qt::casting::Upcast;
use cxx_qt_lib::{QByteArray, QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use cxx_qt_lib_extras::QApplication;
use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};

fn main() {
    setlocale(LocaleCategory::LcAll, "");
    // Like the rest of komplete, the runtime is expected to live under /usr.
    let _ = bindtextdomain("komplete", "/usr/share/locale");
    let _ = textdomain("komplete");

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: komplete-install <config-name>");
        std::process::exit(2);
    }

    let cfg = match config::Config::load(&args[1]) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("komplete-install: {err:#}");
            std::process::exit(1);
        }
    };

    // Fast path: already installed, no GUI needed.
    match config::flatpak_installed(&cfg) {
        Ok(true) => {
            config::notify(
                &cfg,
                &cfg.display_name,
                &config::tr1("%1 is already installed and ready to use.", &cfg.display_name),
            );
            config::run_post_install(&cfg);
            return;
        }
        Ok(false) => {}
        Err(err) => {
            eprintln!("komplete-install: {err:#}");
            std::process::exit(1);
        }
    }

    let mut app = QApplication::new();
    KLocalizedString::set_application_domain(&QByteArray::from("komplete"));

    // To associate the executable to the installed desktop file
    QGuiApplication::set_desktop_file_name(&QString::from("org.kde.komplete_install"));

    // To ensure the style is set correctly
    if env::var("QT_QUICK_CONTROLS_STYLE").is_err() {
        QQuickStyle::set_style(&QString::from("org.kde.desktop"));
    }

    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        ki18n::setup_localized_context(engine.as_mut().upcast_pin());
        engine.load(&QUrl::from("qrc:/qt/qml/org/kde/komplete/src/qml/Main.qml"));
    }

    let mut exit_code = 0;
    if let Some(app) = app.as_mut() {
        exit_code = app.exec();
    }
    std::process::exit(exit_code);
}
