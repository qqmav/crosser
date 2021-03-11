use iced::{button, Button, Element, Row, Text};
use crate::central_ui;
use crate::puzzle_backend;

pub enum State {
    Main,
    New,
}

pub struct ControlsRow {
    new_but: button::State,
    mini_but: button::State,
    weekday_but: button::State,
    weekday_asym_but: button::State,
    sunday_but: button::State,
    back_but: button::State,
    state: State,
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
        }
    }

    pub fn set_state(&mut self, s: State) {
        self.state = s;
    }

    pub fn view(&mut self) -> Element<central_ui::Message> {
        match self.state {
            State::Main => {
                Row::new()
                .spacing(10)
                .push(
                    Button::new(&mut self.new_but, Text::new("New ...")).on_press(central_ui::Message::NewButtonClicked)
                )
                .into()
            }
            State::New => {
                Row::new()
                .spacing(10)
                .push(
                    Button::new(&mut self.back_but, Text::new("Back")).on_press(central_ui::Message::BackButtonClicked)
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
        }
    }
}