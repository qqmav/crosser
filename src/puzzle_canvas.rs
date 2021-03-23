use iced::{
       canvas::{self, event::{self, Event}, Cursor, Path, Stroke, Text},
       mouse, keyboard, Color, Point, Rectangle, Size, HorizontalAlignment, VerticalAlignment,
       };
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

pub struct PuzzleCanvas {
    backend: Rc<RefCell<puzzle_backend::Puzzle>>,
    dim: u32,
    grid_info: GridUIInfo,
    cursor_pos: Point,
    ignore_keystrokes: bool,
    lctrl_held: bool,
    rctrl_held: bool,
    hovered_square: Option<(u32, u32)>,
    selected_square: Option<(u32, u32)>,
    selected_variant: puzzle_backend::EntryVariant,
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
            grid_info: GridUIInfo::new(&Rectangle::with_size(Size::new(1.0,1.0)), d as u32),
            dim: d as u32,
            cursor_pos: Point::new(0.0,0.0),
            ignore_keystrokes: false,
            lctrl_held: false,
            rctrl_held: false,
            hovered_square: None,
            selected_square: None,
            selected_variant: puzzle_backend::EntryVariant::Across,
            grid_cache: Default::default(),
            label_cache: Default::default(),
            content_cache: Default::default(),
            modifier_cache: Default::default(),
            highlighter_cache: Default::default(),
            clues_cache: Default::default(),
            solved_highlight_cache: Default::default(),
        }
    }

    pub fn set_ignore_keystrokes(&mut self, val: bool) {
        self.selected_square = None;
        self.ignore_keystrokes = val;
    }

    pub fn set_no_selected_square(&mut self) {
        self.selected_square = None;
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
    fn update(&mut self, event: Event, bounds: Rectangle, cursor: Cursor) -> (event::Status, Option<Message>) {
        let mut ui_updated = self.grid_info.update(&bounds,self.dim);
        let mut e = event::Status::Captured;
        let mut m: Option<Message> = None;
        match event {
            Event::Mouse(mouse_event) => {
                if cursor.position_in(&bounds).is_none() {
                    return (event::Status::Ignored,None)
                }
                match mouse_event {
                    mouse::Event::CursorMoved { .. } => {
                        if let Some(position) = cursor.position_in(&bounds) {
                            self.cursor_pos = position;
                            let new_sq = project_cursor_into_square(&self.cursor_pos, &self.grid_info.square_width, &self.dim);
                            if self.hovered_square != new_sq {
                                self.hovered_square = new_sq;
                                self.highlighter_cache.clear();
                                self.clues_cache.clear();
                            }
                        } else {
                            e = event::Status::Ignored;
                        }
                    },
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if self.lctrl_held || self.rctrl_held {
                            if let Some((tx,ty)) = self.hovered_square {
                                let did_modify_sq = self.backend.borrow_mut().cycle_modifier(tx,ty);
                                ui_updated = did_modify_sq;
                            }
                        } else {
                            if self.hovered_square == None {
                                e = event::Status::Ignored;
                            } else {
                                if self.hovered_square == self.selected_square {
                                    self.selected_variant = match self.selected_variant {
                                        puzzle_backend::EntryVariant::Across => puzzle_backend::EntryVariant::Down,
                                        puzzle_backend::EntryVariant::Down => puzzle_backend::EntryVariant::Across,
                                    }
                                } else {
                                    // Can unwrap because we have already handled None case.
                                    let (hx,hy) = self.hovered_square.unwrap();
                                    match self.backend.borrow().at(hx,hy).content {
                                        puzzle_backend::SquareContents::TextContent(_,_) => {
                                            self.selected_square = self.hovered_square;
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
                        match self.selected_square {
                            Some((_sx,_sy)) => {
                                self.selected_square = None;
                                ui_updated = true;
                            },
                            None => {
                                if let Some((tx,ty)) = self.hovered_square {
                                    self.backend.borrow_mut().cycle_blocker(tx,ty,false);
                                    ui_updated = true;
                                    m = Some(Message::CluesUpdated);
                                } else {
                                    e = event::Status::Ignored;
                                }
                            },
                        }
                    },
                    _ => ()
                }
            }
            Event::Keyboard(keyboard_event) => {
                if !self.ignore_keystrokes {
                    match keyboard_event {
                        keyboard::Event::KeyPressed { key_code: kc, modifiers: m }  => {
                            match kc {
                                iced::keyboard::KeyCode::Escape => {
                                    self.selected_square = None;
                                    self.highlighter_cache.clear();
                                },
                                iced::keyboard::KeyCode::LControl => {
                                    self.lctrl_held = true;
                                },
                                iced::keyboard::KeyCode::RControl => {
                                    self.rctrl_held = true;
                                },
                                iced::keyboard::KeyCode::Backspace => {
                                    if let Some((tx,ty)) = self.selected_square {
                                        self.backend.borrow_mut().clear_sq_contents(tx,ty);
                                        ui_updated = true;

                                        let prev = match self.selected_variant {
                                            puzzle_backend::EntryVariant::Across => {
                                                self.backend.borrow().at(tx,ty).prev_across
                                            },
                                            puzzle_backend::EntryVariant::Down => {
                                                self.backend.borrow().at(tx,ty).prev_down
                                            },
                                        };

                                        if let Some(s) = prev {
                                            let prev_sq = &self.backend.borrow().squares[s];
                                            self.selected_square = Some((prev_sq.x,prev_sq.y));
                                        };
                                    }
                                },
                                iced::keyboard::KeyCode::Delete => {
                                    if let Some((tx,ty)) = self.selected_square {
                                        self.backend.borrow_mut().clear_sq_contents(tx,ty);
                                        ui_updated = true;
                                    }
                                },
                                iced::keyboard::KeyCode::Up => {
                                    if let Some((tx,ty)) = self.selected_square {
                                        match self.selected_variant {
                                            puzzle_backend::EntryVariant::Down => {
                                                if ty > 0 {
                                                    self.selected_square = Some((tx,ty-1));
                                                    ui_updated = true;
                                                }
                                            },
                                            puzzle_backend::EntryVariant::Across => {
                                                self.selected_variant = puzzle_backend::EntryVariant::Down;
                                                ui_updated = true;
                                            },
                                        };
                                    }
                                },
                                iced::keyboard::KeyCode::Down => {
                                    if let Some((tx,ty)) = self.selected_square {
                                        match self.selected_variant {
                                            puzzle_backend::EntryVariant::Down => {
                                                if ty < self.dim - 1 {
                                                    self.selected_square = Some((tx,ty+1));
                                                    ui_updated = true;
                                                }
                                            },
                                            puzzle_backend::EntryVariant::Across => {
                                                self.selected_variant = puzzle_backend::EntryVariant::Down;
                                                ui_updated = true;
                                            },
                                        };
                                    }
                                },
                                iced::keyboard::KeyCode::Left => {
                                    if let Some((tx,ty)) = self.selected_square {
                                        match self.selected_variant {
                                            puzzle_backend::EntryVariant::Across => {
                                                if tx > 0 {
                                                    self.selected_square = Some((tx-1,ty));
                                                    ui_updated = true;
                                                }
                                            },
                                            puzzle_backend::EntryVariant::Down => {
                                                self.selected_variant = puzzle_backend::EntryVariant::Across;
                                                ui_updated = true;
                                            },
                                        };
                                    }
                                },
                                iced::keyboard::KeyCode::Right => {
                                    if let Some((tx,ty)) = self.selected_square {
                                        match self.selected_variant {
                                            puzzle_backend::EntryVariant::Across => {
                                                if tx < self.dim - 1 {
                                                    self.selected_square = Some((tx+1,ty));
                                                    ui_updated = true;
                                                }
                                            },
                                            puzzle_backend::EntryVariant::Down => {
                                                self.selected_variant = puzzle_backend::EntryVariant::Across;
                                                ui_updated = true;
                                            },
                                        };
                                    }
                                },
                                iced::keyboard::KeyCode::Tab => {
                                    if let Some((tx,ty)) = self.selected_square {
                                        let (next_var,next_clue_index) = match self.selected_variant {
                                            puzzle_backend::EntryVariant::Across => {
                                                let a_entries = &self.backend.borrow().across_entries;
                                                let a_entry = self.backend.borrow().at(tx,ty).across_entry;
                                                if let Some(num) = a_entry {
                                                    let a_index = a_entries.iter().position(|x| x.label == num);
                                                    if let Some(a) = a_index {
                                                        if !m.shift {
                                                            if a < a_entries.len() - 1 {
                                                                (Some(puzzle_backend::EntryVariant::Across), Some(a + 1))
                                                            } else {
                                                                self.selected_variant = puzzle_backend::EntryVariant::Down;
                                                                (Some(puzzle_backend::EntryVariant::Down),Some(0))
                                                            }
                                                        } else {
                                                            if a > 0 {
                                                                (Some(puzzle_backend::EntryVariant::Across),Some(a - 1))
                                                            } else {
                                                                self.selected_variant = puzzle_backend::EntryVariant::Down;
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
                                                        if !m.shift {
                                                            if d < d_entries.len() - 1 {
                                                                (Some(puzzle_backend::EntryVariant::Down),Some(d + 1))
                                                            } else {
                                                                self.selected_variant = puzzle_backend::EntryVariant::Across;
                                                                (Some(puzzle_backend::EntryVariant::Across),Some(0))
                                                            }
                                                        } else {
                                                            if d > 0 {
                                                                (Some(puzzle_backend::EntryVariant::Down),Some(d - 1))
                                                            } else {
                                                                self.selected_variant = puzzle_backend::EntryVariant::Across;
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
                                                        self.selected_square = Some((sq.x,sq.y));
                                                    },
                                                    puzzle_backend::EntryVariant::Down => {
                                                        let sq = &self.backend.borrow().squares[self.backend.borrow().down_entries[i].member_indices[0]];
                                                        self.selected_square = Some((sq.x,sq.y));
                                                    }
                                                }
                                            };
                                        };
                                        ui_updated = true;
                                    }
                                },
                                iced::keyboard::KeyCode::Space => {
                                    if let Some ((tx,ty)) = self.selected_square {
                                        self.backend.borrow_mut().clear_sq_contents(tx,ty);
                                        let next = match self.selected_variant {
                                            puzzle_backend::EntryVariant::Across => {
                                                self.backend.borrow().at(tx,ty).next_across
                                            },
                                            puzzle_backend::EntryVariant::Down => {
                                                self.backend.borrow().at(tx,ty).next_down
                                            },
                                        };
                                        if let Some(s) = next {
                                            let next_sq = &self.backend.borrow().squares[s];
                                            self.selected_square = Some((next_sq.x,next_sq.y));
                                        };
                                        ui_updated = true;
                                    }
                                },
                                _ => {
                                    if let Some(c) = match_keycode_to_char(&kc) {
                                        if let Some((tx,ty)) = self.selected_square {
                                            self.backend.borrow_mut().modify_sq_contents(tx,ty,c,m.shift);
                                            ui_updated = true;
                                            if !m.control  && !m.shift {
                                                // If ctrl isnt held, move to next letter.
                                                let next = match self.selected_variant {
                                                    puzzle_backend::EntryVariant::Across => {
                                                        self.backend.borrow().at(tx,ty).next_across
                                                    }
                                                    puzzle_backend::EntryVariant::Down => {
                                                        self.backend.borrow().at(tx,ty).next_down
                                                    }
                                                };

                                                if let Some(s) = next {
                                                    let next_sq = &self.backend.borrow().squares[s];
                                                    self.selected_square = Some((next_sq.x,next_sq.y));
                                                };
                                            }
                                        }
                                    } else {
                                        e = event::Status::Ignored;
                                    }
                                },
                            };
                        }, 
                        keyboard::Event::KeyReleased { key_code: kc, modifiers: _m }  => {
                            match kc {
                                iced::keyboard::KeyCode::LControl => {
                                    self.lctrl_held = false;
                                },
                                iced::keyboard::KeyCode::RControl => {
                                    self.rctrl_held = false;
                                },
                                _ => {
                                    e = event::Status::Ignored;
                                },
                            };
                        },
                        _ => (),
                    }
                }   
            }
        };

        if ui_updated {
            self.grid_cache.clear();
            self.label_cache.clear();
            self.content_cache.clear();
            self.modifier_cache.clear();
            self.highlighter_cache.clear();
            self.clues_cache.clear();
            self.solved_highlight_cache.clear();
        }

        (e, m)
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor,) -> Vec<canvas::Geometry> {
        // We cant assign to ourselves, but if the bound doesn't match, then we need to maek a new griduiinfo
        let (canv_w, canv_h) = (bounds.size().width, bounds.size().height);
        let min_size = match canv_w < canv_h { 
            true => canv_w,
            false  => canv_h,
        }; 
        let uncached_frame_grid_info = if min_size != self.grid_info.min_size {
            Some(GridUIInfo::new(&bounds,self.dim))
        } else {
            None
        };

        let frame_grid_info = match &uncached_frame_grid_info {
            Some(g) => &g,
            None => &self.grid_info,
        };

        let grid = self.grid_cache.draw(bounds.size(), |frame| {
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

        let labels = self.label_cache.draw(bounds.size(), |frame| {
            for sq in &frame_grid_info.frame_square_infos {
                if let Some(l) = self.backend.borrow().at(sq.x,sq.y).label {
                    let text = Text {
                        color: Color::BLACK,
                        position: sq.content_top_left_corner,
                        size: frame_grid_info.label_size,
                        content: l.to_string(),
                        ..Text::default()
                    };
                    frame.fill_text(text);
                };
            }
        });

        let content = self.content_cache.draw(bounds.size(), |frame| {
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
                        size: sq_text_size,
                        content: s.clone(),
                        horizontal_alignment: HorizontalAlignment::Center,
                        vertical_alignment: VerticalAlignment::Center,
                        ..Text::default()
                    };
                    frame.fill_text(text);
                };
            }
        });

        let modifiers = self.modifier_cache.draw(bounds.size(), |frame| {
            let stroke = Stroke {
                color: Color::BLACK,
                width: frame_grid_info.content_width * 0.03,
                ..Default::default()
            };
            for sq in &frame_grid_info.frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(_s,Some(puzzle_backend::SquareModifier::Circle)) = &self.backend.borrow().at(sq.x,sq.y).content {
                    frame.stroke(&Path::circle(sq.center, 0.92 * frame_grid_info.content_width / 2.0),stroke);
                }
            }
        });

        let highlighter = self.highlighter_cache.draw(bounds.size(), |frame| {
            match self.selected_square {
                None => {
                    if let Some((sx,sy)) = project_cursor_into_square(&self.cursor_pos,&frame_grid_info.square_width, &self.dim) {
                        let r_path = Path::rectangle(
                                frame_grid_info.frame_square_infos[sx as usize * self.dim as usize + sy as usize].content_top_left_corner,
                                Size::new(frame_grid_info.content_width,frame_grid_info.content_width)
                            );
                        let r_c = Color::from_rgba(0.0,0.0,1.0,0.2);
                        frame.fill(&r_path,r_c);
                        match self.backend.borrow().variant {
                            puzzle_backend::PuzzleType::Weekday | puzzle_backend::PuzzleType::Sunday => {
                                if (sx != self.dim / 2 as u32) || (sy != self.dim / 2 as u32) {
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
                        match self.selected_variant {
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

        let clues = self.clues_cache.draw(bounds.size(), |frame| {
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

            let (a_c,d_c) = match self.selected_square {
                Some((_sx,_sy)) => {
                    match self.selected_variant {
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

            let (a_s,d_s) = match self.selected_square {
                Some((sx,sy)) => {
                    self.backend.borrow().get_square_clue_texts(sx,sy)
                },
                None => {
                    match self.hovered_square {
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
                size: a_size,
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
                size: d_size,
                content: d_s,
                ..Text::default()
            };
            frame.fill_text(d_text);
        });

        let solved = self.solved_highlight_cache.draw(bounds.size(), |frame| {
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

fn match_keycode_to_char(kc: &iced::keyboard::KeyCode) -> Option<char> {
    match *kc {
        iced::keyboard::KeyCode::A => Some('A'),
        iced::keyboard::KeyCode::B => Some('B'),
        iced::keyboard::KeyCode::C => Some('C'),
        iced::keyboard::KeyCode::D => Some('D'),
        iced::keyboard::KeyCode::E => Some('E'),
        iced::keyboard::KeyCode::F => Some('F'),
        iced::keyboard::KeyCode::G => Some('G'),
        iced::keyboard::KeyCode::H => Some('H'),
        iced::keyboard::KeyCode::I => Some('I'),
        iced::keyboard::KeyCode::J => Some('J'),
        iced::keyboard::KeyCode::K => Some('K'),
        iced::keyboard::KeyCode::L => Some('L'),
        iced::keyboard::KeyCode::M => Some('M'),
        iced::keyboard::KeyCode::N => Some('N'),
        iced::keyboard::KeyCode::O => Some('O'),
        iced::keyboard::KeyCode::P => Some('P'),
        iced::keyboard::KeyCode::Q => Some('Q'),
        iced::keyboard::KeyCode::R => Some('R'),
        iced::keyboard::KeyCode::S => Some('S'),
        iced::keyboard::KeyCode::T => Some('T'),
        iced::keyboard::KeyCode::U => Some('U'),
        iced::keyboard::KeyCode::V => Some('V'),
        iced::keyboard::KeyCode::W => Some('W'),
        iced::keyboard::KeyCode::X => Some('X'),
        iced::keyboard::KeyCode::Y => Some('Y'),
        iced::keyboard::KeyCode::Z => Some('Z'),
        _ => None,
    }
}
