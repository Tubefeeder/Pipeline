mod errors;
mod gui;
mod subscriptions;
mod youtube_feed;

use crate::gui::app::Win;

use relm::Widget;

#[tokio::main]
async fn main() {
    Win::run(()).unwrap();
}
