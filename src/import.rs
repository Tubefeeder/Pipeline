use std::collections::HashSet;

use gdk_pixbuf::{gio, prelude::FileExt};
use serde::Deserialize;
use tf_join::{AnySubscription, Joiner};
use tf_yt::YTSubscription;

#[derive(Deserialize)]
struct NewPipeBase {
    subscriptions: Vec<NewPipeSubscription>,
}

#[derive(Deserialize)]
struct NewPipeSubscription {
    url: String,
}

pub fn import_newpipe(joiner: &Joiner, file: gio::File) {
    // TODO: Error handling
    let content: String = String::from_utf8(
        file.load_contents(gio::Cancellable::NONE)
            .expect("Loadable file contents")
            .0,
    )
    .expect("File to be string");

    let deserialized: NewPipeBase =
        serde_json::from_str(content.as_str()).expect("File to be json");

    let subscription_list = joiner.subscription_list();
    let available_subscriptions: HashSet<String> = subscription_list
        .iter()
        .filter_map(|s| {
            if let AnySubscription::Youtube(s) = s {
                Some(s)
            } else {
                None
            }
        })
        .map(|s| s.id())
        .collect();

    let uuids: HashSet<String> = deserialized
        .subscriptions
        .into_iter()
        .map(|s| {
            s.url
                .strip_prefix("https://www.youtube.com/channel/")
                .expect("NewPipe URL to start with the youtube URL")
                .to_owned()
        })
        .collect();

    for uuid in uuids.difference(&available_subscriptions) {
        log::trace!("Subscribing to channel with id {}", uuid);
        let sub = YTSubscription::new(&uuid).into();
        subscription_list.add(sub);
    }
}

pub fn import_youtube(joiner: &Joiner, file: gio::File) {
    // TODO: Error handling
    let content: String = String::from_utf8(
        file.load_contents(gio::Cancellable::NONE)
            .expect("Loadable file contents")
            .0,
    )
    .expect("File to be string");

    let subscription_list = joiner.subscription_list();
    let available_subscriptions: HashSet<String> = subscription_list
        .iter()
        .filter_map(|s| {
            if let AnySubscription::Youtube(s) = s {
                Some(s)
            } else {
                None
            }
        })
        .map(|s| s.id())
        .collect();

    let uuids: HashSet<String> = content
        .lines()
        .skip(1)
        .map(|s| {
            s.split(',')
                .next()
                .expect("YouTube CSV to have at least one column")
                .to_owned()
        })
        .collect();

    for uuid in uuids.difference(&available_subscriptions) {
        log::trace!("Subscribing to channel with id {}", uuid);
        let sub = YTSubscription::new(&uuid).into();
        subscription_list.add(sub);
    }
}
