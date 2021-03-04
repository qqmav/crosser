mod central_ui;
mod puzzle;

use iced::{Application, Settings};

fn main() -> iced::Result {
    central_ui::CrosserUI::run(Settings::default())
}
