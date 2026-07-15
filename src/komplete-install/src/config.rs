// SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL
// SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>

//! Non-GUI logic shared by the CLI fast path and the installer QObject.

use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use gettextrs::gettext;
use ini::Ini;

pub const CONFIG_DIR: &str = "/usr/share/komplete";

pub fn fmt1(template: &str, arg: &str) -> String {
    template.replacen("%1", arg, 1)
}

pub fn tr1(msgid: &str, arg: &str) -> String {
    fmt1(&gettext(msgid), arg)
}

/// Parsed komplete config file.
#[derive(Clone)]
pub struct Config {
    pub app_id: String,
    pub display_name: String,
    pub icon: String,
    pub remote_name: String,
    pub remote_url: String,
    pub post_install: String,
}

impl Config {
    pub fn load(name: &str) -> Result<Self> {
        let path = format!("{CONFIG_DIR}/{name}.conf");
        if !Path::new(&path).is_file() {
            bail!("no such config: {path}");
        }
        let ini = Ini::load_from_file(&path).with_context(|| format!("failed to parse {path}"))?;

        let app = ini.section(Some("App")).context("missing [App] section")?;
        let app_id = app.get("Id").context("missing App.Id")?.to_string();
        let display_name = app.get("Name").unwrap_or(&app_id).to_string();
        let icon = app
            .get("Icon")
            .unwrap_or("application-x-executable")
            .to_string();

        let remote = ini.section(Some("Remote")).context("missing [Remote] section")?;
        let remote_name = remote.get("Name").context("missing Remote.Name")?.to_string();
        let remote_url = remote.get("Url").context("missing Remote.Url")?.to_string();

        let post_install = ini
            .section(Some("Install"))
            .and_then(|s| s.get("PostInstall"))
            .unwrap_or("")
            .to_string();

        Ok(Config {
            app_id,
            display_name,
            icon,
            remote_name,
            remote_url,
            post_install,
        })
    }
}

pub fn flatpak_installed(cfg: &Config) -> Result<bool> {
    let out = Command::new("flatpak")
        .args(["list", "--app", "--user", "--columns=application"])
        .output()
        .context("failed to run flatpak")?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.split_whitespace().any(|id| id == cfg.app_id))
}

pub fn notify(cfg: &Config, summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", &cfg.display_name, "-i", &cfg.icon, summary, body])
        .status();
}

pub fn add_remote(cfg: &Config) -> Result<()> {
    let status = Command::new("flatpak")
        .args([
            "remote-add",
            "--if-not-exists",
            "--user",
            &cfg.remote_name,
            &cfg.remote_url,
        ])
        .status()
        .context("failed to run flatpak remote-add")?;
    if !status.success() {
        bail!("flatpak remote-add failed");
    }
    Ok(())
}

/// Refresh the desktop database and run the optional post-install hook.
pub fn run_post_install(cfg: &Config) {
    let home = env::var("HOME").unwrap_or_default();
    let exports = format!("{home}/.local/share/flatpak/exports/share/applications/");
    let _ = Command::new("update-desktop-database").arg(&exports).status();

    if !cfg.post_install.is_empty() {
        if let Some(argv) = shlex::split(&cfg.post_install) {
            if let Some((prog, args)) = argv.split_first() {
                let _ = Command::new(prog).args(args).status();
            }
        }
    }
}

pub fn parse_percent(line: &str) -> Option<i32> {
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'%' && i > 0 {
            let mut start = i;
            while start > 0 && bytes[start - 1].is_ascii_digit() {
                start -= 1;
            }
            if start < i {
                return line[start..i].parse().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_percent;

    #[test]
    fn percent_parsing() {
        assert_eq!(parse_percent("Installing… 42%"), Some(42));
        assert_eq!(parse_percent("1/1 app 100%  512 kB/s"), Some(100));
        assert_eq!(parse_percent("no numbers here"), None);
        assert_eq!(parse_percent("%"), None);
    }
}
