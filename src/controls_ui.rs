use iced::{Center, Element};
use iced::widget::{button, checkbox, row, text, text_input};
use crate::central_ui;
use crate::puzzle_backend;

#[derive(Debug, Clone)]
pub enum State {
    Main,
    New,
    Save,
    Open,
    OperationResult(String),
}

pub struct ControlsRow {
    state: State,
    pub save_path_string: String,
    pub save_empty_grid: bool,
    pub open_path_string: String,
}

impl ControlsRow {
    pub fn new() -> Self {
        ControlsRow {
            state: State::Main,
            save_path_string: std::env::current_dir().unwrap().to_str().unwrap().to_string(),
            save_empty_grid: false,
            open_path_string: std::env::current_dir().unwrap().to_str().unwrap().to_string(),
        }
    }

    pub fn set_state(&mut self, s: State) {
        self.state = s;
    }

    pub fn view(&self) -> Element<'_, central_ui::Message> {
        match &self.state {
            State::Main => {
                row![
                    button("New ...").on_press(central_ui::Message::ControlSetState(State::New)),
                    button("Save ...").on_press(central_ui::Message::ControlSetState(State::Save)),
                    button("Open ...").on_press(central_ui::Message::ControlSetState(State::Open))
                ]
                .spacing(10)
                .into()
            }
            State::New => {
                row![
                    button("Back").on_press(central_ui::Message::ControlSetState(State::Main)),
                    button("New Mini").on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::Mini)),
                    button("New Weekday").on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::Weekday)),
                    button("New Weekday (Asymmetric)").on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::WeekdayAsymmetric)),
                    button("New Sunday").on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::Sunday))
                ]
                .spacing(10)
                .into()
            }
            State::Save => {
                row![
                    button("Back").on_press(central_ui::Message::ControlSetState(State::Main)),
                    text("Save to file (*.cro): "),
                    text_input("Save file path...", &self.save_path_string)
                        .on_input(central_ui::Message::SavePathModified)
                        .on_submit(central_ui::Message::AttemptSave),
                    checkbox(self.save_empty_grid)
                        .label("Save final solvable grid?")
                        .on_toggle(central_ui::Message::SaveEmptyGrid),
                    button("Save").on_press(central_ui::Message::AttemptSave)
                ]
                .spacing(10)
                .align_y(Center)
                .into()
            }
            State::Open => {
                row![
                    button("Back").on_press(central_ui::Message::ControlSetState(State::Main)),
                    text("Open file (*.cro): "),
                    text_input("Open file path...", &self.open_path_string)
                        .on_input(central_ui::Message::OpenPathModified)
                        .on_submit(central_ui::Message::AttemptOpen),
                    button("Open").on_press(central_ui::Message::AttemptOpen)
                ]
                .spacing(10)
                .align_y(Center)
                .into()
            }
            State::OperationResult(s) => {
                row![
                    button("OK").on_press(central_ui::Message::ControlSetState(State::Main)),
                    text(s)
                ]
                .spacing(10)
                .align_y(Center)
                .into()
            }
        }
    }
}
