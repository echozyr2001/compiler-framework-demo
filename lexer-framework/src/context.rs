use crate::cursor::Cursor;
use common_framework::{Checkpoint, Position, TextSlice};

/// Context for lexing operations in CGP (Context-Generic Programming).
/// This trait allows lexing rules to access contextual information
/// without being tightly coupled to a specific lexer implementation.
pub trait LexContext {
    /// Returns a reference to the cursor.
    fn cursor(&self) -> &Cursor;

    /// Returns a mutable reference to the cursor.
    fn cursor_mut(&mut self) -> &mut Cursor;

    /// Returns the current position.
    fn position(&self) -> Position {
        self.cursor().position()
    }

    /// Returns true if at end of input.
    fn is_eof(&self) -> bool {
        self.cursor().is_eof()
    }

    /// Peeks at the next character without advancing.
    fn peek(&self) -> Option<char> {
        self.cursor().peek()
    }

    /// Advances the cursor and returns the character.
    fn advance(&mut self) -> Option<char> {
        self.cursor_mut().advance()
    }

    /// Consumes characters while the predicate returns true.
    fn consume_while<F>(&mut self, predicate: F) -> TextSlice
    where
        F: FnMut(char) -> bool,
    {
        self.cursor_mut().consume_while(predicate)
    }

    /// Creates a checkpoint of the current state.
    fn checkpoint(&self) -> Checkpoint {
        self.cursor().checkpoint()
    }

    /// Restores the cursor to a checkpoint.
    fn restore(&mut self, checkpoint: Checkpoint) {
        self.cursor_mut().restore(checkpoint);
    }

    /// Returns the current byte offset in the input.
    /// This is a convenience method that avoids direct cursor access.
    fn offset(&self) -> usize {
        self.cursor().offset()
    }
}

/// A simple default context implementation.
#[derive(Debug)]
pub struct DefaultContext {
    cursor: Cursor,
}

impl DefaultContext {
    pub fn new<S: Into<String>>(input: S) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }

    pub fn from_cursor(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

impl LexContext for DefaultContext {
    fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }
}
