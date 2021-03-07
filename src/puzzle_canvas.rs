use iced::{
       canvas::{self, event::{self, Event}, Cursor, Path, Stroke, Text},
       Color, Point, Rectangle, Size, HorizontalAlignment, VerticalAlignment,
       };
use crate::puzzle_backend;

struct SquareUIInfo {
    x: u32,
    y: u32,
    center: Point,
    content_top_left_corner: Point,
}

pub struct PuzzleCanvas {
    backend: puzzle_backend::Puzzle,
    dim: u32,
    cursor_pos: Point,
    highlighted_square: Option<(u32, u32)>,
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
            dim: d as u32,
            cursor_pos: Point::new(0.0,0.0),
            highlighted_square: None,
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
        if let Some(position) = cursor.position_in(&bounds) {
            self.cursor_pos = position;
            self.highlighter_cache.clear();
        }
        (event::Status::Captured,None)
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor,) -> Vec<canvas::Geometry> {
        let (canv_w, canv_h) = (bounds.size().width, bounds.size().height);
        let min_size = match canv_w < canv_h { 
            true => canv_w,
            false  => canv_h,
        };
        let sq_width = min_size / self.dim as f32;

        let padding_width = 0.05 * sq_width;
        let content_width = sq_width - padding_width;
        let label_size = 0.30 * sq_width;

        let mut frame_square_infos: Vec<SquareUIInfo> = Vec::with_capacity(self.dim as usize * self.dim as usize);
        // Column-major
        for x in 0..self.dim {
            for y in 0..self.dim {
                let p = Point {
                    x: sq_width * ((x as f32) + 0.5),
                    y: sq_width * ((y as f32) + 0.5),
                };
                let c = Point {
                    x: p.x - content_width / 2.0,
                    y: p.y - content_width / 2.0,
                };
                frame_square_infos.push(SquareUIInfo {
                    x,
                    y,
                    center: p,
                    content_top_left_corner: c,
                });
            }
        }

        let grid = self.grid_cache.draw(bounds.size(), |frame| {
            let dark_bg = Path::rectangle(Point::new(0.0,0.0), Size::new(min_size,min_size));
            frame.fill(&dark_bg, Color::BLACK);
            for sq in &frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(_s,m) = &self.backend.at(sq.x, sq.y).content {
                    let sq_path = Path::rectangle(sq.content_top_left_corner,Size::new(content_width,content_width));
                    let color = match m {
                        Some(puzzle_backend::SquareModifier::Shading) => Color::from_rgba(0.8, 0.8, 0.8, 1.0),
                        _ => Color::WHITE,
                    };
                    frame.fill(&sq_path, color);
                }
            };
        });

        let labels = self.label_cache.draw(bounds.size(), |frame| {
            for sq in &frame_square_infos {
                if let Some(l) = self.backend.at(sq.x,sq.y).label {
                    let text = Text {
                        color: Color::BLACK,
                        position: sq.content_top_left_corner,
                        size: label_size,
                        content: l.to_string(),
                        ..Text::default()
                    };
                    frame.fill_text(text);
                };
            }
        });

        let content = self.content_cache.draw(bounds.size(), |frame| {
            for sq in &frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(s,_m) = &self.backend.at(sq.x,sq.y).content {
                    let text = Text {
                        color: Color::BLACK,
                        position: sq.center,
                        size: content_width,
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
                width: content_width * 0.03,
                ..Default::default()
            };
            for sq in &frame_square_infos {
                if let puzzle_backend::SquareContents::TextContent(_s,Some(puzzle_backend::SquareModifier::Circle)) = &self.backend.at(sq.x,sq.y).content {
                    frame.stroke(&Path::circle(sq.center, 0.92 * content_width / 2.0),stroke);
                }
            }
        });

        let highlighter = self.highlighter_cache.draw(bounds.size(), |frame| {
            let pc_tuple: Option<(Path,Color)> = match self.highlighted_square {
                None => {
                    match project_cursor_into_square(&self.cursor_pos,&sq_width, &self.dim) {
                        Some((sx,sy)) => Some((Path::rectangle(frame_square_infos[sx as usize * self.dim as usize + sy as usize].content_top_left_corner,Size::new(content_width,content_width)),Color::from_rgba(1.0,1.0,0.0,0.3))),
                        None => None,
                        }
                    },
                Some((hx,hy)) => Some((Path::rectangle(frame_square_infos[hx as usize * self.dim as usize + hy as usize].content_top_left_corner,Size::new(content_width,content_width)),Color::from_rgba(1.0,1.0,0.0,0.5)))
            };
            match pc_tuple {
                Some((p,c)) => {frame.fill(&p,c);},
                None => {}
            };
        });
        
        vec![grid,labels,content,modifiers,highlighter]
    }
}
