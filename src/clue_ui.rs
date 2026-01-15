use iced::{Center, Element, Length};
use iced::widget::{button, row, scrollable, text, text_input};
use crate::central_ui;
use crate::puzzle_backend;

use std::rc::Rc;
use std::cell::RefCell;

pub struct CluesBrowser {
    pub backend: Rc<RefCell<puzzle_backend::Puzzle>>,
    pub a_clues: Vec<ClueEntry>,
    pub d_clues: Vec<ClueEntry>,
    pub being_modified: Option<(u32, puzzle_backend::EntryVariant)>,
}

impl CluesBrowser {
    pub fn new(backend: Rc<RefCell<puzzle_backend::Puzzle>>) -> Self {
        let mut c = CluesBrowser {
            backend,
            a_clues: Vec::new(),
            d_clues: Vec::new(),
            being_modified: None,
        };
        c.update_clues();
        c
    }

    pub fn update_clues(&mut self) {
        self.a_clues.clear();
        self.d_clues.clear();

        let a_entries = &self.backend.borrow().across_entries;
        for a in a_entries.iter() {
            self.a_clues.push(ClueEntry::new(a.label,a.variant,a.clue.clone()));
        }

        let d_entries = &self.backend.borrow().down_entries;
        for d in d_entries.iter() {
            self.d_clues.push(ClueEntry::new(d.label,d.variant,d.clue.clone()));
        }
    }

    pub fn set_being_modified(&mut self, label: u32, variant: puzzle_backend::EntryVariant) {
        if !self.backend.borrow().fill_only {
            match variant {
                puzzle_backend::EntryVariant::Across => {
                    let entry = self.a_clues.iter_mut().find(|x| x.label == label).unwrap();
                    entry.being_modified = true;
                }
                puzzle_backend::EntryVariant::Down => {
                    let entry = self.d_clues.iter_mut().find(|x| x.label == label).unwrap();
                    entry.being_modified = true;
                }
            }
            self.being_modified = Some((label,variant));
        }
    }

    pub fn unset_being_modified(&mut self) {
        if !self.backend.borrow().fill_only {
            let (l,v) = self.being_modified.unwrap();
            match v {
                puzzle_backend::EntryVariant::Across => {
                    let entry = self.a_clues.iter_mut().find(|x| x.label == l).unwrap();
                    entry.being_modified = false;
                    self.backend.borrow_mut().set_clue_text(l,v,entry.clue_cache.clone());
                }
                puzzle_backend::EntryVariant::Down => {
                    let entry = self.d_clues.iter_mut().find(|x| x.label == l).unwrap();
                    entry.being_modified = false;
                    self.backend.borrow_mut().set_clue_text(l,v,entry.clue_cache.clone());
                }
            }
            self.being_modified = None;
        }
    }

    pub fn set_clue_text(&mut self, text_val: String) {
        let (l,v) = self.being_modified.unwrap();
        match v {
            puzzle_backend::EntryVariant::Across => {
                let entry = self.a_clues.iter_mut().find(|x| x.label == l).unwrap();
                entry.clue_cache = text_val;
            }
            puzzle_backend::EntryVariant::Down => {
                let entry = self.d_clues.iter_mut().find(|x| x.label == l).unwrap();
                entry.clue_cache = text_val;
            }
        }
    }

    pub fn view(&self) -> Element<'_, central_ui::Message> {
        let a_content: Element<_> = self.a_clues.iter()
            .fold(
                iced::widget::column![].spacing(2),
                |col, entry| col.push(entry.view())
            )
            .into();

        let d_content: Element<_> = self.d_clues.iter()
            .fold(
                iced::widget::column![].spacing(2),
                |col, entry| col.push(entry.view())
            )
            .into();

        row![
            scrollable(a_content).width(Length::Fill),
            scrollable(d_content).width(Length::Fill)
        ]
        .width(Length::FillPortion(1))
        .spacing(5)
        .align_y(Center)
        .into()
    }
}

pub struct ClueEntry {
    pub label: u32,
    pub clue_cache: String,
    pub variant: puzzle_backend::EntryVariant,
    pub being_modified: bool,
}

impl ClueEntry {
    pub fn new(label: u32, variant: puzzle_backend::EntryVariant, clue: String) -> Self {
        ClueEntry {
            label,
            clue_cache: clue,
            variant,
            being_modified: false,
        }
    }

    pub fn view(&self) -> Element<'_, central_ui::Message> {
        let a_or_d = match self.variant {
            puzzle_backend::EntryVariant::Across => 'A',
            puzzle_backend::EntryVariant::Down => 'D',
        };

        let mut prefix = self.label.to_string();
        prefix.push(a_or_d);
        prefix.push_str(": ");

        let t = text(prefix.clone());

        if self.being_modified {
            row![
                button(t)
                    .on_press(central_ui::Message::ClueLeftModification(self.label, self.variant))
                    .width(60),
                text_input("", &self.clue_cache)
                    .on_input(central_ui::Message::ClueModified)
                    .on_submit(central_ui::Message::ClueLeftModification(self.label, self.variant))
            ]
            .spacing(10)
            .align_y(Center)
            .into()
        } else {
            row![
                button(t)
                    .on_press(central_ui::Message::ClueEnteredModification(self.label, self.variant))
                    .width(60),
                text(self.clue_cache.clone())
            ]
            .spacing(10)
            .align_y(Center)
            .into()
        }
    }
}
