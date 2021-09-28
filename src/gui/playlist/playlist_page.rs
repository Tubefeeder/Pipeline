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

use crate::gui::feed::feed_item::{FeedListItem, FeedListItemMsg};

use std::sync::{Arc, Mutex};

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::{Channel, Relm, Sender, Widget};
use relm_derive::{widget, Msg};
use tf_join::AnyVideo;
use tf_observer::Observer;
use tf_playlist::{PlaylistEvent, PlaylistManager};

#[derive(Msg)]
pub enum PlaylistPageMsg {
    NewVideo(AnyVideo),
    RemoveVideo(AnyVideo),
    Clicked(usize),
}

pub struct PlaylistPageModel {
    playlist_manager: PlaylistManager<String, AnyVideo>,
    _playlist_observer: Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>,
    video_items: Vec<(AnyVideo, relm::Component<FeedListItem>)>,
    client: reqwest::Client,
}

#[widget]
impl Widget for PlaylistPage {
    fn model(
        relm: &Relm<Self>,
        (playlist_manager, name): (PlaylistManager<String, AnyVideo>, String),
    ) -> PlaylistPageModel {
        let relm_clone = relm.clone();
        let (_channel, sender) = Channel::new(move |msg| relm_clone.stream().emit(msg));

        let observer = Arc::new(Mutex::new(Box::new(PlaylistPageObserver { sender })
            as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

        let mut playlist_manager_clone = playlist_manager.clone();
        playlist_manager_clone
            .items(&name)
            .iter()
            .for_each(|s| relm.stream().emit(PlaylistPageMsg::NewVideo(s.clone())));
        playlist_manager_clone.attach_at(Arc::downgrade(&observer), &name);

        PlaylistPageModel {
            playlist_manager: playlist_manager_clone,
            _playlist_observer: observer,
            video_items: Vec::new(),
            client: reqwest::Client::new(),
        }
    }

    fn update(&mut self, event: PlaylistPageMsg) {
        match event {
            PlaylistPageMsg::NewVideo(v) => self.new_video(v),
            PlaylistPageMsg::RemoveVideo(v) => self.remove_video(v),
            PlaylistPageMsg::Clicked(i) => {
                self.model.video_items[i].1.emit(FeedListItemMsg::Clicked);
            }
        }
    }

    fn new_video(&mut self, video: AnyVideo) {
        let video_item = relm::create_component::<FeedListItem>((
            video.clone(),
            self.model.client.clone(),
            self.model.playlist_manager.clone(),
        ));
        self.widgets.video_list.prepend(video_item.widget());

        video_item.emit(FeedListItemMsg::SetImage);

        self.model.video_items.insert(0, (video, video_item));
    }

    fn remove_video(&mut self, video: AnyVideo) {
        for (item, widget) in &self.model.video_items {
            if item == &video {
                self.widgets.video_list.remove(widget.widget());
                self.widgets.root.show();
                self.widgets.video_list.show();
            }
        }

        self.model.video_items.retain(|(i, _)| i != &video);
    }

    view! {
        #[name="root"]
        gtk::Box {
            orientation: Vertical,

            gtk::ScrolledWindow {
                hexpand: true,
                vexpand: true,
                gtk::Viewport {
                    #[name="video_list"]
                    gtk::ListBox {
                        selection_mode: gtk::SelectionMode::None,
                        row_activated(listbox, row) => {
                            let index =
                                listbox
                                .children()
                                .iter()
                                .position(|x| x.clone() == row.clone().upcast::<gtk::Widget>())
                                .unwrap();

                            PlaylistPageMsg::Clicked(index)
                        }
                    }
                }
            }
        }
    }
}

pub struct PlaylistPageObserver {
    sender: Sender<PlaylistPageMsg>,
}

impl Observer<PlaylistEvent<AnyVideo>> for PlaylistPageObserver {
    fn notify(&mut self, message: PlaylistEvent<AnyVideo>) {
        match message {
            PlaylistEvent::Add(video) => {
                let _ = self.sender.send(PlaylistPageMsg::NewVideo(video));
            }
            PlaylistEvent::Remove(video) => {
                let _ = self.sender.send(PlaylistPageMsg::RemoveVideo(video));
            }
        }
    }
}
