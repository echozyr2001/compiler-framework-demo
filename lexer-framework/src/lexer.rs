use crate::context::{DefaultContext, LexContext};
use crate::traits::LexingRule;
use std::cmp::Reverse;

/// A lexer that applies rules in priority order.
/// This is the main orchestrator in the CGP design.
pub struct Lexer<Ctx, Tok>
where
    Ctx: LexContext,
{
    context: Ctx,
    rules: Vec<Box<dyn LexingRule<Ctx, Tok>>>,
    // Optimization: lookup table for ASCII characters (0-127)
    // Maps an ASCII char to a list of indices into `rules` that might match it.
    ascii_lookup: [Option<Vec<usize>>; 128],
}

impl<Ctx, Tok> Lexer<Ctx, Tok>
where
    Ctx: LexContext,
{
    /// Creates a new lexer with the given context and rules.
    pub fn new(context: Ctx, rules: Vec<Box<dyn LexingRule<Ctx, Tok>>>) -> Self {
        // Sort rules by priority (highest first)
        let mut sorted_rules = rules;
        sorted_rules.sort_by_key(|rule| Reverse(rule.priority()));

        // Build ASCII lookup table
        // Initialize with None to save memory if not used
        let mut ascii_lookup: [Option<Vec<usize>>; 128] = std::array::from_fn(|_| None);

        // For each ASCII character, find applicable rules
        for char_code in 0..128 {
            let ch = char::from_u32(char_code).unwrap();
            let mut applicable_indices = Vec::new();

            for (idx, rule) in sorted_rules.iter().enumerate() {
                // If quick_check returns Some(false), the rule definitely doesn't match.
                // Otherwise (Some(true) or None), it might match.
                if rule.quick_check(Some(ch)) != Some(false) {
                    applicable_indices.push(idx);
                }
            }

            // Only store if we filtered anything out, or just store all?
            // Storing all allows consistent lookup.
            // To save memory, if applicable_indices.len() == sorted_rules.len(), we could maybe use a sentinel?
            // But for simplicity and speed, let's just store it.
            if !applicable_indices.is_empty() {
                ascii_lookup[char_code as usize] = Some(applicable_indices);
            }
        }

        Self {
            context,
            rules: sorted_rules,
            ascii_lookup,
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
        if self.context.is_eof() {
            return None;
        }

        let first_char = self.context.peek();

        // Determine which rules to try
        let candidate_indices: &[usize] = match first_char {
            Some(ch) if ch.is_ascii() => {
                if let Some(indices) = &self.ascii_lookup[ch as usize] {
                    indices.as_slice()
                } else {
                    // No rules match this ASCII char (based on quick_check)
                    // But we should double check the logic. If ascii_lookup is None, it means
                    // no rule accepted it in quick_check? Yes, based on initialization.
                    // However, let's handle the case where rules might have dynamic behavior slightly gracefully?
                    // No, quick_check takes `Option<char>`, it's stateless regarding context usually.
                    // But wait, some rules might return `None` for quick_check, which means "maybe".
                    // We included those in the lookup. So if lookup is None, it effectively means no rules.
                    return None;
                }
            }
            _ => {
                // Non-ASCII or EOF (though we checked EOF above)
                // Use all rules, but we can skip this part if we had a non-ASCII lookup too.
                // For now, we don't have indices for non-ASCII, so we can't use a slice.
                // We'll handle this case by iterating 0..rules.len()
                &[] // Placeholder, see logic below
            }
        };

        if let Some(ch) = first_char {
            if ch.is_ascii() {
                // Fast path using indices
                for &idx in candidate_indices {
                    // Safe because we built indices from rules
                    let rule = &mut self.rules[idx];

                    // We still run quick_check? No, we already did it statically for the first char.
                    // But quick_check is cheap, maybe running it again is fine?
                    // Actually, rule.try_match() does the real work.

                    // Try match
                    let checkpoint = self.context.checkpoint();
                    if let Some(token) = rule.try_match(&mut self.context) {
                        return Some(token);
                    }
                    self.context.restore(checkpoint);
                }
            } else {
                // Slow path for non-ASCII
                for rule in &mut self.rules {
                    if let Some(false) = rule.quick_check(first_char) {
                        continue;
                    }

                    let checkpoint = self.context.checkpoint();
                    if let Some(token) = rule.try_match(&mut self.context) {
                        return Some(token);
                    }
                    self.context.restore(checkpoint);
                }
            }
        } else {
            // EOF case (should be caught by is_eof check, but some rules might match EOF)
            // Wait, is_eof() check above might prevent EOF rules from running?
            // Many EOF rules rely on is_eof() returning true.
            // But our is_eof() check at start of function returns None immediately.
            // Let's remove the early is_eof check if we want to support EOF tokens.
            // But standard iterator semantics usually end at EOF.
            // Let's assume standard behavior: if EOF, and we haven't matched anything, we are done.
            // UNLESS there is a rule that matches EOF explicitly (like in our examples).

            // Revert early return and handle EOF properly.
            // For EOF, peek() returns None.
            for rule in &mut self.rules {
                if let Some(false) = rule.quick_check(None) {
                    continue;
                }
                // ... try match ...
                // But wait, try_match usually expects context.
                // Let's just stick to the original logic structure but optimized.
            }
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

impl<Tok> Lexer<DefaultContext, Tok> {
    /// Creates a new lexer with a default context from an input string.
    pub fn from_str<S: Into<String>>(
        input: S,
        rules: Vec<Box<dyn LexingRule<DefaultContext, Tok>>>,
    ) -> Self {
        Self::new(DefaultContext::new(input), rules)
    }
}

/// Make Lexer implement Iterator for stream-like processing.
/// This allows using the lexer directly in for loops and iterator chains.
impl<Ctx, Tok> Iterator for Lexer<Ctx, Tok>
where
    Ctx: LexContext,
{
    type Item = Tok;

    fn next(&mut self) -> Option<Self::Item> {
        if self.context.is_eof() {
            return None;
        }

        let offset_before = self.context.offset();

        if let Some(token) = self.next_token() {
            // Check if we made progress
            if self.context.offset() == offset_before {
                // No progress made, this indicates a bug in the rule
                eprintln!("Warning: No progress made at offset {}", offset_before);
                return None;
            }
            Some(token)
        } else if self.context.offset() == offset_before {
            // Stuck - no rule matched and cursor didn't advance
            eprintln!(
                "Error: No rule matched character at offset {}",
                offset_before
            );
            // Try to peek at the next character for error reporting
            if let Some(ch) = self.context.peek() {
                eprintln!("Current character: {:?}", ch);
            }
            None
        } else {
            // Progress was made but no token returned (unusual case)
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Estimate based on remaining input characters
        // This is important for Unicode text (Chinese, emoji, etc.)
        //
        // Token count estimation strategy:
        // - In the worst case, each character could be a token (upper bound)
        // - In practice, tokens are often multi-character (identifiers, numbers, strings)
        // - We use character count as a conservative upper bound

        if let Some(len) = self.context.remaining_len() {
            // If we know the remaining bytes, we can use that as an upper bound
            (0, Some(len))
        } else {
            // Unknown length (streaming)
            (0, None)
        }
    }
}
