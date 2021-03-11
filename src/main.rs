mod central_ui;
mod puzzle_backend;
mod puzzle_canvas;
mod clue_ui;

use iced::{Application, Settings};

fn main() -> iced::Result {
    central_ui::CrosserUI::run(Settings::default())
}
