use crate::Position;

/// A checkpoint for saving and restoring parsing/lexing state.
/// This is used by both lexer and parser frameworks to support backtracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checkpoint {
    /// The index (byte offset for lexer, token index for parser) at this checkpoint.
    index: usize,
    /// The position in the source at this checkpoint.
    position: Position,
}

impl Checkpoint {
    /// Creates a new checkpoint with the given index and position.
    pub fn new(index: usize, position: Position) -> Self {
        Self { index, position }
    }

    /// Returns the index stored in this checkpoint.
    /// For lexers, this is a byte offset.
    /// For parsers, this is a token index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the position stored in this checkpoint.
    pub fn position(&self) -> Position {
        self.position
    }

    /// Convenience method for lexer use: returns the byte offset.
    /// This is an alias for `index()` to maintain compatibility.
    pub fn current(&self) -> usize {
        self.index
    }

    /// Convenience method for parser use: returns the token index.
    /// This is an alias for `index()` to maintain compatibility.
    pub fn token_index(&self) -> usize {
        self.index
    }
}
