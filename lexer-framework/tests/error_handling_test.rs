//! Error handling and boundary scenario tests.

use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

type RuleSet<Tok> = Vec<Box<dyn LexingRule<DefaultContext, Tok>>>;

#[derive(Debug, Clone, PartialEq)]
enum TestToken {
    Char { ch: char, position: Position },
}

impl LexToken for TestToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            TestToken::Char { position, .. } => *position,
        })
    }

    fn is_eof(&self) -> bool {
        false
    }

    fn is_newline(&self) -> bool {
        false
    }

    fn is_whitespace(&self) -> bool {
        false
    }

    fn is_indent(&self) -> bool {
        false
    }
}

// Rule that never matches
struct NeverMatchRule;

impl<Ctx> LexingRule<Ctx, TestToken> for NeverMatchRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, _ctx: &mut Ctx) -> Option<TestToken> {
        None
    }

    fn priority(&self) -> i32 {
        100 // High priority but never matches
    }

    fn quick_check(&self, _first_char: Option<char>) -> Option<bool> {
        Some(true) // Says it might match, but never does
    }
}

// Rule that matches but doesn't advance cursor (buggy rule)
struct BuggyRule;

impl<Ctx> LexingRule<Ctx, TestToken> for BuggyRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        // Bug: Returns token but doesn't advance cursor
        Some(TestToken::Char {
            ch: ctx.peek()?,
            position: ctx.position(),
        })
        // Missing: ctx.advance()
    }

    fn priority(&self) -> i32 {
        50
    }
}

// Normal rule
struct NormalRule;

impl<Ctx> LexingRule<Ctx, TestToken> for NormalRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        let ch = ctx.peek()?;
        let position = ctx.position();
        ctx.advance();
        Some(TestToken::Char { ch, position })
    }

    fn priority(&self) -> i32 {
        10
    }
}

#[test]
fn test_no_rules() {
    // Empty rules list
    let rules: RuleSet<TestToken> = vec![];
    let mut lexer = Lexer::from_str("hello", rules);

    // Should return None since no rules can match
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_no_matching_rules() {
    // Rules that never match
    let rules: RuleSet<TestToken> = vec![Box::new(NeverMatchRule)];
    let mut lexer = Lexer::from_str("hello", rules);

    // Should return None
    assert_eq!(lexer.next_token(), None);

    // Iterator should handle this gracefully
    let tokens: Vec<_> = lexer.collect();
    assert!(tokens.is_empty());
}

#[test]
fn test_rule_that_doesnt_advance() {
    // Test buggy rule that doesn't advance cursor
    // The Iterator implementation should detect this and stop
    let rules: RuleSet<TestToken> = vec![Box::new(BuggyRule)];
    let mut lexer = Lexer::from_str("a", rules);

    // First call should work (but is buggy)
    let token1 = lexer.next_token();
    assert!(token1.is_some());

    // Iterator should detect no progress and stop
    let tokens: Vec<_> = lexer.take(5).collect();
    // Should stop after first token due to no progress
    assert!(tokens.is_empty() || tokens.len() == 1);
}

#[test]
fn test_multiple_quick_check_false() {
    // All rules return quick_check false
    struct AlwaysFalseRule;

    impl<Ctx> LexingRule<Ctx, TestToken> for AlwaysFalseRule
    where
        Ctx: LexContext,
    {
        fn quick_check(&self, _first_char: Option<char>) -> Option<bool> {
            Some(false) // Always says it won't match
        }

        fn try_match(&mut self, _ctx: &mut Ctx) -> Option<TestToken> {
            None
        }

        fn priority(&self) -> i32 {
            10
        }
    }

    let rules: RuleSet<TestToken> = vec![Box::new(AlwaysFalseRule), Box::new(AlwaysFalseRule)];
    let mut lexer = Lexer::from_str("a", rules);

    // All rules skipped, should return None
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_very_long_input() {
    // Test with very long input
    let long_input = "a".repeat(10000);
    let rules: RuleSet<TestToken> = vec![Box::new(NormalRule)];
    let lexer = Lexer::from_str(&long_input, rules);

    // Should handle long input correctly
    let mut count = 0;
    for _token in lexer {
        count += 1;
        if count >= 100 {
            break; // Don't process all 10000
        }
    }

    assert_eq!(count, 100);
}

#[test]
fn test_unicode_edge_cases() {
    // Test with various Unicode edge cases
    let test_cases = vec![
        ("\u{00A0}", "Non-breaking space"),
        ("\u{200D}", "Zero-width joiner"),
        ("\u{FEFF}", "Zero-width no-break space"),
        ("\u{1F3FB}", "Emoji modifier"),
    ];

    for (input, _description) in test_cases {
        let rules: RuleSet<TestToken> = vec![Box::new(NormalRule)];
        let mut lexer = Lexer::from_str(input, rules);

        // Should not panic
        let token = lexer.next_token();
        assert!(token.is_some() || token.is_none()); // Either works, just don't panic
    }
}

#[test]
fn test_position_accuracy() {
    let input = "abc\ndef\nghi";
    let rules: RuleSet<TestToken> = vec![Box::new(NormalRule)];
    let lexer = Lexer::from_str(input, rules);

    let tokens: Vec<_> = lexer.take(10).collect();

    // Find tokens by their character to verify positions
    // "abc\ndef\nghi" = a(1,1), b(1,2), c(1,3), \n(1,4->2,1), d(2,1), e(2,2), f(2,3), \n(2,4->3,1), g(3,1), h(3,2), i(3,3)

    // Find 'a' (first char, line 1, col 1)
    let token_a = tokens
        .iter()
        .find(|t| matches!(t, TestToken::Char { ch: 'a', .. }));
    assert!(token_a.is_some());
    assert_eq!(token_a.unwrap().position().unwrap().line, 1);
    assert_eq!(token_a.unwrap().position().unwrap().column, 1);

    // Find 'd' (first char of line 2)
    let token_d = tokens
        .iter()
        .find(|t| matches!(t, TestToken::Char { ch: 'd', .. }));
    assert!(token_d.is_some());
    assert_eq!(token_d.unwrap().position().unwrap().line, 2);
    assert_eq!(token_d.unwrap().position().unwrap().column, 1);

    // Find 'g' (first char of line 3)
    let token_g = tokens
        .iter()
        .find(|t| matches!(t, TestToken::Char { ch: 'g', .. }));
    assert!(token_g.is_some());
    assert_eq!(token_g.unwrap().position().unwrap().line, 3);
    assert_eq!(token_g.unwrap().position().unwrap().column, 1);
}

#[test]
fn test_checkpoint_nested_restore() {
    // Test that checkpoint/restore works correctly even with nested attempts
    let mut ctx = DefaultContext::new("hello");

    let checkpoint1 = ctx.checkpoint();
    ctx.advance(); // 'h'

    let checkpoint2 = ctx.checkpoint();
    ctx.advance(); // 'e'

    ctx.restore(checkpoint2);
    assert_eq!(ctx.peek(), Some('e'));

    ctx.restore(checkpoint1);
    assert_eq!(ctx.peek(), Some('h'));
}

#[test]
fn test_quick_check_with_eof() {
    // Test quick_check behavior with EOF
    struct EofAwareRule;

    impl<Ctx> LexingRule<Ctx, TestToken> for EofAwareRule
    where
        Ctx: LexContext,
    {
        fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
            // When EOF (None), return None (unknown)
            first_char.map(|_| true)
        }

        fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
            if let Some(ch) = ctx.peek() {
                let position = ctx.position();
                ctx.advance();
                Some(TestToken::Char { ch, position })
            } else {
                None
            }
        }

        fn priority(&self) -> i32 {
            10
        }
    }

    // Test with non-empty input
    let rules1: RuleSet<TestToken> = vec![Box::new(EofAwareRule)];
    let mut lexer1 = Lexer::from_str("a", rules1);
    assert!(lexer1.next_token().is_some());

    // Test with empty input (EOF)
    let rules2: RuleSet<TestToken> = vec![Box::new(EofAwareRule)];
    let mut lexer2 = Lexer::from_str("", rules2);
    assert_eq!(lexer2.next_token(), None);
}

#[test]
fn test_size_hint_updates() {
    let rules: RuleSet<TestToken> = vec![Box::new(NormalRule)];
    let mut lexer = Lexer::from_str("hello world", rules);

    let (lower1, upper1) = lexer.size_hint();
    assert_eq!(lower1, 0);
    assert_eq!(upper1, Some(11)); // 11 characters

    lexer.next(); // Consume one token

    let (lower2, upper2) = lexer.size_hint();
    assert_eq!(lower2, 0);
    assert!(upper2.is_some());
    assert!(upper2.unwrap() < 11); // Should be less after consuming
}
