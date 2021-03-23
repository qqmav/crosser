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
    pub next_across: Option<usize>,
    pub prev_across: Option<usize>,
    pub across_clue_text: Option<String>,
    pub down_entry: Option<u32>,
    pub next_down: Option<usize>,
    pub prev_down: Option<usize>,
    pub down_clue_text: Option<String>,
}

impl Square {
    fn new(x: u32, y: u32) -> Self {
        Square {
            content: SquareContents::TextContent("".to_string(),None),
            label: None,
            x,
            y,
            across_entry: None,
            next_across: None,
            prev_across: None,
            across_clue_text: None,
            down_entry: None,
            next_down: None,
            prev_down: None,
            down_clue_text: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EntryVariant {
    Across,
    Down,
}

#[derive(Clone)]
pub struct PuzzleEntry {
    pub label: u32,
    pub variant: EntryVariant,
    pub member_indices: Vec<usize>,
    pub clue: String,
}

#[derive(Clone, Debug)]
pub enum PuzzleType {
    Mini,
    Weekday,
    WeekdayAsymmetric,
    Sunday,
}

pub struct Puzzle {
    pub title: String,
    dim: usize,
    pub variant: PuzzleType,
    pub squares: Vec<Square>,
    pub across_entries: Vec<PuzzleEntry>,
    pub down_entries: Vec<PuzzleEntry>,
    pub fill_only: bool,
    pub solved_hash: Option<u64>,
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
            fill_only: false,
            solved_hash: None,
        };
        p.calculate_clues();
        p
    }

    pub fn at(&self, x: u32, y: u32) -> &Square {
        let index = self.xy_to_index(x, y);
        &self.squares[index]
    }

    pub fn cycle_blocker(&mut self, x: u32, y: u32, nested: bool)  {
        if !self.fill_only {
            let index = self.xy_to_index(x, y);
            match self.squares[index].content {
                SquareContents::Blocker => {
                    self.squares[index].content = SquareContents::TextContent("".to_string(),None);
                },
                SquareContents::TextContent(_,_) => {
                    self.squares[index].across_clue_text = None;
                    self.squares[index].down_clue_text = None;
                    self.squares[index].content = SquareContents::Blocker;
                },
            };
            match self.variant {
                PuzzleType::Weekday  | PuzzleType::Sunday => {
                    // Also block symmetric piece.
                    if !nested {
                        if (y != self.dim as u32 / 2) || (x != self.dim as u32 / 2) {
                            self.cycle_blocker(self.dim as u32 - x - 1, self.dim as u32 - y - 1, true);
                        }
                    }
                },
                _ => (),
            }
            self.calculate_clues();
        }
    }

    pub fn cycle_modifier(&mut self, x: u32, y: u32) -> bool {
        if !self.fill_only {
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
        } else {
            false
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
        if !self.fill_only {
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
                                // assign next squares
                                for e_index in 0..(entries.len()-1) {
                                    self.squares[entries[e_index]].next_across = Some(entries[e_index+1]);
                                }
                                self.squares[*entries.last().unwrap()].next_across = None;
                                // assign prev squares
                                for e_index in 1..entries.len() {
                                    self.squares[entries[e_index]].prev_across = Some(entries[e_index - 1]);
                                }
                                self.squares[entries[0]].prev_across = None;
                                // set clue texts
                                let text = match &self.squares[entries[0]].across_clue_text {
                                    Some(s) => { s.clone() }
                                    None => { self.squares[entries[0]].across_clue_text = Some("".to_string()); 
                                        "".to_string()        
                                    }
                                };
                                for e_index in 1..entries.len() {
                                    self.squares[entries[e_index]].across_clue_text = None;
                                }

                                // Push to across entries list
                                let e = PuzzleEntry {
                                    label: current_across,
                                    variant: EntryVariant::Across,
                                    member_indices: entries,
                                    clue: text,
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
                    // assign next squares
                    for e_index in 0..(entries.len()-1) {
                        self.squares[entries[e_index]].next_across = Some(entries[e_index+1]);
                    }
                    self.squares[*entries.last().unwrap()].next_across = None;
                    // assign prev squares
                    for e_index in 1..entries.len() {
                        self.squares[entries[e_index]].prev_across = Some(entries[e_index - 1]);
                    }
                    self.squares[entries[0]].prev_across = None;
                    // set clue texts
                    let text = match &self.squares[entries[0]].across_clue_text {
                        Some(s) => { s.clone() }
                        None => { self.squares[entries[0]].across_clue_text = Some("".to_string()); 
                            "".to_string()        
                        }
                    };
                    for e_index in 1..entries.len() {
                        self.squares[entries[e_index]].across_clue_text = None;
                    }

                    // Push to across entries list
                    let e = PuzzleEntry {
                        label: current_across,
                        variant: EntryVariant::Across,
                        member_indices: entries,
                        clue: text,
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
                                // assign next squares
                                for e_index in 0..(entries.len()-1) {
                                    self.squares[entries[e_index]].next_down = Some(entries[e_index+1]);
                                }
                                self.squares[*entries.last().unwrap()].next_down = None;
                                // assign prev squares
                                for e_index in 1..entries.len() {
                                    self.squares[entries[e_index]].prev_down = Some(entries[e_index - 1]);
                                }
                                self.squares[entries[0]].prev_down = None;
                                // set clue texts
                                let text = match &self.squares[entries[0]].down_clue_text {
                                    Some(s) => { s.clone() }
                                    None => { self.squares[entries[0]].down_clue_text = Some("".to_string()); 
                                        "".to_string()        
                                    }
                                };
                                for e_index in 1..entries.len() {
                                    self.squares[entries[e_index]].down_clue_text = None;
                                }

                                // Push to entries list
                                let e = PuzzleEntry {
                                    label: current_down,
                                    variant: EntryVariant::Down,
                                    member_indices: entries,
                                    clue: text,
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
                };

                if entries.len() > 0 {
                    // assign next squares
                    for e_index in 0..(entries.len()-1) {
                        self.squares[entries[e_index]].next_down = Some(entries[e_index+1]);
                    }
                    self.squares[*entries.last().unwrap()].next_down = None;
                    // assign prev squares
                    for e_index in 1..entries.len() {
                        self.squares[entries[e_index]].prev_down = Some(entries[e_index - 1]);
                    }
                    self.squares[entries[0]].prev_down = None;
                    // set clue texts
                    let text = match &self.squares[entries[0]].down_clue_text {
                        Some(s) => { s.clone() }
                        None => { self.squares[entries[0]].down_clue_text = Some("".to_string()); 
                            "".to_string()        
                        }
                    };
                    for e_index in 1..entries.len() {
                        self.squares[entries[e_index]].down_clue_text = None;
                    }

                    // Push to entries list
                    let e = PuzzleEntry {
                        label: current_down,
                        variant: EntryVariant::Down,
                        member_indices: entries,
                        clue: text,
                    };
                    self.down_entries.push(e);
                }
            }

            // Down entries won't be in increasing order, so sort them.
            self.down_entries.sort_by(|a,b| a.label.cmp(&b.label));
        }
    }

    pub fn set_clue_text(&mut self, label: u32, variant: EntryVariant, text: String) {
        match variant {
            EntryVariant::Across => {
                let entry = self.across_entries.iter_mut().find(|x| x.label == label).unwrap();
                entry.clue = text.clone();
                self.squares[entry.member_indices[0]].across_clue_text = Some(text);
            },
            EntryVariant::Down  => {
                let entry = self.down_entries.iter_mut().find(|x| x.label == label).unwrap();
                entry.clue = text.clone();
                self.squares[entry.member_indices[0]].down_clue_text = Some(text);
            },
        }
    }

    fn xy_to_index(&self, x: u32, y: u32) -> usize {
        (y * self.dim as u32 + x) as usize
    }

    pub fn get_clue_entries(&self, x: u32, y: u32) -> (Option<&PuzzleEntry>,Option<&PuzzleEntry>) {
        let sq = self.at(x,y);
        match &sq.content {
            SquareContents::Blocker => {
                (None,None)
            },
            SquareContents::TextContent(_s,_m) => {
                let across = if let Some(a_entry) = sq.across_entry {
                    self.across_entries.iter().find(|x| x.label == a_entry)
                } else {
                    None
                };
                let down = if let Some(d_entry) = sq.down_entry {
                    self.down_entries.iter().find(|x| x.label == d_entry)
                } else {
                    None
                };
                (across,down)
            }
        }
    }

    pub fn get_square_clue_texts(&self, x: u32, y: u32) -> (String,String) {
        let (a,d) = self.get_clue_entries(x,y);

        let across = match a {
            Some(entry) => {
                let clue_t = entry.clue.clone();
                entry.label.to_string() + "A: " + &clue_t
            },
            None => {
                "".to_string()
            },
        };
        let down = match d {
            Some(entry) => {
                let clue_t = entry.clue.clone();
                entry.label.to_string() + "D: " + &clue_t
            },
            None => {
                "".to_string()
            },
        };

        (across,down)
    }

    pub fn get_puzzle_total_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut s = String::new();
        for sq in self.squares.iter() {
            let sq_c = match &sq.content {
                SquareContents::TextContent(s,_m) => {
                    s.as_str()
                },
                SquareContents::Blocker => {
                    "#"
                }
            };
            s.push_str(sq_c);
        };

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    pub fn is_solved(&self) -> bool {
        let curr_hash = self.get_puzzle_total_hash();
        if let Some(h) = self.solved_hash {
            curr_hash == h
        } else {
            false
        }
    }
}

pub fn match_puzzle_dim(p: &PuzzleType) -> usize {
    match p {
        PuzzleType::Mini => 5,
        PuzzleType::Weekday => 15,
        PuzzleType::WeekdayAsymmetric => 15,
        PuzzleType::Sunday => 21,
    }
}
