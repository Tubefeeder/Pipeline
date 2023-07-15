/*
 * Copyright 2021 - 2022 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Pipeline.
 *
 * Pipeline is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Pipeline is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Pipeline.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    fmt::Display,
    process::{Command, Stdio},
    thread,
};

pub fn play<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: Fn() + std::marker::Send + 'static,
>(
    url: S,
    callback: F,
) {
    log::debug!("Playing video with url: {}", url);
    let player_str = std::env::var("PLAYER").unwrap_or("mpv --ytdl".to_string());
    open_with(url, player_str, callback);
}

pub fn open_with<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: Fn() + std::marker::Send + 'static,
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

        let stdout = if log::log_enabled!(log::Level::Debug) {
            Stdio::inherit()
        } else {
            Stdio::null()
        };

        let stderr = if log::log_enabled!(log::Level::Error) {
            Stdio::inherit()
        } else {
            Stdio::null()
        };

        let _ = Command::new(&program)
            .args(args)
            .arg(url)
            .stdout(stdout)
            .stderr(stderr)
            .stdin(Stdio::null())
            .spawn()
            .unwrap()
            .wait();

        callback();
    });
}
