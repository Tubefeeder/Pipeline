use relm::Widget;
use relm_derive::widget;

use gtk::prelude::*;
use gtk::Orientation::Vertical;

#[widget]
impl Widget for SubscriptionsPage {
    fn model() -> () {}

    fn update(&mut self, _: ()) -> () {}

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Label {
                text: "TODO"
            }
        }
    }
}
