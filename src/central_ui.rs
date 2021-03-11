use iced::{Align, Canvas, Element, Length, Row, Sandbox};
use crate::puzzle_backend;
use crate::puzzle_canvas;
use crate::clue_ui;

use std::rc::Rc;
use std::cell::RefCell;

pub struct CrosserUI {
    puzzle_ui: puzzle_canvas::PuzzleCanvas,
    clues: clue_ui::CluesBrowser,
}

#[derive(Debug, Clone)]
pub enum Message {
    ClueEnteredModification(u32,puzzle_backend::EntryVariant),
    ClueModified(String),
    ClueLeftModification(u32,puzzle_backend::EntryVariant),
    CluesUpdated,
}

impl Sandbox for CrosserUI {
    type Message = Message; 

    fn new() -> Self {
        let t = puzzle_backend::PuzzleType::Mini;
        let p = Rc::new(RefCell::new(puzzle_backend::Puzzle::new(t)));
        let u = puzzle_canvas::PuzzleCanvas::new(p.clone());
        let mut c = clue_ui::CluesBrowser::new(p.clone());
        c.update_clues();
        CrosserUI { 
                puzzle_ui: u,
                clues: c,
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
            self.clues.view()
        )
        .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ClueEnteredModification(l,v) => {
                self.puzzle_ui.set_ignore_keystrokes(true);
                self.clues.set_being_modified(l, v);
            }
            Message::ClueLeftModification(_l,_v) => {
                self.puzzle_ui.set_ignore_keystrokes(false);
                self.clues.unset_being_modified();
            }
            Message::ClueModified(s) => {
                // Cache text to prevent mut issues
                self.clues.set_clue_text(s);
            }
            Message::CluesUpdated => {
                self.clues.update_clues();
            }
        }
    }
}

