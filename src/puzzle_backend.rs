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
    pub across_entry: Option<u32>,
    pub down_entry: Option<u32>,
}

impl Square {
    fn new(x: u32, y: u32) -> Self {
        Square {
            content: SquareContents::TextContent("".to_string(),None),
            label: None,
            x,
            y,
            across_entry: None,
            down_entry: None,
        }
    }
}

#[derive(Clone)]
pub enum EntryVariant {
    Across,
    Down,
}

#[derive(Clone)]
pub struct PuzzleEntry {
    pub label: u32,
    pub variant: EntryVariant,
    pub member_indices: Vec<usize>,
}

pub enum PuzzleType {
    Mini,
    Weekday,
    WeekdayAssymetric,
    Sunday,
}

pub struct Puzzle {
    pub title: String,
    dim: usize,
    pub variant: PuzzleType,
    pub squares: Vec<Square>,
    pub across_entries: Vec<PuzzleEntry>,
    pub down_entries: Vec<PuzzleEntry>,
}

impl Puzzle {
    pub fn new(variant: PuzzleType) -> Self {
        let d = match_puzzle_dim(&variant);
        let mut v: Vec<Square> = Vec::with_capacity(d * d);
        
        // Squares are stored in row-major order
        for y_index in 0..d  as u32{
            for x_index in 0..d as u32{
                v.push(Square::new(x_index,y_index));
            }
        }

        let mut p = Puzzle {
            title: "New Puzzle".to_string(),
            dim: d,
            variant,
            squares: v,
            across_entries: Vec::new(),
            down_entries: Vec::new(),
        };
        p.calculate_clues();
        p
    }

    pub fn at(&self, x: u32, y: u32) -> &Square {
        let index = self.xy_to_index(x, y);
        &self.squares[index]
    }

    pub fn cycle_blocker(&mut self, x: u32, y: u32, nested: bool)  {
        let index = self.xy_to_index(x, y);
        match self.squares[index].content {
            SquareContents::Blocker => {
                self.squares[index].content = SquareContents::TextContent("".to_string(),None);
            },
            SquareContents::TextContent(_,_) => {
                self.squares[index].content = SquareContents::Blocker;
            },
        };
        match self.variant {
            PuzzleType::Weekday  | PuzzleType::Sunday => {
                // Also block symmetric piece.
                if !nested {
                    if y < (self.dim / 2) as u32 {
                        self.cycle_blocker(self.dim as u32 - x - 1, self.dim as u32 - y - 1, true);
                    } else if y > (self.dim / 2) as u32 {
                        self.cycle_blocker(self.dim as u32 - x - 1, self.dim as u32 - y - 1, true);
                    } else {
                        if x != self.dim as u32 / 2 {
                            self.cycle_blocker(self.dim as u32 - x - 1, self.dim as u32 - y - 1, true);
                        }
                    }
                }
            },
            _ => (),
        }
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

    pub fn clear_sq_contents(&mut self, x: u32, y: u32) {
        let index = self.xy_to_index(x, y);
        match &self.squares[index].content {
            SquareContents::TextContent(_,modifier_option) => {
                self.squares[index].content = SquareContents::TextContent(String::new(),modifier_option.clone());
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

        // Construct across entries
        self.across_entries.clear();
        for y in 0..self.dim as u32 {
            let mut current_across = 0;
            let mut entries: Vec<usize> = Vec::new();
            for x in 0..self.dim as u32 {
                let index = self.xy_to_index(x, y);
                if let Some(l) = self.squares[index].label {
                    if entries.len() == 0 {
                        current_across = l;
                    }
                };
                match &self.squares[index].content {
                    SquareContents::Blocker => {
                        self.squares[index].across_entry = None;
                        if entries.len() > 0 {
                            let e = PuzzleEntry {
                                label: current_across,
                                variant: EntryVariant::Across,
                                member_indices: entries,
                            };
                            self.across_entries.push(e);
                            entries = Vec::new();
                        }
                    }
                    SquareContents::TextContent(_,_) => {
                        self.squares[index].across_entry = Some(current_across);
                        entries.push(index);
                    }
                }
            }

            if entries.len() > 0 {
                let e = PuzzleEntry {
                    label: current_across,
                    variant: EntryVariant::Down,
                    member_indices: entries,
                };
                self.across_entries.push(e);
            }
        }

        // Construct down entries
        self.down_entries.clear();
        for x in 0..self.dim as u32 {
            let mut current_down = 0;
            let mut entries: Vec<usize> = Vec::new();
            for y in 0..self.dim as u32 {
                let index = self.xy_to_index(x, y);
                if let Some(l) = self.squares[index].label {
                    if entries.len() == 0 {
                        current_down = l;
                    }
                };
                match &self.squares[index].content {
                    SquareContents::Blocker => {
                        self.squares[index].down_entry = None;
                        if entries.len() > 0 {
                            let e = PuzzleEntry {
                                label: current_down,
                                variant: EntryVariant::Across,
                                member_indices: entries,
                            };
                            self.down_entries.push(e);
                            entries = Vec::new();
                        }
                    }
                    SquareContents::TextContent(_,_) => {
                        self.squares[index].down_entry = Some(current_down);
                        entries.push(index);
                    }
                }
            }

            if entries.len() > 0 {
                let e = PuzzleEntry {
                    label: current_down,
                    variant: EntryVariant::Across,
                    member_indices: entries,
                };
                self.down_entries.push(e);
            }
        }

        // Down entries won't be in increasing order, so sort them.
        self.down_entries.sort_by(|a,b| a.label.cmp(&b.label));
    }

    fn xy_to_index(&self, x: u32, y: u32) -> usize {
        (y * self.dim as u32 + x) as usize
    }

    fn get_clue_string(&self, indices: &Vec<usize>) -> String {
        let mut s = String::new();
        for index in indices { 
            if let SquareContents::TextContent(sq_content,_) = &self.squares[*index].content {
                match sq_content.as_str() {
                    "" => s.push('_'),
                    _ => s.push_str(sq_content),
                };
            };
        };
        s
    }
}

pub fn match_puzzle_dim(p: &PuzzleType) -> usize {
    match p {
        PuzzleType::Mini => 5,
        PuzzleType::Weekday => 15,
        PuzzleType::WeekdayAssymetric => 15,
        PuzzleType::Sunday => 21,
    }
}
