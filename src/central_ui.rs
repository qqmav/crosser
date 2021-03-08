use iced::{Align, Canvas, Element, Length, Row, Sandbox, Text};
use crate::puzzle_backend;
use crate::puzzle_canvas;

use std::rc::Rc;
use std::cell::RefCell;

pub struct CrosserUI {
    backend: Rc<RefCell<puzzle_backend::Puzzle>>,
    puzzle_ui: puzzle_canvas::PuzzleCanvas,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
}

impl Sandbox for CrosserUI {
    type Message = Message; 

    fn new() -> Self {
        let t = puzzle_backend::PuzzleType::Mini;
        let p = Rc::new(RefCell::new(puzzle_backend::Puzzle::new(t)));
        let u = puzzle_canvas::PuzzleCanvas::new(p.clone());
        CrosserUI { 
                backend: p,
                puzzle_ui: u,
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
        .into()
    }

    fn update(&mut self, _message: Message) {
    }
}

