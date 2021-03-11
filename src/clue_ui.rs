use iced::{Align, button, Button, Element, Length, Row, scrollable, Scrollable, Text, text_input, TextInput};
use crate::central_ui;
use crate::puzzle_backend;

use std::rc::Rc;
use std::cell::RefCell;

pub struct CluesBrowser {
    pub backend: Rc<RefCell<puzzle_backend::Puzzle>>,
    pub a_clues: Vec<ClueEntry>,
    pub a_scroller: scrollable::State,
    pub d_clues: Vec<ClueEntry>,
    pub d_scroller: scrollable::State,
    pub being_modified: Option<(u32, puzzle_backend::EntryVariant)>,
}

impl CluesBrowser {
    pub fn new(backend: Rc<RefCell<puzzle_backend::Puzzle>>) -> Self {
        let mut c = CluesBrowser {
            backend,
            a_clues: Vec::new(),
            a_scroller: Default::default(),
            d_clues: Vec::new(),
            d_scroller: Default::default(),
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

    pub fn unset_being_modified(&mut self) {
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

    pub fn set_clue_text(&mut self, text: String) {
        let (l,v) = self.being_modified.unwrap();
        match v {
            puzzle_backend::EntryVariant::Across => {
                let entry = self.a_clues.iter_mut().find(|x| x.label == l).unwrap();
                entry.clue_cache = text;
            }
            puzzle_backend::EntryVariant::Down => {
                let entry = self.d_clues.iter_mut().find(|x| x.label == l).unwrap();
                entry.clue_cache = text;
            }
        }
    }

    pub fn view (&mut self) -> Element<central_ui::Message> {
        Row::new()
        .push(
        self.a_clues.iter_mut().fold(
        Scrollable::new(&mut self.a_scroller)
        .width(Length::from(200))
        , |sc, x| sc.push(x.view()))
        )
        .push(
        self.d_clues.iter_mut().fold(
        Scrollable::new(&mut self.d_scroller)
        .width(Length::from(200))
        , |sc, x| sc.push(x.view()))
        )
        .into()
    }
}

pub struct ClueEntry {
    pub button: button::State,
    pub input: text_input::State,
    pub label: u32,
    pub clue_cache: String,
    pub variant: puzzle_backend::EntryVariant,
    pub being_modified: bool,
}

impl ClueEntry {
    pub fn new(label: u32, variant: puzzle_backend::EntryVariant, clue: String) -> Self {
        ClueEntry {
            button: Default::default(),
            input: Default::default(),
            label,
            clue_cache: clue,
            variant,
            being_modified: false,
        }
    }

    pub fn view(&mut self) -> Element<central_ui::Message> {
        let a_or_d = match self.variant {
            puzzle_backend::EntryVariant::Across => 'A',
            puzzle_backend::EntryVariant::Down => 'D',
        };

        let mut prefix = self.label.to_string();
        prefix.push(a_or_d);
        prefix.push_str(&": ".to_string());

        let t = Text::new(prefix);

        if self.being_modified {
            Row::new()
            .spacing(10)
            .align_items(Align::Center)
            .push(
                t
            )
            .push(
                TextInput::new(&mut self.input, "", &self.clue_cache, central_ui::Message::ClueModified)
                .on_submit(central_ui::Message::ClueLeftModification(self.label,self.variant))
            )
            .into()
        } else {
            Row::new()
            .spacing(10)
            .align_items(Align::Center)
            .push(
                Button::new(&mut self.button, t).on_press(central_ui::Message::ClueEnteredModification(self.label,self.variant))
            )
            .push(
                Text::new(self.clue_cache.clone())
            )
            .into()
        }
    }
}