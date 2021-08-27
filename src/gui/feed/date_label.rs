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

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use gtk::prelude::*;
use gtk::Align;
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
            wrap: true,
            wrap_mode: WrapMode::Word,
            halign: Align::Start
        }
    }
}
