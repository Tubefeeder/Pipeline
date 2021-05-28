use crate::gui::app::AppMsg;

use std::convert::{From, Into};
use std::str::FromStr;

use gtk::{ButtonExt, WidgetExt};
use libhandy::HeaderBarExt;
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
}

pub struct HeaderBarModel {
    app_stream: StreamHandle<AppMsg>,
    page: Page,
    title: String,
}

#[widget]
impl Widget for HeaderBar {
    fn model(_relm: &Relm<Self>, app_stream: StreamHandle<AppMsg>) -> HeaderBarModel {
        HeaderBarModel {
            app_stream,
            page: STARTING_PAGE,
            title: STARTING_PAGE.into(),
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
        }
    }

    fn set_page(&mut self, page: Page) {
        self.model.page = page.clone();
        self.model.title = page.into();
    }

    view! {
        #[name="header_bar"]
        libhandy::HeaderBar {
            title: Some(&self.model.title),

            gtk::Button {
                image: Some(&gtk::Image::from_icon_name(Some("view-refresh"), gtk::IconSize::LargeToolbar)),
                clicked => HeaderBarMsg::Reload,
                visible: self.model.page == Page::Feed
            },
            gtk::Button {
                image: Some(&gtk::Image::from_icon_name(Some("list-add"), gtk::IconSize::LargeToolbar)),
                clicked => HeaderBarMsg::AddFilter,
                visible: self.model.page == Page::Filters
            },
            gtk::Button {
                image: Some(&gtk::Image::from_icon_name(Some("list-add"), gtk::IconSize::LargeToolbar)),
                clicked => HeaderBarMsg::AddSubscription,
                visible: self.model.page == Page::Subscriptions
            }
        }
    }
}
