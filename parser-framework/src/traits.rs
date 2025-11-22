use crate::context::ParseContext;
use common_framework::Position;

/// An AST node produced by the parser.
/// This is part of the CGP design, allowing AST nodes to be generic
/// while maintaining a consistent interface.
pub trait AstNode: Clone + std::fmt::Debug {
    /// Returns the position of this AST node in the source.
    fn position(&self) -> Option<Position>;

    /// Returns the span (start and end positions) of this AST node.
    fn span(&self) -> Option<(Position, Position)> {
        self.position().map(|pos| (pos, pos))
    }
}

/// A trait for AST nodes that can carry arbitrary state information.
/// This allows nodes to be annotated with user-defined state (e.g., Incomplete/Complete
/// for editor scenarios, error recovery state for compilers, etc.) without the framework
/// needing to know the specific state type.
///
/// This is an optional extension - not all AST nodes need to be stateful.
pub trait StatefulNode: AstNode {
    /// The type of state this node carries.
    type State: Clone + std::fmt::Debug;

    /// Returns the current state of this node.
    fn state(&self) -> &Self::State;

    /// Sets the state of this node.
    fn set_state(&mut self, state: Self::State);

    /// Attempts to transition state based on a trigger.
    /// Returns true if the state was changed, false otherwise.
    ///
    /// Default implementation does nothing and returns false.
    /// Implementations can override this to implement state machine logic.
    fn transition(&mut self, _trigger: &dyn std::any::Any) -> bool {
        false
    }
}

/// A parsing rule that operates on a context.
/// This is the core of CGP design - rules are generic over context,
/// allowing them to work with different parser implementations.
pub trait ParsingRule<Ctx, Tok, Ast>
where
    Ctx: ParseContext<Tok>,
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    /// Attempts to match and parse an AST node from the context.
    /// Returns Some(node) if matched, None otherwise.
    /// The token stream should only be advanced if a node is successfully parsed.
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Ast>;

    /// Returns the priority of this rule. Higher priority rules are tried first.
    /// Default priority is 0.
    fn priority(&self) -> i32 {
        0
    }

    /// Quick check: returns whether this rule might match based on the current token.
    /// This is an optimization hint for the parser to skip rules that definitely won't match.
    ///
    /// - `Some(true)`  - This rule might match (or definitely matches)
    /// - `Some(false)` - This rule definitely won't match
    /// - `None`        - Unknown, need to try full parse
    ///
    /// Default implementation returns `None`, indicating full parse is always needed.
    /// Rules that can quickly determine match/non-match based on current token
    /// should override this method for better performance.
    #[inline]
    fn quick_check(&self, current_token: Option<&Tok>) -> Option<bool> {
        let _ = current_token; // Suppress unused parameter warning
        None
    }
}
