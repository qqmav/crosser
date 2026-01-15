mod central_ui;
mod clue_ui;
mod controls_ui;
mod puzzle_backend;
mod puzzle_canvas;
mod puzzle_file_io;

fn main() -> iced::Result {
    iced::application(central_ui::CrosserUI::default, central_ui::CrosserUI::update, central_ui::CrosserUI::view)
        .title("Crosser -- The Friendly Crossword Puzzle Templating App")
        .run()
}
