use crate::gui::app::AppMsg;

use std::str::FromStr;

use gtk::{ButtonExt, WidgetExt};
use libhandy::HeaderBarExt;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

const STARTING_PAGE: Page = Page::Feed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Feed,
    Subscriptions,
}

impl Page {
    fn get_all_values() -> Vec<Page> {
        vec![Page::Feed, Page::Subscriptions]
    }
}

impl FromStr for Page {
    type Err = ();

    fn from_str(string: &str) -> Result<Page, Self::Err> {
        let all_values = Page::get_all_values();

        for val in &all_values {
            if &format!("{:?}", val) == string {
                return Ok(val.clone());
            }
        }

        return Err(());
    }
}

#[derive(Msg)]
pub enum HeaderBarMsg {
    SetPage(Page),
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
            app_stream: app_stream.clone(),
            page: STARTING_PAGE,
            title: format!("{:?}", STARTING_PAGE),
        }
    }

    fn update(&mut self, event: HeaderBarMsg) {
        match event {
            HeaderBarMsg::SetPage(page) => self.set_page(page),
            HeaderBarMsg::Reload => self.model.app_stream.emit(AppMsg::Reload),
        }
    }

    fn set_page(&mut self, page: Page) {
        self.model.page = page.clone();
        self.model.title = format!("{:?}", page);
    }

    view! {
        #[name="header_bar"]
        libhandy::HeaderBar {
            title: Some(&self.model.title),

            #[name="button_reload"]
            gtk::Button {
                label: "Reload",
                clicked => HeaderBarMsg::Reload,
                visible: self.model.page == Page::Feed
            }
        }
    }
}
