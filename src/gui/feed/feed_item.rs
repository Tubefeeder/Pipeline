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
 *
 */

use std::path::PathBuf;

use gdk::cairo::Path;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::WidgetExt;
use relm::factory::{FactoryPrototype, FactoryVec};
use tf_core::{Video, VideoEvent};
use tf_join::AnyVideo;
use tf_observer::Observer;
use tubefeeder_derive::FromUiResource;

use super::FeedPageMsg;

pub struct VideoFactory {
    video: AnyVideo,
    thumbnail: Option<PathBuf>,
    _update: usize,
}

impl VideoFactory {
    pub fn new(v: AnyVideo) -> Self {
        VideoFactory {
            video: v,
            thumbnail: None,
            _update: 0,
        }
    }

    pub fn set_thumbnail(&mut self, thumbnail: PathBuf) {
        self.thumbnail = Some(thumbnail);
    }

    pub fn get(&self) -> AnyVideo {
        self.video.clone()
    }

    pub fn update(&mut self) {
        self._update = self._update.wrapping_add(1)
    }
}

#[derive(FromUiResource, Debug)]
pub struct VideoItemWidgets {
    root: gtk::ListBoxRow,
    box_content: gtk::Box,
    playing: gtk::Image,
    thumbnail: gtk::Image,
    box_info: gtk::Box,
    label_title: gtk::Label,
}

impl FactoryPrototype for VideoFactory {
    type Factory = FactoryVec<Self>;
    type Widgets = VideoItemWidgets;
    type Root = gtk::ListBoxRow;
    type View = gtk::ListBox;
    type Msg = FeedPageMsg;

    fn generate(&self, key: &usize, _sender: glib::Sender<FeedPageMsg>) -> Self::Widgets {
        let widgets = VideoItemWidgets::from_resource("/ui/feed_item.ui");
        self.update(key, &widgets);
        widgets
    }

    fn position(&self, _key: &<Self::Factory as relm::factory::Factory<Self, Self::View>>::Key) {}

    fn update(&self, _key: &usize, widgets: &Self::Widgets) {
        widgets.label_title.set_text(&self.video.title());
        widgets.playing.set_visible(self.video.playing());

        if let Some(thumbnail) = &self.thumbnail {
            widgets.thumbnail.set_from_file(thumbnail);
        }
    }

    fn get_root(widgets: &Self::Widgets) -> &Self::Root {
        &widgets.root
    }
}
