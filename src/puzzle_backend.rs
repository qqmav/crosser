pub enum SquareContents {
    Blocker,
    TextContent(String, Option<SquareModifier>),
}

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
        let mut c = SquareContents::TextContent("".to_string(),None);
        if label == Some(5) {
            c = SquareContents::Blocker;
        }
        if label == Some(30) {
            c = SquareContents::TextContent("A".to_string(),Some(SquareModifier::Shading));
        }
        if label == Some(45) {
            c = SquareContents::TextContent("A".to_string(),Some(SquareModifier::Circle));
        }
        Square {
            content: c,
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

        Puzzle {
            title: "New Puzzle".to_string(),
            dim: d,
            variant,
            squares: v,
        }
    }

    pub fn at(&self, x: u32, y: u32) -> &Square {
        let index = y * self.dim as u32 + x;
        &self.squares[index as usize]
    }
}

pub fn match_puzzle_dim(p: &PuzzleType) -> usize {
    match p {
        PuzzleType::Weekday => 15,
    }
}
