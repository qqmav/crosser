use serde_json::json;
use crate::puzzle_backend;

use std::{rc::Rc, cell::RefCell};

pub fn write_puzzle_to_cro(puzzle: Rc<RefCell<puzzle_backend::Puzzle>>, path_str: String, save_solvable_grid: bool) -> std::result::Result<(),std::io::Error> {
    let puzzle = puzzle.borrow();

    let variant_str = match puzzle.variant {
        puzzle_backend::PuzzleType::Mini => "mini".to_string(),
        puzzle_backend::PuzzleType::Weekday => "weekday".to_string(),
        puzzle_backend::PuzzleType::WeekdayAsymmetric => "weekday_asymmetric".to_string(),
        puzzle_backend::PuzzleType::Sunday => "sunday".to_string(),
    };

    let mut sq_strs: Vec<String> = Vec::new();
    for sq in puzzle.squares.iter() {
        let cont = match &sq.content {
            puzzle_backend::SquareContents::Blocker => {
                "#".to_string()
            },
            puzzle_backend::SquareContents::TextContent(s,m) => {
                let mod_str = match m {
                    None => "/n".to_string(),
                    Some(puzzle_backend::SquareModifier::Shading) => "/s".to_string(),
                    Some(puzzle_backend::SquareModifier::Circle) => "/c".to_string(),
                };
                let mut c = if save_solvable_grid {
                    "".to_string()
                } else {
                    s.clone()
                };
                c.push_str(&mod_str);
                c
            }
        };
        sq_strs.push(cont);
    }

    let mut across_clues: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for entry in puzzle.across_entries.iter() {
        across_clues.insert(entry.label.to_string(), serde_json::Value::String(entry.clue.clone()));
    }
    let mut down_clues: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for entry in puzzle.down_entries.iter() {
        down_clues.insert(entry.label.to_string(), serde_json::Value::String(entry.clue.clone()));
    }

    let hash_string = if save_solvable_grid {
        puzzle.get_puzzle_total_hash().to_string()
    } else {
        "NULL".to_string()
    };

    let json_rep = json!({
        "variant": variant_str, 
        "squares": sq_strs,
        "across_clues": across_clues,
        "down_clues": down_clues,
        "hash_string": hash_string,
    });

    std::fs::write(path_str,json_rep.to_string())
}

pub fn get_puzzle_from_cro(path_str: String) -> std::result::Result<puzzle_backend::Puzzle, String> {
    let generic_json_err = Err("Error parsing JSON.".to_string());

    let file_result = std::fs::read_to_string(path_str);
    if let Err(e) = file_result {
        return Err(e.to_string());
    }
    let file_contents = file_result.unwrap();

    let value_result = serde_json::from_str(&file_contents);
    if let Err(e) = value_result {
        return Err(e.to_string());
    }
    let value_contents: serde_json::Value = value_result.unwrap();
    let variant: puzzle_backend::PuzzleType = match &value_contents["variant"] {
        serde_json::Value::String(s) => {
            match s.as_str() {
                "mini" => puzzle_backend::PuzzleType::Mini,
                "weekday" => puzzle_backend::PuzzleType::Weekday,
                "weekday_asymmetric" => puzzle_backend::PuzzleType::WeekdayAsymmetric,
                "sunday" => puzzle_backend::PuzzleType::Sunday,
                _ => { return generic_json_err; },
            }
        }
        _ => {
            return generic_json_err;
        }
    };
    let mut puz = puzzle_backend::Puzzle::new(variant);

    let sqs = match &value_contents["squares"] {
        serde_json::Value::Array(v) => {
            v
        },
        _ => {
            return generic_json_err;
        },
    };
    for sq_index in 0..sqs.len() {
        match &sqs[sq_index] {
            serde_json::Value::String(s) => {
                let slash_index = s.find('/');
                match slash_index {
                    Some(u) => {
                        let contents = s[0..u].to_string();
                        let modifier = match &s[u+1..] {
                            "n" => None,
                            "c" => Some(puzzle_backend::SquareModifier::Circle),
                            "s" => Some(puzzle_backend::SquareModifier::Shading),
                            _ => {return generic_json_err;}
                        };
                        puz.squares[sq_index].content = puzzle_backend::SquareContents::TextContent(contents,modifier);
                    },
                    None => {
                        puz.squares[sq_index].across_clue_text = None;
                        puz.squares[sq_index].down_clue_text = None;
                        puz.squares[sq_index].content = puzzle_backend::SquareContents::Blocker
                    }
                }
            },
            _ => {
                return generic_json_err;
            }
        }
    }
    // Squares are constructed, we can calculate clues.
    puz.calculate_clues();

    // Now we can use the built-in puz clue assignment.
    let acs = match &value_contents["across_clues"] {
        serde_json::Value::Object(m) => {
            m
        },
        _ => {
            return generic_json_err;
        },
    };
    for (label_str,clue_content) in acs.iter() {
        let label = label_str.parse::<u32>().unwrap();
        let clue_text = match clue_content {
            serde_json::Value::String(s) => {
                s
            },
            _ => {
                return generic_json_err;
            }
        };
        puz.set_clue_text(label, puzzle_backend::EntryVariant::Across, clue_text.clone());
    }
    let dcs = match &value_contents["down_clues"] {
        serde_json::Value::Object(m) => {
            m
        },
        _ => {
            return generic_json_err;
        },
    };
    for (label_str,clue_content) in dcs.iter() {
        let label = label_str.parse::<u32>().unwrap();
        let clue_text = match clue_content {
            serde_json::Value::String(s) => {
                s
            },
            _ => {
                return generic_json_err;
            }
        };
        puz.set_clue_text(label, puzzle_backend::EntryVariant::Down, clue_text.clone());
    }

    match &value_contents["hash_string"] {
        serde_json::Value::String(s) => {
            match s.as_str() {
                "NULL" => {
                    // Do nothing, as the puzzle is not in the final, solved state.
                },
                _ => {
                    puz.solved_hash = Some(s.parse::<u64>().unwrap());
                    puz.fill_only = true;
                }
            }
        },
        _ => {
            // For backwards compatibility, doing nothing here is ok.
        }
    };

    Ok(puz)
}