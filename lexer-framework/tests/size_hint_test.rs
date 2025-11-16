use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

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

struct CharRule;

impl<Ctx> LexingRule<Ctx, TestToken> for CharRule
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
fn test_size_hint_ascii() {
    let input = "hello world";
    let rules: RuleSet<TestToken> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str(input, rules);

    let (lower, upper) = lexer.size_hint();
    // For ASCII: 1 char = 1 byte
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(11)); // 11 characters
}

#[test]
fn test_size_hint_chinese() {
    let input = "你好世界";
    let rules: RuleSet<TestToken> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str(input, rules);

    let (lower, upper) = lexer.size_hint();
    // Chinese characters: 3 bytes each, but 1 char = 1 token
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(4)); // 4 characters, not 12 bytes
}

#[test]
fn test_size_hint_emoji() {
    let input = "😀🎉🚀";
    let rules: RuleSet<TestToken> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str(input, rules);

    let (lower, upper) = lexer.size_hint();
    // Emoji: 4 bytes each, but 1 char = 1 token
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(3)); // 3 characters, not 12 bytes
}

#[test]
fn test_size_hint_emoji_combination() {
    // Test emoji with zero-width joiners (ZWJ) - family emoji
    // 👨‍👩‍👧‍👦 = 1 visual character, but 7 Unicode scalar values
    let input = "👨‍👩‍👧‍👦";
    let rules: RuleSet<TestToken> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str(input, rules);

    let (lower, upper) = lexer.size_hint();
    assert_eq!(lower, 0);

    // We use chars().count() which counts Unicode scalar values (7)
    // This is slightly higher than visual characters, but still a valid upper bound
    assert_eq!(upper, Some(7));
}

#[test]
fn test_size_hint_accented_chars() {
    // Test accented characters: é can be 1 character (composed) or 2 chars (decomposed)
    let input = "café résumé";
    let rules: RuleSet<TestToken> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str(input, rules);

    let (lower, upper) = lexer.size_hint();
    assert_eq!(lower, 0);

    // The exact count depends on Unicode normalization, but it's a valid upper bound
    assert!(upper.is_some());
    assert!(upper.unwrap() >= 10); // At least 10 characters
}

#[test]
fn test_size_hint_mixed() {
    let input = "Hello 你好 😀!";
    let rules: RuleSet<TestToken> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str(input, rules);

    let (lower, upper) = lexer.size_hint();
    // Mixed: ASCII + Chinese + Emoji
    // "Hello 你好 😀!" = 5 + 1 + 2 + 1 + 1 + 1 = 11 characters
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(11)); // 11 characters total
}
