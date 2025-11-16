use lexer_framework::{
    DefaultContext, LexContext, Lexer, LexingRule, LexToken, Position,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestToken {
    Digit { value: char, position: Position },
    Letter { value: char, position: Position },
    Other { value: char, position: Position },
}

impl LexToken for TestToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            TestToken::Digit { position, .. }
            | TestToken::Letter { position, .. }
            | TestToken::Other { position, .. } => *position,
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

// Rule with quick_check that returns Some(false) for non-digits
struct DigitRule;

impl<'input, Ctx> LexingRule<'input, Ctx, TestToken> for DigitRule
where
    Ctx: LexContext<'input>,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '0'..='9' => Some(true),
            _ => Some(false), // Definitely won't match
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        let ch = ctx.peek()?;
        if ch.is_ascii_digit() {
            let position = ctx.position();
            ctx.advance();
            Some(TestToken::Digit { value: ch, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10
    }
}

// Rule with quick_check that returns Some(true) for letters
struct LetterRule;

impl<'input, Ctx> LexingRule<'input, Ctx, TestToken> for LetterRule
where
    Ctx: LexContext<'input>,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            'a'..='z' | 'A'..='Z' => Some(true),
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        let ch = ctx.peek()?;
        if ch.is_alphabetic() {
            let position = ctx.position();
            ctx.advance();
            Some(TestToken::Letter { value: ch, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        5
    }
}

// Rule without quick_check (default None)
struct OtherRule;

impl<'input, Ctx> LexingRule<'input, Ctx, TestToken> for OtherRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        let ch = ctx.peek()?;
        if !ch.is_alphanumeric() {
            let position = ctx.position();
            ctx.advance();
            Some(TestToken::Other { value: ch, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        1
    }
    // No quick_check - uses default None
}

#[test]
fn test_quick_check_skips_non_matching() {
    // DigitRule should be skipped for 'a' via quick_check
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(DigitRule), Box::new(LetterRule)];
    let mut lexer = Lexer::from_str("a", rules);
    
    let token = lexer.next_token();
    assert_eq!(
        token,
        Some(TestToken::Letter {
            value: 'a',
            position: Position::new()
        })
    );
}

#[test]
fn test_quick_check_allows_matching() {
    // DigitRule should match '5' via quick_check
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(DigitRule), Box::new(LetterRule)];
    let mut lexer = Lexer::from_str("5", rules);
    
    let token = lexer.next_token();
    assert_eq!(
        token,
        Some(TestToken::Digit {
            value: '5',
            position: Position::new()
        })
    );
}

#[test]
fn test_quick_check_none_always_tries() {
    // OtherRule has no quick_check, should always try
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(DigitRule), Box::new(LetterRule), Box::new(OtherRule)];
    let mut lexer = Lexer::from_str("!", rules);
    
    // DigitRule and LetterRule should be skipped via quick_check
    // OtherRule should match
    let token = lexer.next_token();
    assert_eq!(
        token,
        Some(TestToken::Other {
            value: '!',
            position: Position::new()
        })
    );
}

#[test]
fn test_quick_check_mixed_input() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(DigitRule), Box::new(LetterRule), Box::new(OtherRule)];
    let lexer = Lexer::from_str("a5!", rules);
    
    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 3);
    assert_eq!(
        tokens[0],
        TestToken::Letter {
            value: 'a',
            position: Position::new()
        }
    );
    assert_eq!(
        tokens[1],
        TestToken::Digit {
            value: '5',
            position: Position { line: 1, column: 2, offset: 1 }
        }
    );
    assert_eq!(
        tokens[2],
        TestToken::Other {
            value: '!',
            position: Position { line: 1, column: 3, offset: 2 }
        }
    );
}

#[test]
fn test_quick_check_eof() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(DigitRule)];
    let mut lexer = Lexer::from_str("", rules);
    
    // quick_check with None (EOF) should return None (unknown)
    // So it should try the rule, which will fail, then return None
    let token = lexer.next_token();
    assert_eq!(token, None);
}

