use iced::{
    mouse, keyboard, Color, Point, Rectangle, Size, Renderer, Theme,
    alignment,
};
use iced::widget::canvas::{self, Path, Stroke, Text, Geometry};
use crate::central_ui;
use crate::puzzle_backend;

struct SquareUIInfo {
    x: u32,
    y: u32,
    center: Point,
    content_top_left_corner: Point,
}

struct GridUIInfo {
    min_size: f32,
    square_width: f32,
    content_width: f32,
    label_size: f32,
    frame_square_infos: Vec<SquareUIInfo>,
    clue_start: Point,
    clue_height: f32,
    clue_width: f32,
    clue_padding: f32,
}

impl GridUIInfo {
    fn new(bounds: &Rectangle, dim: u32) -> Self {
        let (canv_w, canv_h) = (bounds.size().width, bounds.size().height);
        let min_size = match canv_w < canv_h {
            true => canv_w,
            false  => canv_h,
        };
        const SCALING_FACTOR: f32 = 0.85;
        let square_width = SCALING_FACTOR * (min_size / dim as f32);

        const PADDING_FACTOR: f32 = 0.05;
        let padding_width = PADDING_FACTOR * square_width;
        let content_width = square_width - padding_width;

        const LABEL_FACTOR: f32 = 0.30;
        let label_size = LABEL_FACTOR * square_width;
        let frame_square_infos = GridUIInfo::get_frame_square_infos(square_width, content_width, dim);

        let clue_start = Point::new(0.0, SCALING_FACTOR * min_size);
        let clue_height = (1.0 - SCALING_FACTOR) * min_size;
        let clue_width = SCALING_FACTOR * min_size;
        const CLUE_PADDING_FACTOR: f32 = 0.01;
        let clue_padding = CLUE_PADDING_FACTOR * clue_height;

        GridUIInfo {
            min_size,
            square_width,
            content_width,
            label_size,
            frame_square_infos,
            clue_start,
            clue_height,
            clue_width,
            clue_padding,
        }
    }

    fn get_frame_square_infos(sq_w: f32, c_w: f32, dim: u32) -> Vec<SquareUIInfo>{
        let mut frame_square_infos: Vec<SquareUIInfo> = Vec::with_capacity(dim as usize * dim as usize);
        // Column-major
        for x in 0..dim {
            for y in 0..dim {
                let p = Point {
                    x: sq_w * ((x as f32) + 0.5),
                    y: sq_w * ((y as f32) + 0.5),
                };
                let c = Point {
                    x: p.x - c_w / 2.0,
                    y: p.y - c_w / 2.0,
                };
                frame_square_infos.push(SquareUIInfo {
                    x,
                    y,
                    center: p,
                    content_top_left_corner: c,
                });
            }
        }

        frame_square_infos
    }

    fn update(&mut self, bounds: &Rectangle, dim: u32) -> bool {
        // If the square size hasn't changed, then nothing else has either.
        let (canv_w, canv_h) = (bounds.size().width, bounds.size().height);
        let min_size = match canv_w < canv_h {
            true => canv_w,
            false  => canv_h,
        };
        if min_size != self.min_size {
            let new_g_ui_info = GridUIInfo::new(bounds, dim);
            *self = new_g_ui_info;
            true
        } else {
            false
        }
    }
}

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    CluesUpdated,
}

pub struct PuzzleCanvas {
    backend: Rc<RefCell<puzzle_backend::Puzzle>>,
    dim: u32,
    grid_info: RefCell<GridUIInfo>,
    cursor_pos: RefCell<Point>,
    ignore_keystrokes: RefCell<bool>,
    lctrl_held: RefCell<bool>,
    rctrl_held: RefCell<bool>,
    hovered_square: RefCell<Option<(u32, u32)>>,
    selected_square: RefCell<Option<(u32, u32)>>,
    selected_variant: RefCell<puzzle_backend::EntryVariant>,
    grid_cache: canvas::Cache,
    label_cache: canvas::Cache,
    content_cache: canvas::Cache,
    modifier_cache: canvas::Cache,
    highlighter_cache: canvas::Cache,
    clues_cache: canvas::Cache,
    solved_highlight_cache: canvas::Cache,
}

impl PuzzleCanvas {
    pub fn new(backend: Rc<RefCell<puzzle_backend::Puzzle>>) -> PuzzleCanvas {
        let d = puzzle_backend::match_puzzle_dim(&backend.borrow().variant);
        PuzzleCanvas {
            backend,
            grid_info: RefCell::new(GridUIInfo::new(&Rectangle::with_size(Size::new(1.0,1.0)), d as u32)),
            dim: d as u32,
            cursor_pos: RefCell::new(Point::new(0.0,0.0)),
            ignore_keystrokes: RefCell::new(false),
            lctrl_held: RefCell::new(false),
            rctrl_held: RefCell::new(false),
            hovered_square: RefCell::new(None),
            selected_square: RefCell::new(None),
            selected_variant: RefCell::new(puzzle_backend::EntryVariant::Across),
            grid_cache: canvas::Cache::default(),
            label_cache: canvas::Cache::default(),
            content_cache: canvas::Cache::default(),
            modifier_cache: canvas::Cache::default(),
            highlighter_cache: canvas::Cache::default(),
            clues_cache: canvas::Cache::default(),
            solved_highlight_cache: canvas::Cache::default(),
        }
    }

    pub fn set_ignore_keystrokes(&self, val: bool) {
        *self.selected_square.borrow_mut() = None;
        *self.ignore_keystrokes.borrow_mut() = val;
    }

    pub fn set_no_selected_square(&self) {
        *self.selected_square.borrow_mut() = None;
    }

    pub fn handle_message(&self, _msg: CanvasMessage) -> Option<central_ui::Message> {
        Some(central_ui::Message::CluesUpdated)
    }

    fn clear_all_caches(&self) {
        self.grid_cache.clear();
        self.label_cache.clear();
        self.content_cache.clear();
        self.modifier_cache.clear();
        self.highlighter_cache.clear();
        self.clues_cache.clear();
        self.solved_highlight_cache.clear();
    }
}

fn project_cursor_into_square(cursor_pos: &Point, sq_width: &f32, grid_dim: &u32) -> Option<(u32,u32)> {
    let t_x: i64 = (cursor_pos.x / sq_width).floor() as i64;
    let t_y: i64 = (cursor_pos.y / sq_width).floor() as i64;
    match ((t_x < (*grid_dim).into()) && (t_x >= 0)) && ((t_y < (*grid_dim).into()) && (t_y >= 0)) {
        true => Some((t_x as u32,t_y as u32)),
        false => None,
    }
}

type Message = central_ui::Message;

impl canvas::Program<Message> for PuzzleCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let mut ui_updated = self.grid_info.borrow_mut().update(&bounds, self.dim);
        let mut m: Option<Message> = None;
        let mut captured = true;

        match event {
            iced::Event::Mouse(mouse_event) => {
                if cursor.position_in(bounds).is_none() {
                    return None;
                }
                match mouse_event {
                    mouse::Event::CursorMoved { .. } => {
                        if let Some(position) = cursor.position_in(bounds) {
                            *self.cursor_pos.borrow_mut() = position;
                            let new_sq = project_cursor_into_square(&position, &self.grid_info.borrow().square_width, &self.dim);
                            if *self.hovered_square.borrow() != new_sq {
                                *self.hovered_square.borrow_mut() = new_sq;
                                self.highlighter_cache.clear();
                                self.clues_cache.clear();
                            }
                        } else {
                            captured = false;
                        }
                    },
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if *self.lctrl_held.borrow() || *self.rctrl_held.borrow() {
                            if let Some((tx,ty)) = *self.hovered_square.borrow() {
                                let did_modify_sq = self.backend.borrow_mut().cycle_modifier(tx,ty);
                                ui_updated = did_modify_sq;
                            }
                        } else {
                            let hovered = *self.hovered_square.borrow();
                            let selected = *self.selected_square.borrow();
                            if hovered.is_none() {
                                captured = false;
                            } else {
                                if hovered == selected {
                                    let current = *self.selected_variant.borrow();
                                    *self.selected_variant.borrow_mut() = match current {
                                        puzzle_backend::EntryVariant::Across => puzzle_backend::EntryVariant::Down,
                                        puzzle_backend::EntryVariant::Down => puzzle_backend::EntryVariant::Across,
                                    };
                                } else {
                                    // Can unwrap because we have already handled None case.
                                    let (hx,hy) = hovered.unwrap();
                                    match self.backend.borrow().at(hx,hy).content {
                                        puzzle_backend::SquareContents::TextContent(_,_) => {
                                            *self.selected_square.borrow_mut() = hovered;
                                            ui_updated = true;
                                        }
                                        _ => {}
                                    }
                                }
                                self.clues_cache.clear();
                                self.highlighter_cache.clear();
                            }
                        }
                    },
                    mouse::Event::ButtonPressed(mouse::Button::Right) => {
                        ui_updated = true;
                        let selected = *self.selected_square.borrow();
                        match selected {
                            Some((_sx,_sy)) => {
                                *self.selected_square.borrow_mut() = None;
                                ui_updated = true;
                            },
                            None => {
                                if let Some((tx,ty)) = *self.hovered_square.borrow() {
                                    self.backend.borrow_mut().cycle_blocker(tx,ty,false);
                                    ui_updated = true;
                                    m = Some(Message::CluesUpdated);
                                } else {
                                    captured = false;
                                }
                            },
                        }
                    },
                    _ => ()
                }
            }
            iced::Event::Keyboard(keyboard_event) => {
                if !*self.ignore_keystrokes.borrow() {
                    match keyboard_event {
                        keyboard::Event::KeyPressed { key, modifiers, .. }  => {
                            match &key {
                                keyboard::Key::Named(named) => {
                                    match named {
                                        keyboard::key::Named::Escape => {
                                            *self.selected_square.borrow_mut() = None;
                                            self.highlighter_cache.clear();
                                        },
                                        keyboard::key::Named::Control => {
                                            *self.lctrl_held.borrow_mut() = true;
                                        },
                                        keyboard::key::Named::Backspace => {
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                let square_was_empty = self.backend.borrow_mut().clear_sq_contents(tx,ty);

                                                ui_updated = true;

                                                if square_was_empty {
                                                    let prev = match *self.selected_variant.borrow() {
                                                        puzzle_backend::EntryVariant::Across => {
                                                            self.backend.borrow().at(tx,ty).prev_across
                                                        },
                                                        puzzle_backend::EntryVariant::Down => {
                                                            self.backend.borrow().at(tx,ty).prev_down
                                                        },
                                                    };

                                                    if let Some(s) = prev {
                                                        let prev_sq = self.backend.borrow().squares[s].clone();
                                                        let (px,py) = (prev_sq.x,prev_sq.y);
                                                        *self.selected_square.borrow_mut() = Some((px,py));
                                                        self.backend.borrow_mut().clear_sq_contents(px,py);
                                                    };
                                                }
                                            }
                                        },
                                        keyboard::key::Named::Delete => {
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                self.backend.borrow_mut().clear_sq_contents(tx,ty);
                                                ui_updated = true;
                                            }
                                        },
                                        keyboard::key::Named::ArrowUp => {
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                match *self.selected_variant.borrow() {
                                                    puzzle_backend::EntryVariant::Down => {
                                                        if ty > 0 {
                                                            *self.selected_square.borrow_mut() = Some((tx,ty-1));
                                                            ui_updated = true;
                                                        }
                                                    },
                                                    puzzle_backend::EntryVariant::Across => {
                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Down;
                                                        ui_updated = true;
                                                    },
                                                };
                                            }
                                        },
                                        keyboard::key::Named::ArrowDown => {
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                match *self.selected_variant.borrow() {
                                                    puzzle_backend::EntryVariant::Down => {
                                                        if ty < self.dim - 1 {
                                                            *self.selected_square.borrow_mut() = Some((tx,ty+1));
                                                            ui_updated = true;
                                                        }
                                                    },
                                                    puzzle_backend::EntryVariant::Across => {
                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Down;
                                                        ui_updated = true;
                                                    },
                                                };
                                            }
                                        },
                                        keyboard::key::Named::ArrowLeft => {
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                match *self.selected_variant.borrow() {
                                                    puzzle_backend::EntryVariant::Across => {
                                                        if tx > 0 {
                                                            *self.selected_square.borrow_mut() = Some((tx-1,ty));
                                                            ui_updated = true;
                                                        }
                                                    },
                                                    puzzle_backend::EntryVariant::Down => {
                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Across;
                                                        ui_updated = true;
                                                    },
                                                };
                                            }
                                        },
                                        keyboard::key::Named::ArrowRight => {
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                match *self.selected_variant.borrow() {
                                                    puzzle_backend::EntryVariant::Across => {
                                                        if tx < self.dim - 1 {
                                                            *self.selected_square.borrow_mut() = Some((tx+1,ty));
                                                            ui_updated = true;
                                                        }
                                                    },
                                                    puzzle_backend::EntryVariant::Down => {
                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Across;
                                                        ui_updated = true;
                                                    },
                                                };
                                            }
                                        },
                                        keyboard::key::Named::Tab => {
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                let (next_var,next_clue_index) = match *self.selected_variant.borrow() {
                                                    puzzle_backend::EntryVariant::Across => {
                                                        let a_entries = &self.backend.borrow().across_entries;
                                                        let a_entry = self.backend.borrow().at(tx,ty).across_entry;
                                                        if let Some(num) = a_entry {
                                                            let a_index = a_entries.iter().position(|x| x.label == num);
                                                            if let Some(a) = a_index {
                                                                if !modifiers.shift() {
                                                                    if a < a_entries.len() - 1 {
                                                                        (Some(puzzle_backend::EntryVariant::Across), Some(a + 1))
                                                                    } else {
                                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Down;
                                                                        (Some(puzzle_backend::EntryVariant::Down),Some(0))
                                                                    }
                                                                } else {
                                                                    if a > 0 {
                                                                        (Some(puzzle_backend::EntryVariant::Across),Some(a - 1))
                                                                    } else {
                                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Down;
                                                                        (Some(puzzle_backend::EntryVariant::Down),Some(self.backend.borrow().down_entries.len() - 1))
                                                                    }
                                                                }
                                                            } else {
                                                                (None,None)
                                                            }
                                                        } else {
                                                            (None,None)
                                                        }
                                                    },
                                                    puzzle_backend::EntryVariant::Down => {
                                                        let d_entries = &self.backend.borrow().down_entries;
                                                        let d_entry = self.backend.borrow().at(tx,ty).down_entry;
                                                        if let Some(num) = d_entry {
                                                            let d_index = d_entries.iter().position(|x| x.label == num);
                                                            if let Some(d) = d_index {
                                                                if !modifiers.shift() {
                                                                    if d < d_entries.len() - 1 {
                                                                        (Some(puzzle_backend::EntryVariant::Down),Some(d + 1))
                                                                    } else {
                                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Across;
                                                                        (Some(puzzle_backend::EntryVariant::Across),Some(0))
                                                                    }
                                                                } else {
                                                                    if d > 0 {
                                                                        (Some(puzzle_backend::EntryVariant::Down),Some(d - 1))
                                                                    } else {
                                                                        *self.selected_variant.borrow_mut() = puzzle_backend::EntryVariant::Across;
                                                                        (Some(puzzle_backend::EntryVariant::Across),Some(self.backend.borrow().across_entries.len()-1))
                                                                    }
                                                                }
                                                            } else {
                                                                (None,None)
                                                            }
                                                        } else {
                                                        (None, None)
                                                    }
                                                    },
                                                };
                                                if let Some(i) = next_clue_index {
                                                    if let Some(v) = next_var {
                                                        match v {
                                                            puzzle_backend::EntryVariant::Across => {
                                                                let sq = &self.backend.borrow().squares[self.backend.borrow().across_entries[i].member_indices[0]];
                                                                *self.selected_square.borrow_mut() = Some((sq.x,sq.y));
                                                            },
                                                            puzzle_backend::EntryVariant::Down => {
                                                                let sq = &self.backend.borrow().squares[self.backend.borrow().down_entries[i].member_indices[0]];
                                                                *self.selected_square.borrow_mut() = Some((sq.x,sq.y));
                                                            }
                                                        }
                                                    };
                                                };
                                                ui_updated = true;
                                            }
                                        },
                                        keyboard::key::Named::Space => {
                                            if let Some ((tx,ty)) = *self.selected_square.borrow() {
                                                self.backend.borrow_mut().clear_sq_contents(tx,ty);
                                                let next = match *self.selected_variant.borrow() {
                                                    puzzle_backend::EntryVariant::Across => {
                                                        self.backend.borrow().at(tx,ty).next_across
                                                    },
                                                    puzzle_backend::EntryVariant::Down => {
                                                        self.backend.borrow().at(tx,ty).next_down
                                                    },
                                                };
                                                if let Some(s) = next {
                                                    let next_sq = &self.backend.borrow().squares[s];
                                                    *self.selected_square.borrow_mut() = Some((next_sq.x,next_sq.y));
                                                };
                                                ui_updated = true;
                                            }
                                        },
                                        _ => {
                                            captured = false;
                                        },
                                    }
                                },
                                keyboard::Key::Character(c) => {
                                    if let Some(ch) = c.chars().next() {
                                        if ch.is_ascii_alphabetic() {
                                            let upper_ch = ch.to_ascii_uppercase();
                                            if let Some((tx,ty)) = *self.selected_square.borrow() {
                                                self.backend.borrow_mut().modify_sq_contents(tx,ty,upper_ch,modifiers.shift());
                                                ui_updated = true;
                                                if !modifiers.control() && !modifiers.shift() {
                                                    // If ctrl isnt held, move to next letter.
                                                    let next = match *self.selected_variant.borrow() {
                                                        puzzle_backend::EntryVariant::Across => {
                                                            self.backend.borrow().at(tx,ty).next_across
                                                        }
                                                        puzzle_backend::EntryVariant::Down => {
                                                            self.backend.borrow().at(tx,ty).next_down
                                                        }
                                                    };

                                                    if let Some(s) = next {
                                                        let next_sq = &self.backend.borrow().squares[s];
                                                        *self.selected_square.borrow_mut() = Some((next_sq.x,next_sq.y));
                                                    };
                                                }
                                            }
                                        } else {
                                            captured = false;
                                        }
                                    } else {
                                        captured = false;
                                    }
                                },
                                _ => {
                                    captured = false;
                                },
                            };
                        },
                        keyboard::Event::KeyReleased { key, .. }  => {
                            match &key {
                                keyboard::Key::Named(keyboard::key::Named::Control) => {
                                    *self.lctrl_held.borrow_mut() = false;
                                    *self.rctrl_held.borrow_mut() = false;
                                },
                                _ => {
                                    captured = false;
                                },
                            };
                        },
                        _ => (),
                    }
                }
            }
            _ => {}
        };

        if ui_updated {
            self.clear_all_caches();
        }

        if captured || m.is_some() {
            Some(canvas::Action::publish(m.unwrap_or(Message::CanvasMessage(CanvasMessage::CluesUpdated))))
        } else {
            None
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // We cant assign to ourselves, but if the bound doesn't match, then we need to make a new griduiinfo
        let (canv_w, canv_h) = (bounds.size().width, bounds.size().height);
        let min_size = match canv_w < canv_h {
            true => canv_w,
            false  => canv_h,
        };

        if min_size != self.grid_info.borrow().min_size {
            *self.grid_info.borrow_mut() = GridUIInfo::new(&bounds, self.dim);
        }

        let frame_grid_info = self.grid_info.borrow();

        let grid = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            let dark_bg = Path::rectangle(Point::new(0.0,0.0), Size::new(frame_grid_info.square_width * self.dim as f32,frame_grid_info.square_width * self.dim as f32));
            frame.fill(&dark_bg, Color::BLACK);
            for sq in &frame_grid_info.frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(_s,m) = &self.backend.borrow().at(sq.x, sq.y).content {
                    let sq_path = Path::rectangle(sq.content_top_left_corner,Size::new(frame_grid_info.content_width,frame_grid_info.content_width));
                    let color = match m {
                        Some(puzzle_backend::SquareModifier::Shading) => Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                        _ => Color::WHITE,
                    };
                    frame.fill(&sq_path, color);
                }
            };
        });

        let labels = self.label_cache.draw(renderer, bounds.size(), |frame| {
            for sq in &frame_grid_info.frame_square_infos {
                if let Some(l) = self.backend.borrow().at(sq.x,sq.y).label {
                    let text = Text {
                        color: Color::BLACK,
                        position: sq.content_top_left_corner,
                        size: frame_grid_info.label_size.into(),
                        content: l.to_string(),
                        ..Text::default()
                    };
                    frame.fill_text(text);
                };
            }
        });

        let content = self.content_cache.draw(renderer, bounds.size(), |frame| {
            for sq in &frame_grid_info.frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(s,_m) = &self.backend.borrow().at(sq.x,sq.y).content {
                    let num_chars = s.len();
                    let sq_text_size = match num_chars {
                        1 => frame_grid_info.content_width,
                        _ => 1.2 * frame_grid_info.content_width / num_chars as f32,
                    };

                    let text = Text {
                        color: Color::BLACK,
                        position: sq.center,
                        size: sq_text_size.into(),
                        content: s.clone(),
                        align_x: alignment::Horizontal::Center.into(),
                        align_y: alignment::Vertical::Center.into(),
                        ..Text::default()
                    };
                    frame.fill_text(text);
                };
            }
        });

        let modifiers = self.modifier_cache.draw(renderer, bounds.size(), |frame| {
            let stroke = Stroke {
                width: frame_grid_info.content_width * 0.03,
                ..Default::default()
            };
            for sq in &frame_grid_info.frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(_s,Some(puzzle_backend::SquareModifier::Circle)) = &self.backend.borrow().at(sq.x,sq.y).content {
                    frame.stroke(&Path::circle(sq.center, 0.92 * frame_grid_info.content_width / 2.0),stroke.clone());
                }
            }
        });

        let highlighter = self.highlighter_cache.draw(renderer, bounds.size(), |frame| {
            let selected = *self.selected_square.borrow();
            match selected {
                None => {
                    let cursor_pos = *self.cursor_pos.borrow();
                    if let Some((sx,sy)) = project_cursor_into_square(&cursor_pos, &frame_grid_info.square_width, &self.dim) {
                        let r_path = Path::rectangle(
                                frame_grid_info.frame_square_infos[sx as usize * self.dim as usize + sy as usize].content_top_left_corner,
                                Size::new(frame_grid_info.content_width,frame_grid_info.content_width)
                            );
                        let r_c = Color::from_rgba(0.0,0.0,1.0,0.2);
                        frame.fill(&r_path,r_c);
                        match self.backend.borrow().variant {
                            puzzle_backend::PuzzleType::Weekday | puzzle_backend::PuzzleType::Sunday => {
                                if (sx != self.dim / 2_u32) || (sy != self.dim / 2_u32) {
                                    let index = (self.dim - sx - 1) as usize * self.dim as usize + (self.dim - sy - 1) as usize;
                                    let s_path = Path::rectangle(
                                        frame_grid_info.frame_square_infos[index].content_top_left_corner,
                                        Size::new(frame_grid_info.content_width, frame_grid_info.content_width)
                                    );
                                    let s_c = Color::from_rgba(0.0, 0.0, 1.0, 0.2);
                                    frame.fill(&s_path,s_c);
                                }
                            },
                            _ => (),
                        }
                    }
                },
                Some((hx,hy)) => {
                    // Fill Selected with green
                    let r_path = Path::rectangle(
                        frame_grid_info.frame_square_infos[hx as usize * self.dim as usize + hy as usize].content_top_left_corner,
                        Size::new(frame_grid_info.content_width,frame_grid_info.content_width)
                    );
                    let r_c = Color::from_rgba(0.0,1.0,0.0,0.5);
                    frame.fill(&r_path,r_c);
                    // Fill rest of clue with yellow
                    let entries: Option<Vec<usize>> = if let (Some(a),Some(d)) = self.backend.borrow().get_clue_entries(hx,hy) {
                        match *self.selected_variant.borrow() {
                            puzzle_backend::EntryVariant::Across => {
                                Some(a.member_indices.clone())
                            },
                            puzzle_backend::EntryVariant::Down => {
                                Some(d.member_indices.clone())
                            },
                        }
                    } else {
                        None
                    };
                    match entries {
                        Some(v) => {
                            for sq_index in v {
                                let sq = &self.backend.borrow().squares[sq_index];
                                let r_path = Path::rectangle(
                                    frame_grid_info.frame_square_infos[(sq.x * self.dim + sq.y) as usize].content_top_left_corner,
                                    Size::new(frame_grid_info.content_width,frame_grid_info.content_width));
                                let r_c = Color::from_rgba(1.0,1.0,0.0,0.3);
                                frame.fill(&r_path,r_c);
                            }
                        },
                        None => {},
                    }
               },
            };
        });

        let clues = self.clues_cache.draw(renderer, bounds.size(), |frame| {
            let bg_r_path = Path::rectangle(frame_grid_info.clue_start,Size::new(frame_grid_info.clue_width,frame_grid_info.clue_height));
            let bg_r_color = Color::BLACK;
            frame.fill(&bg_r_path,bg_r_color);

            let content_size = Size::new(frame_grid_info.clue_width - 2.0 * frame_grid_info.clue_padding, 0.5 * (frame_grid_info.clue_height - 3.0 * frame_grid_info.clue_padding));
            let across_start = Point::new(frame_grid_info.clue_start.x + frame_grid_info.clue_padding, frame_grid_info.clue_start.y + frame_grid_info.clue_padding);
            let down_start = Point::new(frame_grid_info.clue_start.x + frame_grid_info.clue_padding, frame_grid_info.clue_start.y + 2.0 * frame_grid_info.clue_padding + content_size.height);

            let a_r_path = Path::rectangle(across_start, content_size);
            let d_r_path = Path::rectangle(down_start, content_size);

            frame.fill(&a_r_path,Color::WHITE);
            frame.fill(&d_r_path,Color::WHITE);

            let selected = *self.selected_square.borrow();
            let (a_c,d_c) = match selected {
                Some((_sx,_sy)) => {
                    match *self.selected_variant.borrow() {
                        puzzle_backend::EntryVariant::Across => (Color::from_rgba(1.0, 1.0, 0.0, 0.3),Color::from_rgba(0.0, 0.0, 1.0, 0.2)),
                        puzzle_backend::EntryVariant::Down => (Color::from_rgba(0.0, 0.0, 1.0, 0.2),Color::from_rgba(1.0, 1.0, 0.0, 0.3)),
                    }
                },
                None => {
                    (Color::from_rgba(0.0, 0.0, 1.0, 0.2),Color::from_rgba(0.0, 0.0, 1.0, 0.2))
                },
            };

            frame.fill(&a_r_path,a_c);
            frame.fill(&d_r_path,d_c);

            let hovered = *self.hovered_square.borrow();
            let (a_s,d_s) = match selected {
                Some((sx,sy)) => {
                    self.backend.borrow().get_square_clue_texts(sx,sy)
                },
                None => {
                    match hovered {
                        Some((hx,hy)) => {
                            self.backend.borrow().get_square_clue_texts(hx,hy)
                        },
                        None => {
                            ("".to_string(),"".to_string())
                        },
                    }
                },
            };

            let a_len = a_s.len();
            let a_size = if a_len < 50 {
                0.60 * content_size.height
            } else {
                0.60 * content_size.height * (45.0 / a_len as f32)
            };
            let a_text = Text {
                color: Color::BLACK,
                position: Point::new(across_start.x, across_start.y + 0.20 * content_size.height),
                size: a_size.into(),
                content: a_s,
                ..Text::default()
            };
            frame.fill_text(a_text);

            let d_len = d_s.len();
            let d_size = if d_len < 50 {
                0.60 * content_size.height
            } else {
                0.60 * content_size.height * (45.0 / d_len as f32)
            };
            let d_text = Text {
                color: Color::BLACK,
                position: Point::new(down_start.x, down_start.y + 0.20 * content_size.height),
                size: d_size.into(),
                content: d_s,
                ..Text::default()
            };
            frame.fill_text(d_text);
        });

        let solved = self.solved_highlight_cache.draw(renderer, bounds.size(), |frame| {
            if self.backend.borrow().is_solved() {
                let color = Color::from_rgba(0.0,1.0,0.0,0.3);
                for sq in &frame_grid_info.frame_square_infos {
                    let sq_path = Path::rectangle(sq.content_top_left_corner,Size::new(frame_grid_info.content_width,frame_grid_info.content_width));
                    frame.fill(&sq_path, color);
                };
            }
        });

        vec![grid,labels,content,modifiers,highlighter,clues,solved]
    }
}
