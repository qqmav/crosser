pub enum SquareContents {
    Blocker,
    TextContent(String, Option<SquareModifier>),
}

#[derive(Clone)]
pub enum SquareModifier {
    Shading,
    Circle,
}

pub struct Square {
    pub content: SquareContents,
    pub label: Option<u32>,
    pub x: u32,
    pub y: u32,
}

impl Square {
    fn new(x: u32, y: u32, label: Option<u32>) -> Self {
        Square {
            content: SquareContents::TextContent("".to_string(),None),
            label,
            x,
            y,
        }
    }
}

pub enum PuzzleType {
    Weekday,
}

pub struct Puzzle {
    pub title: String,
    dim: usize,
    pub variant: PuzzleType,
    squares: Vec<Square>,
}

impl Puzzle {
    pub fn new(variant: PuzzleType) -> Self {
        let d = match_puzzle_dim(&variant);
        let mut v: Vec<Square> = Vec::with_capacity(d * d);
        
        // Squares are stored in row-major order
        for y_index in 0..d  as u32{
            for x_index in 0..d as u32{
                v.push(Square::new(x_index,y_index,Some(y_index * (d as u32) + x_index)));
            }
        }

        let mut p = Puzzle {
            title: "New Puzzle".to_string(),
            dim: d,
            variant,
            squares: v,
        };
        p.calculate_clues();
        p
    }

    pub fn at(&self, x: u32, y: u32) -> &Square {
        let index = self.xy_to_index(x, y);
        &self.squares[index]
    }

    pub fn cycle_blocker(&mut self, x: u32, y: u32)  {
        let index = self.xy_to_index(x, y);
        match self.squares[index].content {
            SquareContents::Blocker => {
                self.squares[index].content = SquareContents::TextContent("".to_string(),None);
            },
            SquareContents::TextContent(_,_) => {
                self.squares[index].content = SquareContents::Blocker;
            },
        };
        self.calculate_clues();
    }

    pub fn cycle_modifier(&mut self, x: u32, y: u32) -> bool {
        let index = self.xy_to_index(x, y);
        match &self.squares[index].content {
            SquareContents::Blocker => false,
            SquareContents::TextContent(s,modifier_option) => {
                match modifier_option {
                    None => {
                        self.squares[index].content = SquareContents::TextContent(s.clone(),Some(SquareModifier::Shading));
                    }
                    Some(SquareModifier::Shading) => {
                        self.squares[index].content = SquareContents::TextContent(s.clone(),Some(SquareModifier::Circle));
                    }
                    Some(SquareModifier::Circle) => {
                        self.squares[index].content = SquareContents::TextContent(s.clone(),None);
                    }
                };
                true
            },
        }
    }

    pub fn modify_sq_contents(&mut self, x: u32, y: u32, c: char, append: bool) {
        let index = self.xy_to_index(x, y);
        match &self.squares[index].content {
            SquareContents::TextContent(s,modifier_option) => {
                if append {
                    let mut newstr = s.clone();
                    newstr.push(c);
                    self.squares[index].content = SquareContents::TextContent(newstr,modifier_option.clone());
                } else {
                    self.squares[index].content = SquareContents::TextContent(c.to_string(),modifier_option.clone());
                }
            }
            _ => ()
        }
    }

    pub fn calculate_clues(&mut self) {
        let mut start_of_across_clue: Vec<bool> = vec![false; self.dim * self.dim];
        for y in 0..self.dim as u32 {
            let start_index = self.xy_to_index(0, y);
            let mut was_blocker = match self.squares[start_index].content {
                SquareContents::Blocker => {
                    true
                },
                _ => {
                    start_of_across_clue[start_index] = true;
                    false
                },
            };
            for x in 1..self.dim as u32 {
                let index = self.xy_to_index(x, y);
                match self.squares[index].content {
                    SquareContents::Blocker => {
                        was_blocker = true;
                    },
                    _ => {
                        if was_blocker {
                            start_of_across_clue[index] = true;
                            was_blocker = false;
                        }
                    },
                }
            }
        }

        let mut start_of_down_clue: Vec<bool> = vec![false; self.dim * self.dim];
        for x in 0..self.dim as u32 {
            let start_index = self.xy_to_index(x, 0);
            let mut was_blocker = match self.squares[start_index].content {
                SquareContents::Blocker => {
                    true
                },
                _ => {
                    start_of_down_clue[start_index] = true;
                    false
                },
            };
            for y in 1..self.dim as u32 {
                let index = self.xy_to_index(x, y);
                match self.squares[index].content {
                    SquareContents::Blocker => {
                        was_blocker = true;
                    },
                    _ => {
                        if was_blocker {
                            start_of_down_clue[index] = true;
                            was_blocker = false;
                        }
                    },
                }
            }
        }

        let mut current_label = 0;
        for y in 0..self.dim as u32 {
            for x in 0..self.dim as u32 {
                let index = self.xy_to_index(x, y);
                let l = if start_of_across_clue[index] || start_of_down_clue[index] {
                    current_label += 1;
                    Some(current_label)
                } else {
                    None
                };
                self.squares[index].label = l;
            }
        }
    }

    fn xy_to_index(&self, x: u32, y: u32) -> usize {
        (y * self.dim as u32 + x) as usize
    }
}

pub fn match_puzzle_dim(p: &PuzzleType) -> usize {
    match p {
        PuzzleType::Weekday => 15,
    }
}
