use iced::{
       canvas::{self, Cursor, Path, Stroke, Text},
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
    grid_cache: canvas::Cache,
    label_cache: canvas::Cache,
    content_cache: canvas::Cache,
    modifier_cache: canvas::Cache,
}

impl PuzzleCanvas {
    pub fn new(variant: puzzle_backend::PuzzleType) -> PuzzleCanvas {
        let d = puzzle_backend::match_puzzle_dim(&variant);
        PuzzleCanvas {
            backend: puzzle_backend::Puzzle::new(variant),
            dim: d as u32,
            grid_cache: Default::default(),
            label_cache: Default::default(),
            content_cache: Default::default(),
            modifier_cache: Default::default(),
        }
    }
}

impl<Message> canvas::Program<Message> for PuzzleCanvas {
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
        
        vec![grid,labels,content,modifiers]
    }
}