use crate::position::Position;

/// A cursor for traversing input text during lexing.
/// This is part of the CGP (Context-Generic Programming) design,
/// allowing rules to operate on a generic cursor interface.
#[derive(Debug, Clone)]
pub struct Cursor<'input> {
    input: &'input str,
    current: usize,
    position: Position,
}

impl<'input> Cursor<'input> {
    /// Creates a new cursor from the input string.
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            current: 0,
            position: Position::new(),
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
        self.current >= self.input.len()
    }

    /// Returns the next character without advancing the cursor.
    pub fn peek(&self) -> Option<char> {
        self.input[self.current..].chars().next()
    }

    /// Returns the next n characters without advancing the cursor.
    pub fn peek_str(&self, n: usize) -> &'input str {
        if self.is_eof() {
            return "";
        }
        let remaining = &self.input[self.current..];
        let end = remaining
            .char_indices()
            .nth(n)
            .map(|(i, _)| self.current + i)
            .unwrap_or_else(|| self.input.len());
        &self.input[self.current..end]
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
    pub fn consume_while<F>(&mut self, mut predicate: F) -> &'input str
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
        &self.input[start..self.current]
    }

    /// Returns the remaining input from the current position.
    pub fn remaining(&self) -> &'input str {
        &self.input[self.current..]
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
