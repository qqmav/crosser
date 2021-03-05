enum SquareContents {
    Blocker,
    TextContent(String, Option<SquareModifier>),
}

enum SquareModifier {
    Shading(String),
    Circle(String),
}

struct Square {
    content: SquareContents,
    label: Option<u32>,
    x: u32,
    y: u32,
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
    Mini,
    Weekday,
    Sunday,
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

    pub fn at(&self, x: usize, y: usize) -> &Square {
        let index = y * self.dim + x;
        &self.squares[index]
    }
}

fn match_puzzle_dim(p: &PuzzleType) -> usize {
    match p {
        PuzzleType::Mini => 5,
        PuzzleType::Weekday => 15,
        PuzzleType::Sunday => 21,
    }
}
