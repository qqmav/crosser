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
    fn new(x: u32, y: u32) -> Self {
        Square {
            content: SquareContents::TextContent("Test".to_string(),None),
            label: None,
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
        let v = vec!(d * d,)
        Puzzle {
            title: "New Puzzle".to_string(),
            dim: d;
            variant,
            squares: vec![d * d],
        }
    }
}

fn match_puzzle_dim(p: &PuzzleType) -> usize {
    match p {
        PuzzleType::Mini => 5,
        PuzzleType::Weekday => 15,
        PuzzleType::Sunday => 21,
    }
}
