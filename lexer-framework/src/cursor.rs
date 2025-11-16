use common_framework::{Position, TextSlice};
use std::sync::Arc;

/// A cursor for traversing input text during lexing.
/// This is part of the CGP (Context-Generic Programming) design,
/// allowing rules to operate on a generic cursor interface.
#[derive(Debug, Clone)]
pub struct Cursor {
    buffer: Arc<str>,
    current: usize,
    position: Position,
}

impl Cursor {
    /// Creates a new cursor from the input string.
    pub fn new<S: Into<String>>(input: S) -> Self {
        let owned = input.into();
        let buffer = Arc::<str>::from(owned);
        Self::with_arc(buffer)
    }

    /// Creates a cursor from an existing shared buffer.
    pub fn with_arc(buffer: Arc<str>) -> Self {
        Self {
            current: 0,
            position: Position::new(),
            buffer,
        }
    }

    /// Returns the current position in the source.
    pub fn position(&self) -> Position {
        self.position
    }

    /// Returns the current offset in bytes.
    pub fn offset(&self) -> usize {
        self.current
    }

    /// Returns true if the cursor is at the end of the input.
    pub fn is_eof(&self) -> bool {
        self.current >= self.buffer.len()
    }

    /// Returns the next character without advancing the cursor.
    pub fn peek(&self) -> Option<char> {
        self.buffer[self.current..].chars().next()
    }

    /// Returns the next n characters without advancing the cursor.
    pub fn peek_slice(&self, n: usize) -> TextSlice {
        if self.is_eof() {
            return TextSlice::new(self.buffer.clone(), self.current, self.current);
        }
        let remaining = &self.buffer[self.current..];
        let end = remaining
            .char_indices()
            .nth(n)
            .map(|(i, _)| self.current + i)
            .unwrap_or_else(|| self.buffer.len());
        TextSlice::new(self.buffer.clone(), self.current, end)
    }

    /// Legacy helper mirroring the previous `peek_str` API.
    pub fn peek_str(&self, n: usize) -> TextSlice {
        self.peek_slice(n)
    }

    /// Advances the cursor by one character.
    pub fn advance(&mut self) -> Option<char> {
        if self.is_eof() {
            return None;
        }

        let ch = self.peek()?;
        let len = ch.len_utf8();

        // Update position
        if ch == '\n' {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }
        self.position.offset += len;
        self.current += len;

        Some(ch)
    }

    /// Advances the cursor by n characters.
    pub fn advance_by(&mut self, n: usize) -> usize {
        let mut count = 0;
        for _ in 0..n {
            if self.advance().is_none() {
                break;
            }
            count += 1;
        }
        count
    }

    /// Consumes characters while the predicate returns true.
    pub fn consume_while<F>(&mut self, mut predicate: F) -> TextSlice
    where
        F: FnMut(char) -> bool,
    {
        let start = self.current;
        while let Some(ch) = self.peek() {
            if !predicate(ch) {
                break;
            }
            self.advance();
        }
        TextSlice::new(self.buffer.clone(), start, self.current)
    }

    /// Returns the remaining input from the current position.
    pub fn remaining(&self) -> TextSlice {
        TextSlice::new(self.buffer.clone(), self.current, self.buffer.len())
    }

    /// Resets the cursor to the beginning.
    pub fn reset(&mut self) {
        self.current = 0;
        self.position = Position::new();
    }

    /// Creates a checkpoint that can be restored later.
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            current: self.current,
            position: self.position,
        }
    }

    /// Restores the cursor to a previous checkpoint.
    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.current = checkpoint.current;
        self.position = checkpoint.position;
    }
}

/// A checkpoint for cursor position.
#[derive(Debug, Clone, Copy)]
pub struct Checkpoint {
    current: usize,
    position: Position,
}
