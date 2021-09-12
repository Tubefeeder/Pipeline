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

use std::sync::Arc;
use std::sync::Mutex;

use gtk::prelude::*;
use gtk::Justification;
use pango::{EllipsizeMode, WrapMode};
use relm::Relm;
use relm::Sender;
use relm::{Channel, Widget};
use relm_derive::{widget, Msg};
use tf_core::{ErrorEvent, ErrorStore};
use tf_observer::{Observable, Observer};

#[derive(Msg)]
pub enum ErrorLabelMsg {
    Set,
    Clear,
}

pub struct ErrorLabelModel {
    err_text: String,

    error_store: ErrorStore,

    _error_observer: Arc<Mutex<Box<dyn Observer<ErrorEvent> + Send>>>,
}

#[widget]
impl Widget for ErrorLabel {
    fn model(relm: &Relm<Self>, error_store: ErrorStore) -> ErrorLabelModel {
        let relm_clone = relm.clone();
        let (_channel, sender) = Channel::new(move |msg| relm_clone.stream().emit(msg));
        let _error_observer = Arc::new(Mutex::new(
            Box::new(ErrorLabelObserver { sender }) as Box<dyn Observer<ErrorEvent> + Send>
        ));

        let mut error_store_clone = error_store.clone();
        error_store_clone.attach(Arc::downgrade(&_error_observer));

        ErrorLabelModel {
            err_text: "".to_string(),

            error_store: error_store_clone,

            _error_observer,
        }
    }

    fn update(&mut self, event: ErrorLabelMsg) {
        match event {
            ErrorLabelMsg::Set => {
                let summary = self.model.error_store.summary();
                // TODO: Nice formatiing

                if summary.network() > 0 {
                    self.model.err_text = "Error connecting to the network".to_string();
                } else if summary.parse() > 0 {
                    self.model.err_text =
                        format!("Error parsing {} subscriptions", summary.parse());
                }
            }
            ErrorLabelMsg::Clear => self.model.err_text = "".to_string(),
        }
    }

    view! {
        #[name="label"]
        gtk::Label {
            text: &self.model.err_text,
            visible: !self.model.err_text.is_empty(),
            ellipsize: EllipsizeMode::End,
            wrap: true,
            wrap_mode: WrapMode::Word,
            lines: 2,
            justify: Justification::Center,
        }
    }
}

pub struct ErrorLabelObserver {
    sender: Sender<ErrorLabelMsg>,
}

impl Observer<ErrorEvent> for ErrorLabelObserver {
    fn notify(&mut self, message: ErrorEvent) {
        log::debug!("Notify label");
        match message {
            ErrorEvent::Add(_error) => {
                let _ = self.sender.send(ErrorLabelMsg::Set);
            }
            ErrorEvent::Clear => {
                let _ = self.sender.send(ErrorLabelMsg::Clear);
            }
        }
    }
}
