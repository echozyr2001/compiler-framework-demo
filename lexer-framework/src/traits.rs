use crate::context::LexContext;
use common_framework::Position;

/// A token produced by the lexer.
/// This is part of the CGP design, allowing tokens to be generic
/// while maintaining a consistent interface.
pub trait LexToken: Clone + std::fmt::Debug {
    /// Returns the position of this token in the source.
    fn position(&self) -> Option<Position>;

    /// Returns true if this token represents end-of-file.
    fn is_eof(&self) -> bool;

    /// Returns true if this token represents a newline.
    fn is_newline(&self) -> bool;

    /// Returns true if this token represents whitespace.
    fn is_whitespace(&self) -> bool;

    /// Returns true if this token represents indentation.
    fn is_indent(&self) -> bool;
}

/// A lexing rule that operates on a context.
/// This is the core of CGP design - rules are generic over context,
/// allowing them to work with different lexer implementations.
pub trait LexingRule<'input, Ctx, Tok>
where
    Ctx: LexContext<'input>,
{
    /// Attempts to match and consume a token from the context.
    /// Returns Some(token) if matched, None otherwise.
    /// The cursor should only be advanced if a token is successfully matched.
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<Tok>;

    /// Returns the priority of this rule. Higher priority rules are tried first.
    /// Default priority is 0.
    fn priority(&self) -> i32 {
        0
    }

    /// Quick check: returns whether this rule might match based on the first character.
    /// This is an optimization hint for the lexer to skip rules that definitely won't match.
    ///
    /// - `Some(true)`  - This rule might match (or definitely matches)
    /// - `Some(false)` - This rule definitely won't match
    /// - `None`        - Unknown, need to try full match
    ///
    /// Default implementation returns `None`, indicating full match is always needed.
    /// Rules that can quickly determine match/non-match based on first character
    /// should override this method for better performance.
    #[inline]
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        let _ = first_char; // Suppress unused parameter warning
        None
    }
}
