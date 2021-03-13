mod errors;
mod filter;
mod gui;
mod subscriptions;
mod youtube_feed;

use crate::gui::Win;

use relm::Widget;

#[tokio::main]
async fn main() {
    Win::run(()).unwrap();
}
