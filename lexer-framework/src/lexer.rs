use crate::context::{DefaultContext, LexContext};
use crate::traits::LexingRule;
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
    ///
    /// This method optimizes rule matching by:
    /// 1. Using quick_check() to skip rules that definitely won't match
    /// 2. Only creating checkpoints when actually trying a rule
    pub fn next_token(&mut self) -> Option<Tok> {
        let first_char = self.context.peek();

        for rule in &mut self.rules {
            // Quick check optimization: skip rules that definitely won't match
            if let Some(false) = rule.quick_check(first_char) {
                continue;
            }

            // Only create checkpoint if we're actually trying this rule
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
    ///
    /// Uses iterator internally for better code reuse and allows the lexer
    /// to be used as an iterator directly.
    pub fn tokenize(&mut self) -> Vec<Tok> {
        self.collect()
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

/// Make Lexer implement Iterator for stream-like processing.
/// This allows using the lexer directly in for loops and iterator chains.
impl<'input, Ctx, Tok> Iterator for Lexer<'input, Ctx, Tok>
where
    Ctx: LexContext<'input>,
{
    type Item = Tok;

    fn next(&mut self) -> Option<Self::Item> {
        if self.context.is_eof() {
            return None;
        }

        let offset_before = self.context.cursor().offset();

        if let Some(token) = self.next_token() {
            // Check if we made progress
            if self.context.cursor().offset() == offset_before {
                // No progress made, this indicates a bug in the rule
                eprintln!("Warning: No progress made at offset {}", offset_before);
                return None;
            }
            Some(token)
        } else {
            // No rule matched, check if we can make progress
            if self.context.cursor().offset() == offset_before {
                // Stuck - no rule matched and cursor didn't advance
                eprintln!(
                    "Error: No rule matched character at offset {}",
                    offset_before
                );
                eprintln!(
                    "Remaining input: {:?}",
                    self.context
                        .cursor()
                        .remaining()
                        .chars()
                        .take(10)
                        .collect::<String>()
                );
                None
            } else {
                // Progress was made but no token returned (unusual case)
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Estimate based on remaining input characters (not bytes!)
        // This is important for Unicode text (Chinese, emoji, etc.)
        //
        // Token count estimation strategy:
        // - In the worst case, each character could be a token (upper bound)
        // - In practice, tokens are often multi-character (identifiers, numbers, strings)
        // - We use character count as a conservative upper bound
        let remaining = self.context.cursor().remaining();

        // Count Unicode characters (not bytes) for accurate estimation
        let char_count = remaining.chars().count();

        // Use character count as upper bound since:
        // - Single-char tokens (operators, punctuation): 1 char = 1 token
        // - Multi-char tokens (identifiers, strings, numbers): N chars = 1 token
        // - Whitespace may or may not be tokens depending on rules
        // So char_count is a safe upper bound (most tokens span multiple chars)
        (0, Some(char_count))
    }
}
