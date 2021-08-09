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
