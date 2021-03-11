mod central_ui;
mod clue_ui;
mod controls_ui;
mod puzzle_backend;
mod puzzle_canvas;
mod puzzle_file_io;

use iced::{Application, Settings};

fn main() -> iced::Result {
    central_ui::CrosserUI::run(Settings::default())
}
