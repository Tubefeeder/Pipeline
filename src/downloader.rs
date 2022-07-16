/*
 * Copyright 2021 - 2022 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder.
 *
 * Tubefeeder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{fmt::Display, process::Command, thread};

const DOWNLOAD_MERGE: &str = "[Merger] Merging formats into ";
const DOWNLOAD_DESTINATION: &str = "[download] Destination: ";

pub fn download<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: Fn(Option<String>) + std::marker::Send + 'static + std::marker::Sync,
>(
    url: S,
    callback: F,
) {
    log::debug!("Downloading video with url: {}", url);
    let download_dir = std::env::var("XDG_DOWNLOAD_DIR")
        .unwrap_or("$HOME/Downloads/%(title)s-%(id)s.%(ext)s".to_string());
    let downloader_str =
        std::env::var("DOWNLOADER").unwrap_or(format!("youtube-dl --output {}", download_dir));
    open_with_output(url, downloader_str, move |output| {
        callback(
            output
                .lines()
                .into_iter()
                .rev()
                .find(|s| s.starts_with(DOWNLOAD_MERGE) || s.starts_with(DOWNLOAD_DESTINATION))
                .map(|s| s.strip_prefix(DOWNLOAD_MERGE).unwrap_or(s))
                .map(|s| s.strip_prefix(DOWNLOAD_DESTINATION).unwrap_or(s))
                .map(|s| s.trim_matches('"').to_owned()),
        )
    });
}

pub fn open_with_output<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: Fn(String) + std::marker::Send + 'static,
>(
    url: S,
    command: String,
    callback: F,
) {
    thread::spawn(move || {
        let mut command_iter = command.split(" ");
        let program = command_iter
            .next()
            .expect("The command should have a program");
        let args: Vec<String> = command_iter.map(|s| s.to_string()).collect();

        let out = Command::new(&program).args(args).arg(url).output();

        callback(String::from_utf8_lossy(&out.map(|o| o.stdout).unwrap_or_default()).to_string());
    });
}
