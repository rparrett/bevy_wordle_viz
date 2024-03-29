use std::str::FromStr;
use thiserror::Error;

#[derive(Debug)]
pub struct WordleGuess {
    /// Supports another guess against gravity
    pub support: bool,
    /// If someone biffs an entire row, we want to indicate that visually without
    /// actually filling in every "black box."
    ///
    /// This field marks `NotInWord` guesses that should be displayed to accomplish
    /// this.
    pub topper: bool,
    pub kind: WordleGuessKind,
}
impl Default for WordleGuess {
    fn default() -> Self {
        Self {
            support: false,
            topper: false,
            kind: WordleGuessKind::NotInWord,
        }
    }
}

#[derive(Debug)]
pub enum WordleGuessKind {
    Correct,
    InWord,
    NotInWord,
}
impl TryFrom<char> for WordleGuessKind {
    type Error = WordleParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            BLACK | WHITE => Ok(Self::NotInWord),
            YELLOW => Ok(Self::InWord),
            GREEN => Ok(Self::Correct),
            _ => Err(Self::Error::InvalidCharacter),
        }
    }
}

pub struct WordleGrid {
    /// Optional "puzzle number" that the user may or may not have pasted in.
    pub number: Option<u32>,
    pub guesses: Vec<Vec<WordleGuess>>,
}

#[derive(Error, Debug)]
pub enum WordleParseError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("invalid character")]
    InvalidCharacter,
}

const BLACK: char = '⬛';
const WHITE: char = '⬜';
const GREEN: char = '🟩';
const YELLOW: char = '🟨';
const CB_GREEN: char = '🟧';
const CB_YELLOW: char = '🟦';

const VALID: [char; 6] = [BLACK, WHITE, GREEN, YELLOW, CB_GREEN, CB_YELLOW];

impl FromStr for WordleGrid {
    type Err = WordleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = vec![];
        let mut number = None;
        let mut prev_num_cols = None;

        // slack
        let s = s.replace(":black_large_square:", &BLACK.to_string());
        let s = s.replace(":white_large_square:", &BLACK.to_string());
        let s = s.replace(":large_green_square:", &GREEN.to_string());
        let s = s.replace(":large_orange_square:", &GREEN.to_string());
        let s = s.replace(":large_yellow_square:", &YELLOW.to_string());
        let s = s.replace(":large_blue_square:", &YELLOW.to_string());
        // twitter
        let s = s.replace("Black large square", &BLACK.to_string());
        let s = s.replace("Green square", &GREEN.to_string());
        let s = s.replace("Orange square", &GREEN.to_string());
        let s = s.replace("Yellow square", &YELLOW.to_string());
        let s = s.replace("Blue square", &YELLOW.to_string());

        for line in s.lines().rev() {
            if line.starts_with("Wordle") {
                if let Some(num) = line
                    .split(' ')
                    .nth(1)
                    .and_then(|num_str| num_str.parse::<u32>().ok())
                {
                    number = Some(num);
                }
            };

            let num_cols = line.chars().count();

            if num_cols == 0 {
                continue;
            }

            if line.chars().any(|c| !VALID.contains(&c)) {
                continue;
            }

            if let Some(prev_num_cols) = prev_num_cols {
                if num_cols != prev_num_cols {
                    return Err(WordleParseError::InvalidFormat);
                }
            }

            prev_num_cols = Some(num_cols);

            grid.push(
                line.chars()
                    .map(|c| WordleGuess {
                        // unwrap: gate above protects us
                        kind: WordleGuessKind::try_from(c).unwrap(),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            )
        }

        if grid.is_empty() {
            return Err(WordleParseError::InvalidFormat);
        }

        // Find blocks that are needed to support other blocks against the force of gravity

        for col in 0..grid[0].len() {
            let mut needs_support = false;

            for row in grid.iter_mut().rev() {
                match row[col].kind {
                    WordleGuessKind::InWord | WordleGuessKind::Correct => needs_support = true,
                    _ if needs_support => row[col].support = true,
                    _ => {}
                }
            }
        }

        // We want to make sure that puzzles with entirely incorrect first guesses are
        // distinguishable.
        //
        // Find the tallest stacks of guesses that are not `NotInWord`. From the top,
        // Mark `NotInWord` in those columns with the `topper` field.

        if grid
            .last()
            .unwrap()
            .iter()
            .all(|g| matches!(g.kind, WordleGuessKind::NotInWord))
        {
            let mut depths: Vec<usize> = vec![std::usize::MAX; grid[0].len()];

            for (col, depth) in depths.iter_mut().enumerate() {
                let mut num: usize = 0;
                for row in (0..grid.len()).rev() {
                    match grid[row][col].kind {
                        WordleGuessKind::NotInWord => {
                            num += 1;
                        }
                        _ => break,
                    }
                }
                if *depth > num {
                    *depth = num;
                }
            }

            let min = depths.iter().min().unwrap();

            for (col, depth) in depths.iter().enumerate() {
                if *depth != *min {
                    continue;
                }

                for row in (0..grid.len()).rev() {
                    match grid[row][col].kind {
                        WordleGuessKind::NotInWord => {
                            grid[row][col].topper = true;
                        }
                        _ => break,
                    }
                }
            }
        }

        Ok(WordleGrid {
            number,
            guesses: grid,
        })
    }
}

impl WordleGrid {
    /// Returns the number of rows in the puzzle grid
    pub fn rows(&self) -> usize {
        self.guesses.len()
    }

    /// Returns the number of columns in the puzzle grid
    pub fn columns(&self) -> usize {
        self.guesses.first().map_or(0, |row| row.len())
    }

    /// Iterate the grid from the bottom in a snake-like manner, initially
    /// left to right.
    ///
    /// 678
    /// 543
    /// 012
    pub fn snake_iter(&self) -> WordleGridSnakeIterator {
        WordleGridSnakeIterator {
            grid: self,
            index: 0,
        }
    }
}

#[derive(Clone)]
pub struct WordleGridSnakeIterator<'a> {
    grid: &'a WordleGrid,
    index: usize,
}

impl<'a> Iterator for WordleGridSnakeIterator<'a> {
    type Item = (usize, usize, &'a WordleGuess);
    fn next(&mut self) -> Option<Self::Item> {
        let rows = self.grid.rows();
        let columns = self.grid.columns();

        if rows == 0 || columns == 0 {
            return None;
        }

        if self.index >= columns * rows {
            return None;
        }

        let row = self.index / columns;
        let col = if row % 2 == 0 {
            self.index % columns
        } else {
            columns - 1 - self.index % columns
        };

        self.index += 1;

        Some((row, col, &self.grid.guesses[row][col]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let share = "Wordle 218 3/6
⬛⬛⬛⬛⬛
⬛🟩⬛⬛🟨
🟨🟩⬛⬛⬛
🟩🟩🟩🟩🟩";

        let grid = WordleGrid::from_str(share);

        assert!(grid.is_ok());

        let grid = grid.unwrap();

        assert_eq!(grid.number, Some(218));

        let snake = grid.snake_iter();

        let coords = snake
            .clone()
            .map(|(row, col, _)| (row, col))
            .collect::<Vec<_>>();

        assert_eq!(
            coords,
            vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 4),
                (1, 3),
                (1, 2),
                (1, 1),
                (1, 0),
                (2, 0),
                (2, 1),
                (2, 2),
                (2, 3),
                (2, 4),
                (3, 4),
                (3, 3),
                (3, 2),
                (3, 1),
                (3, 0),
            ]
        );

        let toppers = snake
            .clone()
            .filter(|(_, _, guess)| guess.topper)
            .map(|(row, col, _)| (row, col))
            .collect::<Vec<_>>();

        assert_eq!(toppers, vec![(3, 4), (3, 1)])
    }
}
