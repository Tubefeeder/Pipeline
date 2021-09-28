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

use crate::gui::app::AppMsg;

use std::convert::{From, Into};
use std::str::FromStr;

use gtk::prelude::*;
use gtk::AboutDialogBuilder;
use libhandy::traits::HeaderBarExt;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

const STARTING_PAGE: Page = Page::Feed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Feed,
    WatchLater,
    Filters,
    Subscriptions,
}

impl Page {
    fn get_all_values() -> Vec<Page> {
        vec![
            Page::Feed,
            Page::WatchLater,
            Page::Filters,
            Page::Subscriptions,
        ]
    }
}

impl FromStr for Page {
    type Err = ();

    fn from_str(string: &str) -> Result<Page, Self::Err> {
        let all_values = Page::get_all_values();
        let owned = string.to_owned();

        for val in &all_values {
            let val_str: String = val.clone().into();
            if val_str == owned {
                return Ok(val.clone());
            }
        }

        Err(())
    }
}

impl From<Page> for String {
    fn from(page: Page) -> Self {
        match page {
            Page::WatchLater => "Watch Later".to_string(),
            _ => format!("{:?}", page),
        }
    }
}

#[derive(Msg)]
pub enum HeaderBarMsg {
    SetPage(Page),
    AddSubscription,
    AddFilter,
    Reload,
    About,
}

pub struct HeaderBarModel {
    app_stream: StreamHandle<AppMsg>,
    page: Page,
    title: String,
    relm: Relm<HeaderBar>,
}

#[widget]
impl Widget for HeaderBar {
    fn model(relm: &Relm<Self>, app_stream: StreamHandle<AppMsg>) -> HeaderBarModel {
        HeaderBarModel {
            app_stream,
            page: STARTING_PAGE,
            title: STARTING_PAGE.into(),
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: HeaderBarMsg) {
        match event {
            HeaderBarMsg::SetPage(page) => self.set_page(page),
            HeaderBarMsg::Reload => self.model.app_stream.emit(AppMsg::Reload),
            HeaderBarMsg::AddSubscription => {
                self.model.app_stream.emit(AppMsg::ToggleAddSubscription)
            }
            HeaderBarMsg::AddFilter => self.model.app_stream.emit(AppMsg::ToggleAddFilter),
            HeaderBarMsg::About => {
                let about_dialog = AboutDialogBuilder::new()
                    .authors(vec!["Julian Schmidhuber".to_string()])
                    .comments("A Youtube-Client made for the Pinephone")
                    .copyright(
                        include_str!("../../NOTICE")
                            .to_string()
                            .lines()
                            .next()
                            .unwrap_or_default(),
                    )
                    .license_type(gtk::License::Gpl30)
                    .logo_icon_name("icon")
                    .program_name("Tubefeeder")
                    .version("1.3.0")
                    .website("https://www.tubefeeder.de")
                    .build();
                about_dialog.show();
            }
        }
    }

    fn set_page(&mut self, page: Page) {
        self.model.page = page.clone();
        self.model.title = page.into();
    }

    fn init_view(&mut self) {
        let menu_button = gtk::MenuButton::new();
        menu_button.set_image(Some(&gtk::Image::from_icon_name(
            Some("open-menu-symbolic"),
            gtk::IconSize::LargeToolbar,
        )));

        let menu = gtk::Menu::new();
        let about_item = gtk::MenuItem::with_label("About");
        relm::connect!(
            self.model.relm,
            about_item,
            connect_activate(_),
            HeaderBarMsg::About
        );
        menu.append(&about_item);
        menu.show_all();
        menu_button.set_popup(Some(&menu));

        self.widgets.header_bar.pack_end(&menu_button);
        self.widgets.header_bar.show_all();
    }

    view! {
        #[name="header_bar"]
        libhandy::HeaderBar {
            title: Some(&self.model.title),
            show_close_button: true,

            gtk::Button {
                image: Some(&gtk::Image::from_icon_name(Some("view-refresh-symbolic"), gtk::IconSize::LargeToolbar)),
                clicked => HeaderBarMsg::Reload,
                visible: self.model.page == Page::Feed
            },
            gtk::Button {
                image: Some(&gtk::Image::from_icon_name(Some("list-add-symbolic"), gtk::IconSize::LargeToolbar)),
                clicked => HeaderBarMsg::AddFilter,
                visible: self.model.page == Page::Filters
            },
            gtk::Button {
                image: Some(&gtk::Image::from_icon_name(Some("list-add-symbolic"), gtk::IconSize::LargeToolbar)),
                clicked => HeaderBarMsg::AddSubscription,
                visible: self.model.page == Page::Subscriptions
            },
        }
    }
}
