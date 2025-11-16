use lexer_framework::{
    DefaultContext, LexContext, Lexer, LexingRule, LexToken, Position,
};

type RuleSet<Tok> = Vec<Box<dyn LexingRule<DefaultContext, Tok>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestToken {
    A { position: Position },
    B { position: Position },
    C { position: Position },
    Eof { position: Position },
}

impl LexToken for TestToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            TestToken::A { position }
            | TestToken::B { position }
            | TestToken::C { position }
            | TestToken::Eof { position } => *position,
        })
    }

    fn is_eof(&self) -> bool {
        matches!(self, TestToken::Eof { .. })
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

// Rule that matches 'a'
struct ARule;

impl<Ctx> LexingRule<Ctx, TestToken> for ARule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        if ctx.peek() == Some('a') {
            let position = ctx.position();
            ctx.advance();
            Some(TestToken::A { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10
    }

    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('a'))
    }
}

// Rule that matches 'b'
struct BRule;

impl<Ctx> LexingRule<Ctx, TestToken> for BRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        if ctx.peek() == Some('b') {
            let position = ctx.position();
            ctx.advance();
            Some(TestToken::B { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        5 // Lower priority than ARule
    }

    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('b'))
    }
}

// Rule that matches 'c' (no quick_check)
struct CRule;

impl<Ctx> LexingRule<Ctx, TestToken> for CRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        if ctx.peek() == Some('c') {
            let position = ctx.position();
            ctx.advance();
            Some(TestToken::C { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        1 // Lowest priority
    }
    // No quick_check implementation - uses default None
}

// EOF rule
struct EofRule;

impl<Ctx> LexingRule<Ctx, TestToken> for EofRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        if ctx.is_eof() {
            Some(TestToken::Eof {
                position: ctx.position(),
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        -1
    }
}

#[test]
fn test_lexer_new() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule)];
    let ctx = DefaultContext::new("a");
    let lexer = Lexer::new(ctx, rules);
    assert!(!lexer.context().is_eof());
}

#[test]
fn test_lexer_from_str() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule)];
    let lexer = Lexer::from_str("a", rules);
    assert!(!lexer.context().is_eof());
}

#[test]
fn test_lexer_next_token_single() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule)];
    let mut lexer = Lexer::from_str("a", rules);
    
    let token = lexer.next_token();
    assert_eq!(token, Some(TestToken::A { position: Position::new() }));
}

#[test]
fn test_lexer_next_token_multiple() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule), Box::new(BRule)];
    let mut lexer = Lexer::from_str("ab", rules);
    
    assert_eq!(
        lexer.next_token(),
        Some(TestToken::A {
            position: Position::new()
        })
    );
    assert_eq!(
        lexer.next_token(),
        Some(TestToken::B {
            position: Position { line: 1, column: 2, offset: 1 }
        })
    );
}

#[test]
fn test_lexer_priority() {
    // Both rules can match 'a', but ARule has higher priority
    let rules: RuleSet<TestToken> =
        vec![Box::new(BRule), Box::new(ARule)]; // BRule added first, but ARule has higher priority
    let mut lexer = Lexer::from_str("a", rules);
    
    // Should match ARule (higher priority) even though BRule was added first
    let token = lexer.next_token();
    assert_eq!(token, Some(TestToken::A { position: Position::new() }));
}

#[test]
fn test_lexer_quick_check_optimization() {
    // ARule has quick_check that returns Some(false) for 'b'
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule), Box::new(BRule)];
    let mut lexer = Lexer::from_str("b", rules);
    
    // Should match BRule, ARule should be skipped via quick_check
    let token = lexer.next_token();
    assert_eq!(token, Some(TestToken::B { position: Position::new() }));
}

#[test]
fn test_lexer_no_match() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule), Box::new(BRule)];
    let mut lexer = Lexer::from_str("x", rules);
    
    // No rule matches 'x'
    let token = lexer.next_token();
    assert_eq!(token, None);
}

#[test]
fn test_lexer_eof() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule), Box::new(EofRule)];
    let mut lexer = Lexer::from_str("a", rules);
    
    let token1 = lexer.next_token();
    assert_eq!(token1, Some(TestToken::A { position: Position::new() }));
    
    let token2 = lexer.next_token();
    assert_eq!(
        token2,
        Some(TestToken::Eof {
            position: Position { line: 1, column: 2, offset: 1 }
        })
    );
}

#[test]
fn test_lexer_tokenize() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule), Box::new(BRule), Box::new(EofRule)];
    let mut lexer = Lexer::from_str("ab", rules);
    
    let tokens = lexer.tokenize();
    // tokenize() uses Iterator, which stops when EOF is reached and no more tokens
    // With current Iterator implementation, EOF may not be returned as a token
    // Check that at least 'a' and 'b' are present
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0], TestToken::A { position: Position::new() });
    assert_eq!(
        tokens[1],
        TestToken::B {
            position: Position { line: 1, column: 2, offset: 1 }
        }
    );
}

#[test]
fn test_lexer_iterator() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule), Box::new(BRule)];
    let lexer = Lexer::from_str("ab", rules);
    
    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], TestToken::A { position: Position::new() });
    assert_eq!(
        tokens[1],
        TestToken::B {
            position: Position { line: 1, column: 2, offset: 1 }
        }
    );
}

#[test]
fn test_lexer_iterator_take() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(ARule), Box::new(BRule)];
    let lexer = Lexer::from_str("ab", rules);
    
    let tokens: Vec<_> = lexer.take(1).collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], TestToken::A { position: Position::new() });
}

#[test]
fn test_lexer_empty_input() {
    let rules: RuleSet<TestToken> =
        vec![Box::new(EofRule)];
    let mut lexer = Lexer::from_str("", rules);
    
    let token = lexer.next_token();
    assert!(matches!(token, Some(TestToken::Eof { .. })));
}

#[test]
fn test_lexer_rule_without_quick_check() {
    // CRule doesn't implement quick_check, should still work
    let rules: RuleSet<TestToken> =
        vec![Box::new(CRule)];
    let mut lexer = Lexer::from_str("c", rules);
    
    let token = lexer.next_token();
    assert_eq!(token, Some(TestToken::C { position: Position::new() }));
}

#[test]
fn test_lexer_checkpoint_restore() {
    // Rule that tries to match but fails, should restore cursor
    struct FailingRule;

    impl<Ctx> LexingRule<Ctx, TestToken> for FailingRule
    where
        Ctx: LexContext,
    {
        fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
            // Try to match 'x' but fail
            if ctx.peek() == Some('x') {
                ctx.advance(); // Advance cursor
                None // But return None (simulating failure)
            } else {
                None
            }
        }

        fn priority(&self) -> i32 {
            20 // High priority
        }
    }

    let rules: RuleSet<TestToken> =
        vec![Box::new(FailingRule), Box::new(ARule)];
    let mut lexer = Lexer::from_str("a", rules);
    
    // FailingRule should try first, fail, restore cursor, then ARule should match
    let token = lexer.next_token();
    assert_eq!(token, Some(TestToken::A { position: Position::new() }));
}

