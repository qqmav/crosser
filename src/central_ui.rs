use iced::{Align, Canvas, Column, Element, Length, Row, Sandbox, Text};
use crate::puzzle_backend;
use crate::puzzle_canvas;

pub struct CrosserUI {
    puzzle_ui: puzzle_canvas::PuzzleCanvas,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
}

impl Sandbox for CrosserUI {
    type Message = Message; 

    fn new() -> Self {
        CrosserUI { 
                    puzzle_ui: puzzle_canvas::PuzzleCanvas::new(puzzle_backend::PuzzleType::Weekday),
                  }
    }

    fn title(&self) -> String {
        String::from("Crosser -- The Friendly Crossword Puzzle Templating App")
    }

    fn view(&mut self) -> Element<Message> {
        Row::new()
        .padding(20)
        .align_items(Align::Center)
        .push(
            Canvas::new(&mut self.puzzle_ui)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .push(
            Text::new(String::from("test widget"))
        )
        .into()
    }

    fn update(&mut self, _message: Message) {
    }
}

