// SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL
// SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>

//! The `Installer` QObject exposed to QML.
//!
//! Runs `flatpak install` on a worker thread, mirrors its "NN%" output into
//! the `progress` property, and emits `finished(success)` when done.

use std::env;
use std::io::{BufReader, Read};
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use crate::config::{self, Config};

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, display_name)]
        #[qproperty(QString, icon)]
        #[qproperty(i32, progress)]
        type Installer = super::InstallerStruct;

        /// Add the remote and start `flatpak install` on a worker thread.
        #[qinvokable]
        fn start(self: Pin<&mut Installer>);

        /// Kill the running installation (progress page's Cancel button).
        #[qinvokable]
        fn cancel(self: Pin<&mut Installer>);

        /// User answered "No" to the install prompt: exit(1), like the
        /// original script, so the caller must not continue.
        #[qinvokable]
        fn decline(self: Pin<&mut Installer>);

        /// Installation succeeded: notify, run post-install hook, exit(0).
        #[qinvokable]
        fn complete_success(self: Pin<&mut Installer>);

        /// Error dialog was dismissed: exit(1).
        #[qinvokable]
        fn quit_error(self: Pin<&mut Installer>);

        #[qsignal]
        fn finished(self: Pin<&mut Installer>, success: bool);
    }

    impl cxx_qt::Threading for Installer {}
}

pub struct InstallerStruct {
    display_name: QString,
    icon: QString,
    progress: i32,
    config: Option<Config>,
    child: Arc<Mutex<Option<Child>>>,
    cancelled: Arc<AtomicBool>,
}

impl Default for InstallerStruct {
    fn default() -> Self {
        let config = env::args().nth(1).and_then(|name| Config::load(&name).ok());
        let (display_name, icon) = match &config {
            Some(cfg) => (QString::from(&cfg.display_name), QString::from(&cfg.icon)),
            None => (
                QString::from("komplete"),
                QString::from("application-x-executable"),
            ),
        };
        Self {
            display_name,
            icon,
            progress: 0,
            config,
            child: Arc::new(Mutex::new(None)),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ffi::Installer {
    pub fn start(self: Pin<&mut Self>) {
        let Some(cfg) = self.config.clone() else {
            self.finished(false);
            return;
        };

        let qt_thread = self.qt_thread();
        let child_slot = Arc::clone(&self.child);
        let cancelled = Arc::clone(&self.cancelled);

        std::thread::spawn(move || {
            let success = install_worker(&cfg, &child_slot, &cancelled, &qt_thread)
                .unwrap_or(false)
                && !cancelled.load(Ordering::SeqCst);
            let _ = qt_thread.queue(move |mut installer| {
                installer.as_mut().finished(success);
            });
        });
    }

    pub fn cancel(self: Pin<&mut Self>) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.child.lock() {
            if let Some(child) = guard.as_mut() {
                let _ = child.kill();
            }
        }
    }

    pub fn decline(self: Pin<&mut Self>) {
        std::process::exit(1); // user cancelled -> must not continue
    }

    pub fn complete_success(self: Pin<&mut Self>) {
        if let Some(cfg) = &self.config {
            config::notify(
                cfg,
                &config::tr1("%1 Ready", &cfg.display_name),
                &gettextrs::gettext("Installation complete."),
            );
            config::run_post_install(cfg);
        }
        std::process::exit(0);
    }

    pub fn quit_error(self: Pin<&mut Self>) {
        std::process::exit(1);
    }
}

/// Runs on the worker thread: add remote, run flatpak install, stream progress.
fn install_worker(
    cfg: &Config,
    child_slot: &Mutex<Option<Child>>,
    cancelled: &AtomicBool,
    qt_thread: &cxx_qt::CxxQtThread<ffi::Installer>,
) -> anyhow::Result<bool> {
    config::add_remote(cfg)?;

    let mut child = Command::new("flatpak")
        .args(["install", "--user", &cfg.app_id, "--assumeyes"])
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().expect("stdout was piped");
    *child_slot.lock().unwrap() = Some(child);

    let mut reader = BufReader::new(stdout);
    let mut pending: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break; // EOF: process exited or was killed by cancel()
        }
        if cancelled.load(Ordering::SeqCst) {
            break;
        }
        pending.extend_from_slice(&chunk[..n]);

        while let Some(pos) = pending.iter().position(|&b| b == b'\r' || b == b'\n') {
            let line: Vec<u8> = pending.drain(..=pos).collect();
            if let Some(pct) = config::parse_percent(&String::from_utf8_lossy(&line)) {
                let _ = qt_thread.queue(move |mut installer| {
                    installer.as_mut().set_progress(pct);
                });
            }
        }
    }

    let status = child_slot
        .lock()
        .unwrap()
        .take()
        .expect("child was stored above")
        .wait()?;
    Ok(status.success())
}
