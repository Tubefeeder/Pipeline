use crate::gui::feed_list::FeedList;
use crate::gui::feed_list::FeedListMsg::Reload;

use relm::Widget;
use relm_derive::{widget, Msg};

use gtk::prelude::*;
use gtk::Inhibit;
use gtk::Orientation::Vertical;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> () {}

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                gtk::Button {
                    clicked => feed_list@Reload,
                    label: "Reload",
                },
                gtk::Button {
                    clicked => Msg::Quit,
                    label: "Quit",
                },
                #[name="feed_list"]
                FeedList
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
