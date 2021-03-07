use crate::subscriptions::channel::Channel;

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use pango::{AttrList, Attribute, EllipsizeMode};
use relm::{Relm, Widget};
use relm_derive::widget;

#[widget]
impl Widget for SubscriptionItem {
    fn model(_: &Relm<Self>, channel: Channel) -> Channel {
        channel
    }

    fn update(&mut self, _: ()) {}

    fn init_view(&mut self) {
        let name_attr_list = AttrList::new();
        name_attr_list.insert(Attribute::new_size(15 * pango::SCALE).unwrap());
        self.widgets
            .label_name
            .set_attributes(Some(&name_attr_list));

        let id_attr_list = AttrList::new();
        id_attr_list.insert(Attribute::new_size(7 * pango::SCALE).unwrap());
        self.widgets.label_id.set_attributes(Some(&id_attr_list));
    }

    view! {
        gtk::ListBoxRow {
            gtk::Box {
                orientation: Vertical,
                #[name="label_name"]
                gtk::Label {
                    text: &self.model.get_name().unwrap_or("".to_string()),
                    ellipsize: EllipsizeMode::End,
                },
                #[name="label_id"]
                gtk::Label {
                    text: &self.model.get_id(),
                    ellipsize: EllipsizeMode::End,
                },
            }
        }
    }
}
