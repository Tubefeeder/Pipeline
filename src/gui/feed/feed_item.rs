use crate::gui::app::AppMsg;
use crate::gui::feed::date_label::DateLabel;
use crate::gui::feed::thumbnail::{Thumbnail, ThumbnailMsg};
use crate::youtube_feed::Entry;

use std::sync::{Arc, Mutex};
use std::thread;

use gtk::prelude::*;
use gtk::{Align, ImageExt, Justification, Orientation, PackType};
use pango::{AttrList, Attribute, EllipsizeMode, WrapMode};
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum FeedListItemMsg {
    SetImage,
    Clicked,
    SetPlaying(bool),
    WatchLater,
}

pub struct FeedListItemModel {
    app_stream: StreamHandle<AppMsg>,
    entry: Entry,
    playing: bool,
    relm: Relm<FeedListItem>,
    killed: Arc<Mutex<bool>>,
}

impl Drop for FeedListItem {
    fn drop(&mut self) {
        let mut killed = self.model.killed.lock().unwrap();
        *killed = true;
    }
}

#[widget]
impl Widget for FeedListItem {
    fn model(
        relm: &Relm<Self>,
        (entry, app_stream): (Entry, StreamHandle<AppMsg>),
    ) -> FeedListItemModel {
        FeedListItemModel {
            app_stream,
            entry,
            playing: false,
            relm: relm.clone(),
            killed: Arc::new(Mutex::new(false)),
        }
    }

    fn update(&mut self, event: FeedListItemMsg) {
        match event {
            FeedListItemMsg::SetImage => {
                self.components.thumbnail.emit(ThumbnailMsg::SetImage);
            }
            FeedListItemMsg::SetPlaying(playing) => {
                self.model.playing = playing;
            }
            FeedListItemMsg::Clicked => {
                let result = self.model.entry.play();

                if let Ok(mut child) = result {
                    let stream = self.model.relm.stream().clone();

                    stream.emit(FeedListItemMsg::SetPlaying(true));

                    let (_channel, sender) = relm::Channel::new(move |_| {
                        stream.emit(FeedListItemMsg::SetPlaying(false));
                    });

                    let killed_arc = self.model.killed.clone();

                    thread::spawn(move || {
                        let _ = child.wait();
                        let killed = (*killed_arc).lock().unwrap();
                        if !*killed {
                            sender.send(()).expect("Could not send message");
                        }
                    });
                }
            }
            FeedListItemMsg::WatchLater => {
                self.model
                    .app_stream
                    .emit(AppMsg::ToggleWatchLater(self.model.entry.clone()));
            }
        }
    }

    fn init_view(&mut self) {
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

        let title_attr_list = AttrList::new();
        title_attr_list.insert(Attribute::new_size(12 * pango::SCALE).unwrap());
        self.widgets
            .label_title
            .set_attributes(Some(&title_attr_list));

        let author_attr_list = AttrList::new();
        author_attr_list.insert(Attribute::new_size(8 * pango::SCALE).unwrap());
        self.widgets
            .label_author
            .set_attributes(Some(&author_attr_list));

        let date_attr_list = author_attr_list.clone();
        self.widgets
            .label_date
            .set_attributes(Some(&date_attr_list));

        self.widgets
            .playing
            .set_from_icon_name(Some("media-playback-start"), gtk::IconSize::LargeToolbar);
    }

    view! {
        gtk::ListBoxRow {
            #[name="box_content"]
            gtk::Box {
                orientation: Orientation::Horizontal,

                #[name="playing"]
                gtk::Image {
                    visible: self.model.playing
                },

                #[name="thumbnail"]
                Thumbnail(self.model.entry.media.thumbnail.clone()),

                #[name="box_info"]
                gtk::Box {
                    orientation: Orientation::Vertical,

                    #[name="label_title"]
                    gtk::Label {
                        text: &self.model.entry.title,
                        ellipsize: EllipsizeMode::End,
                        property_wrap: true,
                        property_wrap_mode: WrapMode::Word,
                        lines: 2,
                        justify: Justification::Left,
                    },
                    #[name="label_author"]
                    gtk::Label {
                        text: &self.model.entry.author.name,
                        ellipsize: EllipsizeMode::End,
                        property_wrap: true,
                        property_wrap_mode: WrapMode::Word,
                        halign: Align::Start
                    },
                    #[name="label_date"]
                    DateLabel(self.model.entry.published.clone()) {}
                },
                #[name="button_watch_later"]
                gtk::Button {
                    clicked => FeedListItemMsg::WatchLater,
                    image: Some(&gtk::Image::from_icon_name(Some("appointment-soon"), gtk::IconSize::LargeToolbar)),
                }
            }
        }
    }
}
