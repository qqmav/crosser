use iced::{Center, Element, Length};
use iced::widget::{column, row, canvas};
use crate::puzzle_backend;
use crate::puzzle_canvas;
use crate::clue_ui;
use crate::controls_ui;
use crate::puzzle_file_io;

use std::rc::Rc;
use std::cell::RefCell;

pub struct CrosserUI {
    puzzle: Rc<RefCell<puzzle_backend::Puzzle>>,
    puzzle_ui: puzzle_canvas::PuzzleCanvas,
    clues: clue_ui::CluesBrowser,
    controls: controls_ui::ControlsRow,
}

#[derive(Debug, Clone)]
pub enum Message {
    ClueEnteredModification(u32,puzzle_backend::EntryVariant),
    ClueModified(String),
    ClueLeftModification(u32,puzzle_backend::EntryVariant),
    CluesUpdated,

    ControlSetState(controls_ui::State),

    NewPuzzle(puzzle_backend::PuzzleType),

    SavePathModified(String),
    SaveEmptyGrid(bool),
    AttemptSave,

    OpenPathModified(String),
    AttemptOpen,

    CanvasMessage(puzzle_canvas::CanvasMessage),
}

impl Default for CrosserUI {
    fn default() -> Self {
        let t = puzzle_backend::PuzzleType::Mini;
        let p = Rc::new(RefCell::new(puzzle_backend::Puzzle::new(t)));
        let puzzle_ui = puzzle_canvas::PuzzleCanvas::new(p.clone());
        let clues = clue_ui::CluesBrowser::new(p.clone());
        let controls = controls_ui::ControlsRow::new();
        CrosserUI {
            puzzle: p,
            puzzle_ui,
            clues,
            controls,
        }
    }
}

impl CrosserUI {
    pub fn view(&self) -> Element<'_, Message> {
        column![
            self.controls.view(),
            row![
                canvas(&self.puzzle_ui)
                    .height(Length::Fill)
                    .width(Length::FillPortion(2)),
                self.clues.view()
            ]
            .padding(20)
            .align_y(Center)
        ]
        .padding(10)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ClueEnteredModification(l,v) => {
                if !self.puzzle.borrow().fill_only {
                    if self.clues.being_modified.is_none() {
                        self.puzzle_ui.set_ignore_keystrokes(true);
                        self.clues.set_being_modified(l, v);
                    }
                }
            }
            Message::ClueLeftModification(_l,_v) => {
                if !self.puzzle.borrow().fill_only {
                    self.puzzle_ui.set_ignore_keystrokes(false);
                    self.clues.unset_being_modified();
                }
            }
            Message::ClueModified(s) => {
                // Cache text to prevent mut issues
                self.clues.set_clue_text(s);
            }
            Message::CluesUpdated => {
                self.clues.update_clues();
            }
            Message::ControlSetState(s) => {
                self.puzzle_ui.set_no_selected_square();
                self.controls.set_state(s);
            }
            Message::NewPuzzle(v) => {
                self.puzzle = Rc::new(RefCell::new(puzzle_backend::Puzzle::new(v)));
                self.puzzle_ui = puzzle_canvas::PuzzleCanvas::new(self.puzzle.clone());
                self.clues = clue_ui::CluesBrowser::new(self.puzzle.clone());
                self.controls = controls_ui::ControlsRow::new();
            }
            Message::SavePathModified(s) => {
                self.controls.save_path_string = s;
            }
            Message::SaveEmptyGrid(b) => {
                self.controls.save_empty_grid = b;
            }
            Message::AttemptSave => {
                let res = puzzle_file_io::write_puzzle_to_cro(self.puzzle.clone(),self.controls.save_path_string.clone(),self.controls.save_empty_grid);
                match res {
                    Ok(()) => {
                        self.controls.set_state(controls_ui::State::OperationResult("File wrote successfully!".to_string()));
                    }
                    Err(e) => {
                        self.controls.set_state(controls_ui::State::OperationResult("Write unsuccessful: ".to_string() + &e.to_string()));
                    }
                }
            }
            Message::OpenPathModified(s) => {
                self.controls.open_path_string = s;
            }
            Message::AttemptOpen => {
                let res = puzzle_file_io::get_puzzle_from_cro(self.controls.open_path_string.clone());
                match res {
                    Ok(p) => {
                        self.puzzle = Rc::new(RefCell::new(p));
                        self.puzzle_ui = puzzle_canvas::PuzzleCanvas::new(self.puzzle.clone());
                        self.clues = clue_ui::CluesBrowser::new(self.puzzle.clone());
                        self.controls = controls_ui::ControlsRow::new();
                    }
                    Err(s) => {
                        self.controls.set_state(controls_ui::State::OperationResult("Open unsuccessful: ".to_string() + &s));
                    }
                }
            }
            Message::CanvasMessage(canvas_msg) => {
                // Handle messages from the canvas
                match self.puzzle_ui.handle_message(canvas_msg) {
                    Some(Message::CluesUpdated) => {
                        self.clues.update_clues();
                    }
                    _ => {}
                }
            }
        }
    }
}
