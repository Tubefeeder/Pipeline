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

pub fn play(video: AnyVideo) {
    thread::spawn(move || {
        log::debug!("Playing video with title: {}", video.title());
        video.play();
        let player_str = std::env::var("PLAYER").unwrap_or("mpv".to_string());

        let mut player_iter = player_str.split(" ");
        let player = player_iter.next().unwrap_or("mpv");
        let args: Vec<String> = player_iter.map(|s| s.to_string()).collect();

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

        let _ = Command::new(&player)
            .args(args)
            .arg(video.url())
            .stdout(stdout)
            .stderr(stderr)
            .stdin(Stdio::null())
            .spawn()
            .unwrap()
            .wait();
        log::debug!("Stopped video with title: {}", video.title());
        video.stop();
    });
}
