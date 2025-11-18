use common_framework::{Checkpoint, Position};

/// Context for parsing operations in CGP (Context-Generic Programming).
/// This trait allows parsing rules to access token stream information
/// without being tightly coupled to a specific parser implementation.
pub trait ParseContext<Tok>
where
    Tok: Clone + std::fmt::Debug,
{
    /// Returns the current token without advancing.
    fn peek(&self) -> Option<&Tok>;

    /// Returns the token at the given offset from current position without advancing.
    fn peek_at(&self, offset: usize) -> Option<&Tok>;

    /// Advances the token stream and returns the consumed token.
    fn advance(&mut self) -> Option<Tok>;

    /// Returns the current position.
    fn position(&self) -> Position;

    /// Returns true if at end of token stream.
    fn is_eof(&self) -> bool;

    /// Returns the current token index.
    fn token_index(&self) -> usize;

    /// Creates a checkpoint of the current state.
    fn checkpoint(&self) -> Checkpoint;

    /// Restores the parser to a checkpoint.
    fn restore(&mut self, checkpoint: Checkpoint);
}

/// A simple default context implementation that works with a token iterator.
#[derive(Debug)]
pub struct DefaultContext<Tok>
where
    Tok: Clone + std::fmt::Debug,
{
    tokens: Vec<Tok>,
    current: usize,
    position: Position,
}

impl<Tok> DefaultContext<Tok>
where
    Tok: Clone + std::fmt::Debug,
{
    /// Creates a new context from a vector of tokens.
    pub fn new(tokens: Vec<Tok>) -> Self {
        let position = tokens
            .first()
            .and_then(|t| {
                // Try to get position from token if it implements a position method
                // This uses a helper trait to extract position
                extract_position_from_token(t)
            })
            .unwrap_or_default();

        Self {
            tokens,
            current: 0,
            position,
        }
    }

    /// Creates a new context from an iterator of tokens.
    pub fn from_token_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Tok>,
    {
        Self::new(iter.into_iter().collect())
    }
}

/// Helper function to extract position from tokens.
/// This allows DefaultContext to work with tokens that may or may not
/// implement a position method.
///
/// Users can implement a trait for their token types to provide position information,
/// or use the lexer-framework's LexToken trait if available.
pub(crate) fn extract_position_from_token<T>(_token: &T) -> Option<Position> {
    // For now, return None - users should implement position extraction
    // for their token types, or use a helper trait
    None
}

impl<Tok> ParseContext<Tok> for DefaultContext<Tok>
where
    Tok: Clone + std::fmt::Debug,
{
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.current)
    }

    fn peek_at(&self, offset: usize) -> Option<&Tok> {
        self.tokens.get(self.current + offset)
    }

    fn advance(&mut self) -> Option<Tok> {
        if self.current >= self.tokens.len() {
            return None;
        }

        let token = self.tokens[self.current].clone();

        // Update position based on token if possible
        if let Some(new_position) = extract_position_from_token(&token) {
            self.position = new_position;
        }

        self.current += 1;
        Some(token)
    }

    fn position(&self) -> Position {
        // Try to get position from current token if available
        if let Some(token) = self.peek() {
            if let Some(token_position) = extract_position_from_token(token) {
                return token_position;
            }
        }
        self.position
    }

    fn is_eof(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn token_index(&self) -> usize {
        self.current
    }

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint::new(self.current, self.position)
    }

    fn restore(&mut self, checkpoint: Checkpoint) {
        self.current = checkpoint.token_index();
        self.position = checkpoint.position();

        // Try to update position from restored token if available
        if let Some(token) = self.peek() {
            if let Some(token_position) = extract_position_from_token(token) {
                self.position = token_position;
            }
        }
    }
}
