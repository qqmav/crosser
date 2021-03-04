use iced::{Column, Sandbox, Text, Element};
use crate::puzzle;

pub struct CrosserUI {
    puzzle: puzzle::Puzzle,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
}

impl Sandbox for CrosserUI {
    type Message = Message; 

    fn new() -> Self {
        CrosserUI { puzzle: puzzle::Puzzle::new(puzzle::PuzzleType::Mini) }
    }

    fn title(&self) -> String {
        String::from("Crosser -- The Friendly Crossword Puzzle Templating App")
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .push(
                Text::new("Test".to_string()).size(50),
            )
            .into()
    }

    fn update(&mut self, message: Message) {
    }
}

