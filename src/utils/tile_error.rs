use std::error::Error;
use std::fmt::{Display, Formatter, Result};

type Underlying = Box<dyn Error + Send + Sync>;

#[derive(Debug)]
pub struct TileError {
    col: u32,
    row: u32,
    original_error: Underlying,
}

impl TileError {
    pub fn new(col: u32, row: u32, original_error: impl Into<Underlying>) -> Self {
        TileError {
            col,
            row,
            original_error: original_error.into(),
        }
    }
}

impl Display for TileError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Tile {}/{}: {}", self.col, self.row, self.original_error)
    }
}

impl Error for TileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.original_error)
    }
}
