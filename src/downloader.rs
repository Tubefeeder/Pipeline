/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
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

use std::fmt::Display;

use crate::player::open_with;

pub fn download<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: Fn() + std::marker::Send + 'static,
>(
    url: S,
    callback: F,
) {
    log::debug!("Downloading video with url: {}", url);
    let download_dir = std::env::var("XDG_DOWNLOAD_DIR")
        .unwrap_or("$HOME/Downloads/%(title)s-%(id)s.%(ext)s".to_string());
    let downloader_str =
        std::env::var("DOWNLOADER").unwrap_or(format!("youtube-dl --output {}", download_dir));
    open_with(url, downloader_str, callback);
}
