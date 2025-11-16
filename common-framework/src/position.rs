/// Represents the position of a token or AST node in the source text.
///
/// This is used by both the lexer and parser frameworks to track
/// the location of tokens and AST nodes in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset from the start of the input
    pub offset: usize,
}

impl Position {
    /// Creates a new position at the start of the input.
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Creates a position with the given values.
    pub fn at(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);
    }

    #[test]
    fn test_position_at() {
        let pos = Position::at(5, 10, 100);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
        assert_eq!(pos.offset, 100);
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos, Position::new());
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::at(1, 2, 3);
        let pos2 = Position::at(1, 2, 3);
        let pos3 = Position::at(1, 2, 4);
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }
}
