use crate::errors::Error;

use gtk::{Justification, LabelExt, WidgetExt};
use pango::{EllipsizeMode, WrapMode};
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum ErrorLabelMsg {
    Set(Option<Error>),
}

pub struct ErrorLabelModel {
    err: Option<Error>,
    err_text: String,
}

#[widget]
impl Widget for ErrorLabel {
    fn model() -> ErrorLabelModel {
        ErrorLabelModel {
            err: None,
            err_text: "".to_string(),
        }
    }

    fn update(&mut self, event: ErrorLabelMsg) {
        match event {
            ErrorLabelMsg::Set(err) => {
                self.model.err = err.clone();
                if let Some(e) = err {
                    self.model.err_text = format!("{}", e);
                } else {
                    self.model.err_text = "".to_string();
                }
            }
        }
    }

    view! {
        #[name="label"]
        gtk::Label {
            text: &self.model.err_text,
            visible: !self.model.err_text.is_empty(),
            ellipsize: EllipsizeMode::End,
            property_wrap: true,
            property_wrap_mode: WrapMode::Word,
            lines: 2,
            justify: Justification::Center,
        }
    }
}
