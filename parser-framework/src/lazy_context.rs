use crate::context::ParseContext;
use common_framework::{Checkpoint, Position};
use std::collections::VecDeque;

/// A parsing context that lazily consumes tokens from an iterator.
///
/// It maintains a sliding window buffer to support limited lookahead and backtracking.
/// Tokens are pulled from the iterator on demand.
/// Old tokens are discarded when they fall out of the sliding window.
pub struct LazyContext<I, Tok>
where
    I: Iterator<Item = Tok>,
    Tok: Clone + std::fmt::Debug,
{
    iter: I,
    buffer: VecDeque<Tok>,
    /// The global token index of the first element in the buffer
    base_index: usize,
    /// The current position of the parser relative to the start of the buffer
    cursor_offset: usize,
    /// Current position information
    position: Position,
    /// Maximum size of the history window before pruning
    window_size: usize,
    /// Tokens with index < committed_index will never be revisited.
    committed_index: usize,
}

impl<I, Tok> LazyContext<I, Tok>
where
    I: Iterator<Item = Tok>,
    Tok: Clone + std::fmt::Debug,
{
    pub fn new(iter: I, window_size: usize) -> Self {
        Self {
            iter,
            buffer: VecDeque::with_capacity(window_size),
            base_index: 0,
            cursor_offset: 0,
            position: Position::default(),
            window_size,
            committed_index: 0,
        }
    }

    /// Ensures that the buffer contains the token at the given relative offset.
    /// Returns false if EOF is reached.
    fn ensure_buffer(&mut self, relative_offset: usize) -> bool {
        while self.cursor_offset + relative_offset >= self.buffer.len() {
            if let Some(token) = self.iter.next() {
                self.buffer.push_back(token);
            } else {
                return false;
            }
        }
        true
    }

    /// Prunes the buffer if the cursor has advanced far enough.
    fn maybe_prune(&mut self) {
        // First drop everything below committed_index
        while self.base_index < self.committed_index {
            self.buffer.pop_front();
            self.base_index += 1;
            if self.cursor_offset > 0 {
                self.cursor_offset -= 1;
            }
        }

        // Keep at least half the window size as history relative to cursor
        let keep_history = self.window_size / 2;
        if self.cursor_offset > keep_history {
            let prune_count = self.cursor_offset - keep_history;
            for _ in 0..prune_count {
                self.buffer.pop_front();
            }
            self.base_index += prune_count;
            self.cursor_offset -= prune_count;
        }
    }
}

impl<I, Tok> ParseContext<Tok> for LazyContext<I, Tok>
where
    I: Iterator<Item = Tok>,
    Tok: Clone + std::fmt::Debug,
{
    fn peek(&mut self) -> Option<&Tok> {
        self.peek_at(0)
    }

    fn peek_at(&mut self, offset: usize) -> Option<&Tok> {
        if self.ensure_buffer(offset) {
            // buffer must have element at cursor_offset + offset
            // but we need to return a reference.
            // Since we hold &mut self, we can return &Tok.
            // However, we cannot return reference to buffer while mutably borrowing self later?
            // Wait, the signature is `fn peek_at(&mut self, offset: usize) -> Option<&Tok>`.
            // This means the returned reference borrows `self`.
            // So user cannot call `advance` while holding the reference. This is fine.
            self.buffer.get(self.cursor_offset + offset)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<Tok> {
        // Ensure we have current token
        self.ensure_buffer(0);

        if self.cursor_offset < self.buffer.len() {
            let token = self.buffer[self.cursor_offset].clone();

            // Update internal state
            self.cursor_offset += 1;

            // Try to update position (if Token supported it, but here we don't know Token type details easily unless we bound it)
            // For now, simply return.

            self.maybe_prune();
            Some(token)
        } else {
            None
        }
    }

    fn position(&self) -> Position {
        self.position
    }

    fn is_eof(&mut self) -> bool {
        // If we have tokens in buffer at cursor, not EOF
        if self.cursor_offset < self.buffer.len() {
            return false;
        }
        // Try to fetch one more
        !self.ensure_buffer(0)
    }

    fn token_index(&self) -> usize {
        self.base_index + self.cursor_offset
    }

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint::new(self.token_index(), self.position)
    }

    fn restore(&mut self, checkpoint: Checkpoint) {
        let target_index = checkpoint.token_index();
        if target_index < self.base_index {
            panic!(
                "LazyContext: Backtracking too far! Target {}, current base {}",
                target_index, self.base_index
            );
        }
        let new_offset = target_index - self.base_index;
        if new_offset > self.buffer.len() {
            // This shouldn't happen if checkpoint was valid and we haven't discarded future?
            // We only discard past.
            panic!("LazyContext: Invalid future restore?");
        }
        self.cursor_offset = new_offset;
        self.position = checkpoint.position();
    }

    fn commit(&mut self) {
        let current_index = self.token_index();
        if current_index > self.committed_index {
            self.committed_index = current_index;
        }
        self.maybe_prune();
    }
}
