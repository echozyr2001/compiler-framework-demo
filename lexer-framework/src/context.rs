use crate::cursor::Cursor;
use crate::position::Position;

/// Context for lexing operations in CGP (Context-Generic Programming).
/// This trait allows lexing rules to access contextual information
/// without being tightly coupled to a specific lexer implementation.
pub trait LexContext<'input> {
    /// Returns a reference to the cursor.
    fn cursor(&self) -> &Cursor<'input>;

    /// Returns a mutable reference to the cursor.
    fn cursor_mut(&mut self) -> &mut Cursor<'input>;

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
    fn consume_while<F>(&mut self, predicate: F) -> &'input str
    where
        F: FnMut(char) -> bool,
    {
        self.cursor_mut().consume_while(predicate)
    }

    /// Creates a checkpoint of the current state.
    fn checkpoint(&self) -> crate::cursor::Checkpoint {
        self.cursor().checkpoint()
    }

    /// Restores the cursor to a checkpoint.
    fn restore(&mut self, checkpoint: crate::cursor::Checkpoint) {
        self.cursor_mut().restore(checkpoint);
    }
}

/// A simple default context implementation.
#[derive(Debug)]
pub struct DefaultContext<'input> {
    cursor: Cursor<'input>,
}

impl<'input> DefaultContext<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }
}

impl<'input> LexContext<'input> for DefaultContext<'input> {
    fn cursor(&self) -> &Cursor<'input> {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut Cursor<'input> {
        &mut self.cursor
    }
}
