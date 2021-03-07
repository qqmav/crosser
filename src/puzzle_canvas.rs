use iced::{
       canvas::{self, event::{self, Event}, Cursor, Path, Stroke, Text},
       mouse, keyboard, Color, Point, Rectangle, Size, HorizontalAlignment, VerticalAlignment,
       };
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
}

impl GridUIInfo {
    fn new(bounds: &Rectangle, dim: u32) -> Self {
        let (canv_w, canv_h) = (bounds.size().width, bounds.size().height);
        let min_size = match canv_w < canv_h { 
            true => canv_w,
            false  => canv_h,
        };
        let square_width = min_size / dim as f32;

        let padding_width = 0.05 * square_width;
        let content_width = square_width - padding_width;
        let label_size = 0.30 * square_width;

        let frame_square_infos = GridUIInfo::get_frame_square_infos(square_width, content_width, dim);
 
        GridUIInfo {
            min_size,
            square_width,
            content_width,
            label_size,
            frame_square_infos,
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

pub struct PuzzleCanvas {
    backend: puzzle_backend::Puzzle,
    dim: u32,
    grid_info: GridUIInfo,
    cursor_pos: Point,
    lctrl_held: bool,
    rctrl_held: bool,
    hovered_square: Option<(u32, u32)>,
    selected_square: Option<(u32, u32)>,
    grid_cache: canvas::Cache,
    label_cache: canvas::Cache,
    content_cache: canvas::Cache,
    modifier_cache: canvas::Cache,
    highlighter_cache: canvas::Cache,
}

impl PuzzleCanvas {
    pub fn new(variant: puzzle_backend::PuzzleType) -> PuzzleCanvas {
        let d = puzzle_backend::match_puzzle_dim(&variant);
        PuzzleCanvas {
            backend: puzzle_backend::Puzzle::new(variant),
            grid_info: GridUIInfo::new(&Rectangle::with_size(Size::new(1.0,1.0)), d as u32),
            dim: d as u32,
            cursor_pos: Point::new(0.0,0.0),
            lctrl_held: false,
            rctrl_held: false,
            hovered_square: None,
            selected_square: None,
            grid_cache: Default::default(),
            label_cache: Default::default(),
            content_cache: Default::default(),
            modifier_cache: Default::default(),
            highlighter_cache: Default::default(),
        }
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

impl<Message> canvas::Program<Message> for PuzzleCanvas {
    fn update(&mut self, event: Event, bounds: Rectangle, cursor: Cursor) -> (event::Status, Option<Message>) {
        let mut ui_updated = self.grid_info.update(&bounds,self.dim);

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    if let Some(position) = cursor.position_in(&bounds) {
                        self.cursor_pos = position;
                        self.hovered_square = project_cursor_into_square(&self.cursor_pos, &self.grid_info.square_width, &self.dim);
                        self.highlighter_cache.clear();
                    }
                },
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if self.lctrl_held || self.rctrl_held {
                        if let Some((tx,ty)) = self.hovered_square {
                            let did_modify_sq = self.backend.cycle_modifier(tx,ty);
                            ui_updated = did_modify_sq;
                        }
                    } else {
                        self.selected_square = self.hovered_square;
                        self.highlighter_cache.clear();
                    }
                },
                mouse::Event::ButtonPressed(mouse::Button::Right) => {
                    ui_updated = true;
                    self.selected_square = None;
                    if let Some((tx,ty)) = self.hovered_square {
                        self.backend.cycle_blocker(tx,ty,false);
                    }
                },
                _ => ()
            }
            Event::Keyboard(keyboard_event) => match keyboard_event {
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
                                    self.backend.clear_sq_contents(tx,ty);
                                    ui_updated = true;
                                }
                        },
                        iced::keyboard::KeyCode::Delete => {
                                if let Some((tx,ty)) = self.selected_square {
                                    self.backend.clear_sq_contents(tx,ty);
                                    ui_updated = true;
                                }
                        },
                        _ => {
                            if let Some(c) = match_keycode_to_char(&kc) {
                                if let Some((tx,ty)) = self.selected_square {
                                    self.backend.modify_sq_contents(tx,ty,c,m.shift);
                                    ui_updated = true;
                                }
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
                        _ => (),
                    };
                },
                _ => (),
            }
        };

        if ui_updated {
            self.grid_cache.clear();
            self.label_cache.clear();
            self.content_cache.clear();
            self.modifier_cache.clear();
            self.highlighter_cache.clear();
        }

        (event::Status::Captured, None)
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor,) -> Vec<canvas::Geometry> {
        
        let grid = self.grid_cache.draw(bounds.size(), |frame| {
            let dark_bg = Path::rectangle(Point::new(0.0,0.0), Size::new(self.grid_info.min_size,self.grid_info.min_size));
            frame.fill(&dark_bg, Color::BLACK);
            for sq in &self.grid_info.frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(_s,m) = &self.backend.at(sq.x, sq.y).content {
                    let sq_path = Path::rectangle(sq.content_top_left_corner,Size::new(self.grid_info.content_width,self.grid_info.content_width));
                    let color = match m {
                        Some(puzzle_backend::SquareModifier::Shading) => Color::from_rgba(0.8, 0.8, 0.8, 1.0),
                        _ => Color::WHITE,
                    };
                    frame.fill(&sq_path, color);
                }
            };
        });

        let labels = self.label_cache.draw(bounds.size(), |frame| {
            for sq in &self.grid_info.frame_square_infos {
                if let Some(l) = self.backend.at(sq.x,sq.y).label {
                    let text = Text {
                        color: Color::BLACK,
                        position: sq.content_top_left_corner,
                        size: self.grid_info.label_size,
                        content: l.to_string(),
                        ..Text::default()
                    };
                    frame.fill_text(text);
                };
            }
        });

        let content = self.content_cache.draw(bounds.size(), |frame| {
            for sq in &self.grid_info.frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(s,_m) = &self.backend.at(sq.x,sq.y).content {
                    let num_chars = s.len();
                    let sq_text_size = match num_chars {
                        1 => self.grid_info.content_width,
                        _ => 1.2 * self.grid_info.content_width / num_chars as f32,
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
                width: self.grid_info.content_width * 0.03,
                ..Default::default()
            };
            for sq in &self.grid_info.frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(_s,Some(puzzle_backend::SquareModifier::Circle)) = &self.backend.at(sq.x,sq.y).content {
                    frame.stroke(&Path::circle(sq.center, 0.92 * self.grid_info.content_width / 2.0),stroke);
                }
            }
        });

        let highlighter = self.highlighter_cache.draw(bounds.size(), |frame| {
            let pc_tuple: Option<(Path,Color)> = match self.selected_square {
                None => {
                    match project_cursor_into_square(&self.cursor_pos,&self.grid_info.square_width, &self.dim) {
                        Some((sx,sy)) => Some((
                            Path::rectangle(
                                self.grid_info.frame_square_infos[sx as usize * self.dim as usize + sy as usize].content_top_left_corner,
                                Size::new(self.grid_info.content_width,self.grid_info.content_width)
                            ),
                            Color::from_rgba(0.0,0.0,1.0,0.2)
                            )),
                        None => None,
                        }
                    },
                Some((hx,hy)) => Some((
                    Path::rectangle(
                        self.grid_info.frame_square_infos[hx as usize * self.dim as usize + hy as usize].content_top_left_corner,
                        Size::new(self.grid_info.content_width,self.grid_info.content_width)
                    ),
                    Color::from_rgba(1.0,1.0,0.0,0.5)
                    )),
            };
            match pc_tuple {
                Some((p,c)) => {frame.fill(&p,c);},
                None => {}
            };
        });
        
        vec![grid,labels,content,modifiers,highlighter]
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