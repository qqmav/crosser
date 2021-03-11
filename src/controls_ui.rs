use iced::{Align, button, Button, Checkbox, Element, Row, Text, text_input, TextInput, VerticalAlignment};
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
    new_but: button::State,
    mini_but: button::State,
    weekday_but: button::State,
    weekday_asym_but: button::State,
    sunday_but: button::State,
    back_but: button::State,
    state: State,
    save_but: button::State,
    save_field: text_input::State,
    pub save_path_string: String,
    pub save_empty_grid: bool,
    open_but: button::State,
    open_field: text_input::State,
    pub open_path_string: String,
}

impl ControlsRow {
    pub fn new() -> Self {
        ControlsRow {
            new_but: Default::default(),
            mini_but: Default::default(),
            weekday_but: Default::default(),
            weekday_asym_but: Default::default(),
            sunday_but: Default::default(),
            back_but: Default::default(),
            state: State::Main,
            save_but: Default::default(),
            save_field: Default::default(),
            save_path_string: std::env::current_dir().unwrap().to_str().unwrap().to_string(),
            save_empty_grid: false,
            open_but: Default::default(),
            open_field: Default::default(),
            open_path_string: std::env::current_dir().unwrap().to_str().unwrap().to_string(),
        }
    }

    pub fn set_state(&mut self, s: State) {
        self.state = s;
    }

    pub fn view(&mut self) -> Element<central_ui::Message> {
        match &self.state {
            State::Main => {
                Row::new()
                .spacing(10)
                .push(
                    Button::new(&mut self.new_but, Text::new("New ...")).on_press(central_ui::Message::ControlSetState(State::New))
                )
                .push(
                    Button::new(&mut self.save_but, Text::new("Save ...")).on_press(central_ui::Message::ControlSetState(State::Save))
                )
                .push(
                    Button::new(&mut self.open_but, Text::new("Open ...")).on_press(central_ui::Message::ControlSetState(State::Open))
                )
                .into()
            }
            State::New => {
                Row::new()
                .spacing(10)
                .push(
                    Button::new(&mut self.back_but, Text::new("Back")).on_press(central_ui::Message::ControlSetState(State::Main))
                )
                .push(
                    Button::new(&mut self.mini_but, Text::new("New Mini")).on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::Mini))
                )
                .push(
                    Button::new(&mut self.weekday_but, Text::new("New Weekday")).on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::Weekday))
                )
                .push(
                    Button::new(&mut self.weekday_asym_but, Text::new("New Weekday (Asymmetric)")).on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::WeekdayAsymmetric))
                )
                .push(
                    Button::new(&mut self.sunday_but, Text::new("New Sunday")).on_press(central_ui::Message::NewPuzzle(puzzle_backend::PuzzleType::Sunday))
                )
                .into()
            }
            State::Save => {
                Row::new()
                .spacing(10)
                .align_items(Align::Center)
                .push(
                    Button::new(&mut self.back_but, Text::new("Back")).on_press(central_ui::Message::ControlSetState(State::Main))
                )
                .push(
                    Text::new("Save to file (*.cro): ").vertical_alignment(VerticalAlignment::Center)
                )
                .push(
                    TextInput::new(&mut self.save_field, "Save file path..." , &self.save_path_string, central_ui::Message::SavePathModified)
                    .on_submit(central_ui::Message::AttemptSave)
                )
                .push(
                    Checkbox::new(
                        self.save_empty_grid,
                        "Strip grid content?".to_string(),
                        central_ui::Message::SaveEmptyGrid
                    )
                )
                .push(
                    Button::new(&mut self.save_but, Text::new("Save")).on_press(central_ui::Message::AttemptSave)
                )
                .into()
            }
            State::Open => {
                Row::new()
                .spacing(10)
                .align_items(Align::Center)
                .push(
                    Button::new(&mut self.back_but, Text::new("Back")).on_press(central_ui::Message::ControlSetState(State::Main))
                )
                .push(
                    Text::new("Open file (*.cro): ").vertical_alignment(VerticalAlignment::Center)
                )
                .push(
                    TextInput::new(&mut self.open_field, "Open file path..." , &self.open_path_string, central_ui::Message::OpenPathModified)
                    .on_submit(central_ui::Message::AttemptOpen)
                )
                .push(
                    Button::new(&mut self.open_but, Text::new("Open")).on_press(central_ui::Message::AttemptOpen)
                )
                .into()
            }
            State::OperationResult(s) => {
                Row::new()
                .spacing(10)
                .align_items(Align::Center)
                .push(
                    Button::new(&mut self.back_but, Text::new("OK")).on_press(central_ui::Message::ControlSetState(State::Main))
                )
                .push(
                    Text::new(s)
                )
                .into()
            }
        }
    }
}