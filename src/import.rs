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

// TODO: Better error handling
pub fn import_newpipe(joiner: &Joiner, file: gio::File) -> Result<(), Box<dyn std::error::Error>> {
    let content: String = String::from_utf8(file.load_contents(gio::Cancellable::NONE)?.0)?;

    let deserialized: NewPipeBase = serde_json::from_str(content.as_str())?;

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
        .filter_map(|s| {
            s.url
                .strip_prefix("https://www.youtube.com/channel/")
                .map(ToOwned::to_owned)
        })
        .collect();

    for uuid in uuids.difference(&available_subscriptions) {
        log::trace!("Subscribing to channel with id {}", uuid);
        let sub = YTSubscription::new(&uuid).into();
        subscription_list.add(sub);
    }
    Ok(())
}

// TODO: Better error handling
pub fn import_youtube(joiner: &Joiner, file: gio::File) -> Result<(), Box<dyn std::error::Error>> {
    let content: String = String::from_utf8(file.load_contents(gio::Cancellable::NONE)?.0)?;

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
    Ok(())
}
