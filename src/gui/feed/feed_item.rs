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

use std::sync::{Arc, Mutex};

use crate::downloader::download;
use crate::gui::feed::date_label::DateLabel;
use crate::gui::feed::thumbnail::{Thumbnail, ThumbnailMsg};
use crate::gui::{get_font_size, FONT_RATIO};
use crate::player::play;

use tf_core::{Video, VideoEvent};
use tf_join::AnyVideo;
use tf_observer::{Observable, Observer};
use tf_playlist::PlaylistManager;

use gtk::prelude::*;
use gtk::{Align, Justification, Orientation, PackType};
use pango::{AttrList, Attribute, EllipsizeMode, WrapMode};
use relm::{Channel, Relm, Sender, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum FeedListItemMsg {
    SetImage,
    Clicked,
    SetPlaying(bool),
    WatchLater,
    Expand,
    Clipboard,
    Download,
}

pub struct FeedListItemModel {
    entry: AnyVideo,
    expanded: bool,
    playing: bool,
    relm: Relm<FeedListItem>,
    observer: Arc<Mutex<Box<dyn Observer<VideoEvent> + Send>>>,
    playlist_manager: PlaylistManager<String, AnyVideo>,

    client: reqwest::Client,
}

#[widget]
impl Widget for FeedListItem {
    fn model(
        relm: &Relm<Self>,
        (entry, client, playlist_manager): (
            AnyVideo,
            reqwest::Client,
            PlaylistManager<String, AnyVideo>,
        ),
    ) -> FeedListItemModel {
        let relm_clone = relm.clone();
        let (_channel, sender) = Channel::new(move |msg| {
            relm_clone.stream().emit(msg);
        });
        FeedListItemModel {
            entry,
            expanded: false,
            playing: false,
            relm: relm.clone(),
            observer: Arc::new(Mutex::new(Box::new(FeedListItemObserver { sender }))),
            playlist_manager,

            client,
        }
    }

    fn update(&mut self, event: FeedListItemMsg) {
        match event {
            FeedListItemMsg::SetImage => {
                self.components.thumbnail.emit(ThumbnailMsg::SetImage);
            }
            FeedListItemMsg::SetPlaying(playing) => {
                self.model.playing = playing;
                self.widgets.box_content.show();
            }
            FeedListItemMsg::Clicked => {
                play(self.model.entry.clone());
            }
            FeedListItemMsg::WatchLater => {
                self.model
                    .playlist_manager
                    .toggle(&("WATCHLATER".to_string()), &self.model.entry);
            }
            FeedListItemMsg::Expand => {
                self.model.expanded = !self.model.expanded;
                let expand_icon_name = if self.model.expanded {
                    "arrow1-right-symbolic"
                } else {
                    "arrow1-left-symbolic"
                };
                self.widgets
                    .button_expand
                    .set_image(Some(&gtk::Image::from_icon_name(
                        Some(expand_icon_name),
                        gtk::IconSize::LargeToolbar,
                    )));
            }
            FeedListItemMsg::Clipboard => {
                let clipboard = gtk::Clipboard::get(&gdk::Atom::intern("CLIPBOARD"));
                // Replace // with / because of simple bug I am too lazy to fix in the youtube-extractor.
                clipboard.set_text(&self.model.entry.url().replace("//watch", "/watch"));
                clipboard.store();
            }
            FeedListItemMsg::Download => download(self.model.entry.clone()),
        }
    }

    fn init_view(&mut self) {
        self.model
            .entry
            .attach(Arc::downgrade(&self.model.observer));
        self.widgets.box_content.set_child_packing(
            &self.widgets.button_watch_later,
            false,
            true,
            0,
            PackType::End,
        );

        self.widgets.box_content.set_child_packing(
            &self.widgets.box_info,
            true,
            true,
            0,
            PackType::Start,
        );

        let font_size = get_font_size();

        let title_attr_list = AttrList::new();
        title_attr_list.insert(Attribute::new_size(font_size * pango::SCALE));
        self.widgets
            .label_title
            .set_attributes(Some(&title_attr_list));

        let small_text_attr_list = AttrList::new();
        small_text_attr_list.insert(Attribute::new_size(
            (FONT_RATIO * (font_size * pango::SCALE) as f32) as i32,
        ));

        self.widgets
            .label_author
            .set_attributes(Some(&small_text_attr_list));
        self.widgets
            .label_platform
            .set_attributes(Some(&small_text_attr_list));
        self.widgets
            .label_date
            .set_attributes(Some(&small_text_attr_list));

        self.widgets.playing.set_from_icon_name(
            Some("media-playback-start-symbolic"),
            gtk::IconSize::LargeToolbar,
        );

        self.model
            .relm
            .stream()
            .emit(FeedListItemMsg::SetPlaying(self.model.entry.playing()));
    }

    view! {
        #[name="root"]
        gtk::ListBoxRow {
            #[name="box_content"]
            gtk::Box {
                orientation: Orientation::Horizontal,
                spacing: 8,

                #[name="playing"]
                gtk::Image {
                    visible: self.model.playing
                },

                #[name="thumbnail"]
                Thumbnail(self.model.entry.clone(), self.model.client.clone()),

                #[name="box_info"]
                gtk::Box {
                    orientation: Orientation::Vertical,
                    spacing: 4,

                    #[name="label_title"]
                    gtk::Label {
                        text: &self.model.entry.title(),
                        ellipsize: EllipsizeMode::End,
                        wrap: true,
                        wrap_mode: WrapMode::Word,
                        lines: 2,
                        justify: Justification::Left,
                    },
                    gtk::Box {
                        spacing: 4,
                        #[name="label_author"]
                        gtk::Label {
                            text: &self.model.entry.subscription().to_string(),
                            ellipsize: EllipsizeMode::End,
                            wrap: true,
                            wrap_mode: WrapMode::Word,
                            halign: Align::Start
                        },
                        #[name="label_platform"]
                        gtk::Label {
                            text: &("(".to_owned() + &self.model.entry.platform().to_string() + ")"),
                            ellipsize: EllipsizeMode::End,
                            wrap: true,
                            wrap_mode: WrapMode::Word,
                            halign: Align::Start
                        },
                    },
                    #[name="label_date"]
                    DateLabel(self.model.entry.uploaded().clone()) {}
                },
                #[name="button_expand"]
                gtk::Button {
                    clicked => FeedListItemMsg::Expand,
                    image: Some(&gtk::Image::from_icon_name(Some("arrow1-left-symbolic"), gtk::IconSize::LargeToolbar)),
                },
                gtk::Box {
                    visible: self.model.expanded,

                    #[name="button_clipboard"]
                    gtk::Button {
                        clicked => FeedListItemMsg::Clipboard,
                        image: Some(&gtk::Image::from_icon_name(Some("clipboard-symbolic"), gtk::IconSize::LargeToolbar)),
                    },
                    #[name="button_download"]
                    gtk::Button {
                        clicked => FeedListItemMsg::Download,
                        image: Some(&gtk::Image::from_icon_name(Some("folder-download-symbolic"), gtk::IconSize::LargeToolbar)),
                    }
                },
                #[name="button_watch_later"]
                gtk::Button {
                    clicked => FeedListItemMsg::WatchLater,
                    image: Some(&gtk::Image::from_icon_name(Some("appointment-new-symbolic"), gtk::IconSize::LargeToolbar)),
                }
            }
        }
    }
}

pub struct FeedListItemObserver {
    sender: Sender<FeedListItemMsg>,
}

impl Observer<VideoEvent> for FeedListItemObserver {
    fn notify(&mut self, message: VideoEvent) {
        match message {
            VideoEvent::Play => {
                let _ = self.sender.send(FeedListItemMsg::SetPlaying(true));
            }
            VideoEvent::Stop => {
                let _ = self.sender.send(FeedListItemMsg::SetPlaying(false));
            }
        }
    }
}
