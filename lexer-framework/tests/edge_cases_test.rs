//! 边界情况和特殊场景测试

use lexer_framework::{Cursor, DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

#[derive(Debug, Clone, PartialEq)]
enum TestToken {
    Token { ch: char, position: Position },
}

impl LexToken for TestToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            TestToken::Token { position, .. } => *position,
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

impl<'input, Ctx> LexingRule<'input, Ctx, TestToken> for CharRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<TestToken> {
        let ch = ctx.peek()?;
        let position = ctx.position();
        ctx.advance();
        Some(TestToken::Token { ch, position })
    }

    fn priority(&self) -> i32 {
        10
    }
}

#[test]
fn test_cursor_peek_str_zero() {
    let cursor = Cursor::new("hello");
    assert_eq!(cursor.peek_str(0), "");
}

#[test]
fn test_cursor_peek_str_beyond_end() {
    let cursor = Cursor::new("hi");
    assert_eq!(cursor.peek_str(100), "hi");
}

#[test]
fn test_cursor_peek_str_at_eof() {
    let mut cursor = Cursor::new("h");
    cursor.advance();
    assert_eq!(cursor.peek_str(5), "");
}

#[test]
fn test_cursor_advance_at_eof() {
    let mut cursor = Cursor::new("h");
    assert_eq!(cursor.advance(), Some('h'));
    assert_eq!(cursor.advance(), None);
    assert_eq!(cursor.advance(), None); // Should be safe to call multiple times
}

#[test]
fn test_cursor_consume_while_all() {
    let mut cursor = Cursor::new("hello");
    let result = cursor.consume_while(|_| true);
    assert_eq!(result, "hello");
    assert!(cursor.is_eof());
}

#[test]
fn test_cursor_consume_while_none() {
    let mut cursor = Cursor::new("hello");
    let result = cursor.consume_while(|_| false);
    assert_eq!(result, "");
    assert_eq!(cursor.peek(), Some('h'));
}

#[test]
fn test_cursor_unicode_surrogate() {
    // Test that cursor handles Unicode correctly
    // Using a valid Unicode character instead of invalid surrogate
    let mut cursor = Cursor::new("\u{FFFD}"); // Replacement character
                                              // Should not panic
    let _ = cursor.peek();
    let _ = cursor.advance();
}

#[test]
fn test_position_default() {
    let pos1 = Position::default();
    let pos2 = Position::new();
    assert_eq!(pos1, pos2);
}

#[test]
fn test_position_display() {
    // Test Position can be formatted
    let pos = Position {
        line: 10,
        column: 20,
        offset: 100,
    };
    let formatted = format!("{:?}", pos);
    assert!(formatted.contains("10"));
    assert!(formatted.contains("20"));
}

#[test]
fn test_context_empty_string() {
    let ctx = DefaultContext::new("");
    assert!(ctx.is_eof());
    assert_eq!(ctx.peek(), None);

    let mut ctx = DefaultContext::new("");
    assert_eq!(ctx.advance(), None);
    assert_eq!(ctx.consume_while(|_| true), "");
}

#[test]
fn test_context_unicode_multibyte() {
    let mut ctx = DefaultContext::new("你好");
    assert_eq!(ctx.advance(), Some('你'));
    // Column should advance by 1 (visual character), not by bytes
    assert_eq!(ctx.position().column, 2);
    assert_eq!(ctx.position().offset, 3); // '你' is 3 bytes in UTF-8
}

#[test]
fn test_lexer_iterator_with_take() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str("hello", rules);

    let tokens: Vec<_> = lexer.take(3).collect();
    assert_eq!(tokens.len(), 3);
}

#[test]
fn test_lexer_iterator_with_filter() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str("hello", rules);

    // Filter out 'e'
    let tokens: Vec<_> = lexer
        .filter(|t| !matches!(t, TestToken::Token { ch: 'e', .. }))
        .collect();

    assert_eq!(tokens.len(), 4); // 'h', 'l', 'l', 'o'
}

#[test]
fn test_lexer_context_access() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str("hello", rules);

    // Should be able to access context
    let context = lexer.context();
    assert!(!context.is_eof());
    assert_eq!(context.peek(), Some('h'));
}

#[test]
fn test_lexer_context_mut_access() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(CharRule)];
    let mut lexer = Lexer::from_str("hello", rules);

    // Should be able to mutate context (though not recommended in normal use)
    let context_mut = lexer.context_mut();
    let _ = context_mut.peek();
}

#[test]
fn test_checkpoint_position_preservation() {
    let mut cursor = Cursor::new("a\nb");
    cursor.advance(); // 'a'
    cursor.advance(); // '\n' - should update line

    let checkpoint = cursor.checkpoint();
    // Checkpoint position is private, test via restore
    cursor.restore(checkpoint);
    assert_eq!(cursor.position().line, 2);
    assert_eq!(cursor.position().column, 1);

    cursor.advance(); // 'b'
    cursor.restore(checkpoint);

    assert_eq!(cursor.position().line, 2);
    assert_eq!(cursor.position().column, 1);
    assert_eq!(cursor.peek(), Some('b'));
}

#[test]
fn test_cursor_reset_position() {
    let mut cursor = Cursor::new("a\nb\nc");
    cursor.advance(); // 'a'
    cursor.advance(); // '\n'
    cursor.advance(); // 'b'

    assert_eq!(cursor.position().line, 2);
    cursor.reset();

    assert_eq!(cursor.position().line, 1);
    assert_eq!(cursor.position().column, 1);
    assert_eq!(cursor.position().offset, 0);
}

#[test]
fn test_cursor_advance_by_zero() {
    let mut cursor = Cursor::new("hello");
    let count = cursor.advance_by(0);
    assert_eq!(count, 0);
    assert_eq!(cursor.peek(), Some('h'));
}

#[test]
fn test_remaining_after_eof() {
    let mut cursor = Cursor::new("hi");
    cursor.advance();
    cursor.advance();
    assert_eq!(cursor.remaining(), "");
}

#[test]
fn test_position_offset_accuracy() {
    let mut cursor = Cursor::new("a\nb");

    cursor.advance(); // 'a' (1 byte)
    assert_eq!(cursor.position().offset, 1);

    cursor.advance(); // '\n' (1 byte)
    assert_eq!(cursor.position().offset, 2);

    cursor.advance(); // 'b' (1 byte)
    assert_eq!(cursor.position().offset, 3);
}

#[test]
fn test_position_offset_unicode() {
    let mut cursor = Cursor::new("你"); // 3 bytes

    cursor.advance();
    assert_eq!(cursor.position().offset, 3); // 3 bytes in UTF-8
    assert_eq!(cursor.position().column, 2); // 1 visual character
}

#[test]
fn test_lexer_empty_rules() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> = vec![];
    let lexer = Lexer::from_str("hello", rules);

    let tokens: Vec<_> = lexer.collect();
    assert!(tokens.is_empty());
}

#[test]
fn test_lexer_single_char() {
    let rules: Vec<Box<dyn LexingRule<'_, DefaultContext<'_>, TestToken> + '_>> =
        vec![Box::new(CharRule)];
    let lexer = Lexer::from_str("a", rules);

    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], TestToken::Token { ch: 'a', .. }));
}

#[test]
fn test_cursor_clone() {
    let cursor1 = Cursor::new("hello");
    let mut cursor2 = cursor1.clone();

    cursor2.advance();

    // cursor1 should be unchanged
    assert_eq!(cursor1.peek(), Some('h'));
    assert_eq!(cursor2.peek(), Some('e'));
}
