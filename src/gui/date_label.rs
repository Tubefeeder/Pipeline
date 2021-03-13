use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use gtk::{Align, LabelExt, WidgetExt};
use pango::{EllipsizeMode, WrapMode};
use relm::{Relm, Widget};
use relm_derive::widget;

#[widget]
impl Widget for DateLabel {
    fn model(_: &Relm<Self>, date: NaiveDateTime) -> String {
        let local = DateTime::<Local>::from_utc(date, Local.offset_from_utc_datetime(&date));

        local.format("%d.%m.%Y - %H:%M").to_string()
    }

    fn update(&mut self, _: ()) {}

    view! {
        #[name="label"]
        gtk::Label {
            text: &self.model,
            ellipsize: EllipsizeMode::End,
            property_wrap: true,
            property_wrap_mode: WrapMode::Word,
            halign: Align::Start
        }
    }
}
