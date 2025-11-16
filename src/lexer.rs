mod context;
mod cursor;
mod rules;
mod traits;

pub use context::{DefaultContext, LexContext};
pub use cursor::{Checkpoint, Cursor};
pub use rules::{
    EofRule, IdentRule, NewlineRule, NumberRule, OperatorRule, SimpleToken, WhitespaceRule,
    default_rules,
};
pub use traits::{LexToken, LexingRule};

use std::cmp::Reverse;

/// A lexer that applies rules in priority order.
/// This is the main orchestrator in the CGP design.
pub struct Lexer<'input, Ctx, Tok>
where
    Ctx: LexContext<'input>,
{
    context: Ctx,
    rules: Vec<Box<dyn LexingRule<'input, Ctx, Tok> + 'input>>,
}

impl<'input, Ctx, Tok> Lexer<'input, Ctx, Tok>
where
    Ctx: LexContext<'input>,
{
    /// Creates a new lexer with the given context and rules.
    pub fn new(context: Ctx, rules: Vec<Box<dyn LexingRule<'input, Ctx, Tok> + 'input>>) -> Self {
        // Sort rules by priority (highest first)
        let mut sorted_rules = rules;
        sorted_rules.sort_by_key(|rule| Reverse(rule.priority()));

        Self {
            context,
            rules: sorted_rules,
        }
    }

    /// Returns a reference to the context.
    pub fn context(&self) -> &Ctx {
        &self.context
    }

    /// Returns a mutable reference to the context.
    pub fn context_mut(&mut self) -> &mut Ctx {
        &mut self.context
    }

    /// Tries to match the next token using the rules.
    pub fn next_token(&mut self) -> Option<Tok> {
        for rule in &mut self.rules {
            let checkpoint = self.context.checkpoint();
            if let Some(token) = rule.try_match(&mut self.context) {
                return Some(token);
            }
            // If rule didn't match, restore cursor
            self.context.restore(checkpoint);
        }
        None
    }

    /// Collects all tokens from the input.
    pub fn tokenize(&mut self) -> Vec<Tok> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}

impl<'input, Tok> Lexer<'input, DefaultContext<'input>, Tok> {
    /// Creates a new lexer with a default context from an input string.
    pub fn from_str(
        input: &'input str,
        rules: Vec<Box<dyn LexingRule<'input, DefaultContext<'input>, Tok> + 'input>>,
    ) -> Self {
        Self::new(DefaultContext::new(input), rules)
    }
}
