use crate::lexer::context::LexContext;
use crate::position::Position;

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
}
