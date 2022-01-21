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

use std::{
    collections::HashMap,
    convert::TryInto,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use relm::{
    factory::{Factory, FactoryVec},
    AsyncComponentUpdate, AsyncRelmWorker, ComponentUpdate, Components, MessageHandler, Model,
    RelmMsgHandler, Widgets,
};
use tf_core::{ErrorStore, Generator, Video, VideoEvent};
use tf_join::{AnyVideo, Joiner};

use tf_observer::{Observable, Observer};
use tf_playlist::PlaylistManager;
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::{channel, Sender},
};
use tubefeeder_derive::FromUiResource;

use gtk::prelude::ListBoxRowExt;

use crate::{
    gui::{
        app::{AppMsg, AppWidgets},
        AppModel,
    },
    player::play,
};

use super::feed_item::VideoFactory;

const PAGE_SIZE: usize = 10;

pub struct FeedPageModel {
    joiner: Joiner,
    errors: ErrorStore,
    playlist_manager: PlaylistManager<String, AnyVideo>,

    videos: FactoryVec<VideoFactory>,
    observers: HashMap<i32, Arc<Mutex<Box<dyn Observer<VideoEvent> + Send>>>>,
}

pub enum FeedPageMsg {
    Clicked(i32),
    Update(i32),
    SetThumbnail(usize, PathBuf),
    AddVideos(Vec<AnyVideo>),
    Reload,
}

#[derive(FromUiResource)]
pub struct FeedPageWidgets {
    root: gtk::ScrolledWindow,
    video_list: gtk::ListBox,
}

pub struct FeedPageComponents {
    reload: AsyncRelmWorker<ReloadModel, FeedPageModel>,
    thumbnail: RelmMsgHandler<ThumbnailHandler, FeedPageModel>,
}

impl Model for FeedPageModel {
    type Msg = FeedPageMsg;
    type Widgets = FeedPageWidgets;
    type Components = FeedPageComponents;
}

impl ComponentUpdate<AppModel> for FeedPageModel {
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: relm::Sender<Self::Msg>,
        _parent_sender: relm::Sender<AppMsg>,
    ) {
        match msg {
            FeedPageMsg::Clicked(i) => {
                let video_res = self.videos.get_mut(i.try_into().unwrap_or(0));
                if let Some(video) = video_res {
                    play(video.get());

                    if self.observers.get(&i).is_none() {
                        let observer =
                            Arc::new(Mutex::new(
                                Box::new(FeedListItemObserver { sender, index: i })
                                    as Box<(dyn Observer<VideoEvent> + Send + 'static)>,
                            ));
                        video.get().attach(Arc::downgrade(&observer));
                        self.observers.insert(i, observer);
                    }
                }
            }
            FeedPageMsg::Update(i) => {
                let video_res = self.videos.get_mut(i.try_into().unwrap_or(0));
                if let Some(video) = video_res {
                    video.update();
                }
            }
            FeedPageMsg::SetThumbnail(i, thumbnail) => {
                let video_res = self.videos.get_mut(i.try_into().unwrap_or(0));
                if let Some(video) = video_res {
                    video.set_thumbnail(thumbnail);
                }
            }
            FeedPageMsg::AddVideos(videos) => {
                for v in videos {
                    let factory = VideoFactory::new(v.clone());
                    let index = self.videos.len();
                    self.videos.push(factory);
                    components.thumbnail.send(ThumbnailMsg::Get(index, v));
                }
            }
            FeedPageMsg::Reload => {
                self.videos.clear();
                self.observers.clear();
                let _ = components.reload.send(ReloadMsg::Reload);
            }
        }
    }

    fn init_model(parent_model: &AppModel) -> Self {
        FeedPageModel {
            joiner: parent_model.joiner.clone(),
            errors: parent_model.errors.clone(),
            playlist_manager: parent_model.playlist_manager.clone(),
            videos: FactoryVec::new(),
            observers: HashMap::new(),
        }
    }
}

impl Widgets<FeedPageModel, AppModel> for FeedPageWidgets {
    type Root = gtk::ScrolledWindow;

    fn init_view(
        _model: &FeedPageModel,
        _components: &FeedPageComponents,
        sender: relm::Sender<FeedPageMsg>,
    ) -> Self {
        let widgets = FeedPageWidgets::from_resource("/ui/feed_page.ui");
        widgets
            .video_list
            .connect_row_activated(move |_listbox, listboxrow| {
                let position = listboxrow.index();
                let _ = sender.send(FeedPageMsg::Clicked(position));
            });
        widgets
    }

    fn root_widget(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(&mut self, model: &FeedPageModel, sender: relm::Sender<FeedPageMsg>) {
        model.videos.generate(&self.video_list, sender);
    }
}

impl Components<FeedPageModel> for FeedPageComponents {
    fn init_components(
        parent_model: &FeedPageModel,
        parent_sender: relm::Sender<FeedPageMsg>,
    ) -> Self {
        FeedPageComponents {
            reload: AsyncRelmWorker::with_new_tokio_rt(parent_model, parent_sender.clone()),
            thumbnail: RelmMsgHandler::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, _parent_widgets: &FeedPageWidgets) {}
}

pub struct FeedListItemObserver {
    sender: relm::Sender<FeedPageMsg>,
    index: i32,
}

impl Observer<VideoEvent> for FeedListItemObserver {
    fn notify(&mut self, _message: VideoEvent) {
        let _ = self.sender.send(FeedPageMsg::Update(self.index));
    }
}

struct ReloadModel {
    joiner: Joiner,
    errors: ErrorStore,
    videos: Vec<AnyVideo>,
}

#[derive(Debug)]
enum ReloadMsg {
    Reload,
    More,
}

impl Model for ReloadModel {
    type Msg = ReloadMsg;
    type Widgets = ();
    type Components = ();
}

#[relm::async_trait]
impl AsyncComponentUpdate<FeedPageModel> for ReloadModel {
    fn init_model(parent_model: &FeedPageModel) -> Self {
        ReloadModel {
            joiner: parent_model.joiner.clone(),
            errors: parent_model.errors.clone(),
            videos: vec![],
        }
    }

    async fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        sender: relm::Sender<Self::Msg>,
        parent_sender: relm::Sender<FeedPageMsg>,
    ) {
        match msg {
            ReloadMsg::Reload => {
                self.videos = self.joiner.generate(&self.errors).await.collect();
                let _ = sender.send(ReloadMsg::More);
            }
            ReloadMsg::More => {
                if self.videos.len() < PAGE_SIZE {
                    let _ = parent_sender.send(FeedPageMsg::AddVideos(self.videos.clone()));
                    self.videos = vec![];
                } else {
                    let page = self.videos.drain(..PAGE_SIZE).collect();
                    let _ = parent_sender.send(FeedPageMsg::AddVideos(page));
                }
            }
        }
    }
}

struct ThumbnailHandler {
    _rt: Runtime,
    sender: Sender<ThumbnailMsg>,
}

enum ThumbnailMsg {
    Get(usize, AnyVideo),
}

impl MessageHandler<FeedPageModel> for ThumbnailHandler {
    type Msg = ThumbnailMsg;
    type Sender = Sender<ThumbnailMsg>;

    fn init(_parent_model: &FeedPageModel, parent_sender: relm::Sender<FeedPageMsg>) -> Self {
        log::debug!("Initialized ThumbnailHandler");
        let (sender, mut rx) = channel::<ThumbnailMsg>(50);

        let rt = Builder::new_multi_thread()
            .worker_threads(10)
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        let mut cache_dir = glib::user_cache_dir();
        cache_dir.push("tubefeeder");

        rt.spawn(async move {
            while let Some(msg) = rx.recv().await {
                let parent_sender = parent_sender.clone();
                let mut cache_dir = cache_dir.clone();
                tokio::spawn(async move {
                    match msg {
                        ThumbnailMsg::Get(idx, v) => {
                            cache_dir.push(&format!("{}.png", v.title()));

                            if !cache_dir.exists() {
                                let thumbnail = v.thumbnail().await;
                                let resized = thumbnail.resize(
                                    120,
                                    90,
                                    image::imageops::FilterType::Triangle,
                                );
                                let _ = resized.save(&cache_dir);
                            }

                            let _ = parent_sender.send(FeedPageMsg::SetThumbnail(idx, cache_dir));
                        }
                    }
                });
            }
        });

        ThumbnailHandler { _rt: rt, sender }
    }

    fn send(&self, msg: Self::Msg) {
        let sender = self.sender();
        // No idea why spawn_blocking is needed.
        tokio::task::spawn_blocking(move || {
            let _ = sender.blocking_send(msg);
        });
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}
