use crate::gui::feed_list::{FeedList, FeedListMsg::Reload};

use relm::Widget;
use relm_derive::widget;

use gtk::prelude::*;
use gtk::Orientation::Vertical;

#[widget]
impl Widget for FeedPage {
    fn model() -> () {}

    fn update(&mut self, _: ()) -> () {}

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Button {
                label: "Reload",
                clicked => feed_list@Reload

            },
            #[name="feed_list"]
            FeedList
        }
    }
}
