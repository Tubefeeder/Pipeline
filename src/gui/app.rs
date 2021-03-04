use crate::gui::feed_page::FeedPage;
use crate::gui::subscriptions_page::SubscriptionsPage;

use relm::Widget;
use relm_derive::{widget, Msg};

use gtk::prelude::*;
use gtk::Inhibit;
use gtk::Orientation::Vertical;

use libhandy::ViewSwitcherBarBuilder;

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

    fn init_view(&mut self) {
        let view_switcher = ViewSwitcherBarBuilder::new()
            .stack(&self.widgets.application_stack)
            .reveal(true)
            .build();
        self.widgets.view_switcher_box.add(&view_switcher);
        self.widgets.view_switcher_box.show_all();
    }

    view! {
        gtk::Window {
            #[name="view_switcher_box"]
            gtk::Box {
                orientation: Vertical,
                #[name="application_stack"]
                gtk::Stack {
                    FeedPage {
                        child: {
                            title: Some("Feed")
                        }
                    },
                    SubscriptionsPage {
                        child: {
                            title: Some("Subscriptions")
                        }
                    }
                },
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
