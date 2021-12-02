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

use std::{
    process::{Command, Stdio},
    thread,
};

use tf_core::Video;
use tf_join::AnyVideo;

pub fn download(video: AnyVideo) {
    thread::spawn(move || {
        log::debug!("Downloading video with title: {}", video.title());
        log::debug!("Downloading video with url: {}", video.url());
        video.play();
        let download_dir = std::env::var("XDG_DOWNLOAD_DIR")
            .unwrap_or("$HOME/Downloads/%(title)s-%(id)s.%(ext)s".to_string());
        let downloader_str =
            std::env::var("DOWNLOADER").unwrap_or(format!("youtube-dl --output {}", download_dir));

        let mut downloader_iter = downloader_str.split(" ");
        let downloader = downloader_iter.next().unwrap_or("youtube-dl");
        let args: Vec<String> = downloader_iter.map(|s| s.to_string()).collect();

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

        let _ = Command::new(&downloader)
            .args(args)
            .arg(video.url())
            .stdout(stdout)
            .stderr(stderr)
            .stdin(Stdio::null())
            .spawn()
            .unwrap()
            .wait();
        log::debug!("Stopped downloading with title: {}", video.title());
        video.stop();
    });
}
