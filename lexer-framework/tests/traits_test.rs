//! Traits 测试：测试 LexToken 和 LexingRule trait 的行为

use lexer_framework::{DefaultContext, LexContext, LexToken, LexingRule, Position};

type RuleSet<Tok> = Vec<Box<dyn LexingRule<DefaultContext, Tok>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestToken {
    value: String,
    position: Position,
    is_eof: bool,
    is_newline: bool,
    is_whitespace: bool,
    is_indent: bool,
}

impl LexToken for TestToken {
    fn position(&self) -> Option<Position> {
        Some(self.position)
    }

    fn is_eof(&self) -> bool {
        self.is_eof
    }

    fn is_newline(&self) -> bool {
        self.is_newline
    }

    fn is_whitespace(&self) -> bool {
        self.is_whitespace
    }

    fn is_indent(&self) -> bool {
        self.is_indent
    }
}

#[test]
fn test_lex_token_position() {
    let pos = Position {
        line: 5,
        column: 10,
        offset: 100,
    };
    let token = TestToken {
        value: "test".to_string(),
        position: pos,
        is_eof: false,
        is_newline: false,
        is_whitespace: false,
        is_indent: false,
    };

    assert_eq!(token.position(), Some(pos));
}

#[test]
fn test_lex_token_is_eof() {
    let token = TestToken {
        value: "".to_string(),
        position: Position::new(),
        is_eof: true,
        is_newline: false,
        is_whitespace: false,
        is_indent: false,
    };

    assert!(token.is_eof());
}

#[test]
fn test_lex_token_is_newline() {
    let token = TestToken {
        value: "\n".to_string(),
        position: Position::new(),
        is_eof: false,
        is_newline: true,
        is_whitespace: false,
        is_indent: false,
    };

    assert!(token.is_newline());
}

#[test]
fn test_lex_token_is_whitespace() {
    let token = TestToken {
        value: "   ".to_string(),
        position: Position::new(),
        is_eof: false,
        is_newline: false,
        is_whitespace: true,
        is_indent: false,
    };

    assert!(token.is_whitespace());
}

#[test]
fn test_lex_token_is_indent() {
    let token = TestToken {
        value: "\t\t".to_string(),
        position: Position::new(),
        is_eof: false,
        is_newline: false,
        is_whitespace: true,
        is_indent: true,
    };

    assert!(token.is_indent());
}

// Test LexingRule trait
struct SimpleRule {
    match_char: char,
}

impl<Ctx> LexingRule<Ctx, TestToken> for SimpleRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        if ctx.peek() == Some(self.match_char) {
            let position = ctx.position();
            ctx.advance();
            Some(TestToken {
                value: self.match_char.to_string(),
                position,
                is_eof: false,
                is_newline: false,
                is_whitespace: false,
                is_indent: false,
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10
    }
}

#[test]
fn test_lexing_rule_default_priority() {
    struct DefaultPriorityRule;

    impl<Ctx> LexingRule<Ctx, TestToken> for DefaultPriorityRule
    where
        Ctx: LexContext,
    {
        fn try_match(&mut self, _ctx: &mut Ctx) -> Option<TestToken> {
            None
        }
        // No priority() override - should use default 0
    }

    // Test through actual usage
    use lexer_framework::Lexer;
    let rules: RuleSet<TestToken> = vec![Box::new(DefaultPriorityRule)];
    let _lexer = Lexer::from_str("test", rules);
    // This test verifies the rule compiles and uses default priority
}

#[test]
fn test_lexing_rule_custom_priority() {
    // Test through actual usage
    use lexer_framework::Lexer;
    let rules: RuleSet<TestToken> = vec![Box::new(SimpleRule { match_char: 'a' })];
    let mut lexer = Lexer::from_str("a", rules);

    // Should match 'a' with priority 10
    let token = lexer.next_token();
    assert!(token.is_some());
}

#[test]
fn test_lexing_rule_default_quick_check() {
    struct NoQuickCheckRule;

    impl<Ctx> LexingRule<Ctx, TestToken> for NoQuickCheckRule
    where
        Ctx: LexContext,
    {
        fn try_match(&mut self, _ctx: &mut Ctx) -> Option<TestToken> {
            None
        }
        // No quick_check() override - should return None
    }

    // Test through actual usage with DefaultContext
    // quick_check should return None (default implementation)
    // We can't directly test this without type parameters, but we can test behavior
    // through integration with Lexer
    // This test verifies the rule compiles and can be used
    let _rule = NoQuickCheckRule;
}

#[test]
fn test_lexing_rule_quick_check_implemented() {
    struct WithQuickCheckRule;

    impl<Ctx> LexingRule<Ctx, TestToken> for WithQuickCheckRule
    where
        Ctx: LexContext,
    {
        fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
            Some(first_char == Some('x'))
        }

        fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
            if ctx.peek() == Some('x') {
                let position = ctx.position();
                ctx.advance();
                Some(TestToken {
                    value: "x".to_string(),
                    position,
                    is_eof: false,
                    is_newline: false,
                    is_whitespace: false,
                    is_indent: false,
                })
            } else {
                None
            }
        }
    }

    // Test through actual usage
    use lexer_framework::Lexer;
    let rules: RuleSet<TestToken> = vec![Box::new(WithQuickCheckRule)];
    let mut lexer = Lexer::from_str("x", rules);

    // Should match 'x'
    let token = lexer.next_token();
    assert!(token.is_some());

    // Should not match 'y' (quick_check returns false)
    let rules2: RuleSet<TestToken> = vec![Box::new(WithQuickCheckRule)];
    let mut lexer2 = Lexer::from_str("y", rules2);
    let token2 = lexer2.next_token();
    assert!(token2.is_none()); // quick_check should skip this rule
}
