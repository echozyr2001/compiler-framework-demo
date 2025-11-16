use lexer_framework::{DefaultContext, LexContext, Position};

#[test]
fn test_default_context_new() {
    let ctx = DefaultContext::new("hello");
    assert!(!ctx.is_eof());
    assert_eq!(ctx.peek(), Some('h'));
}

#[test]
fn test_default_context_peek() {
    let ctx = DefaultContext::new("hello");
    assert_eq!(ctx.peek(), Some('h'));
    // Peek should not advance
    assert_eq!(ctx.peek(), Some('h'));
}

#[test]
fn test_default_context_advance() {
    let mut ctx = DefaultContext::new("hello");
    assert_eq!(ctx.advance(), Some('h'));
    assert_eq!(ctx.peek(), Some('e'));
}

#[test]
fn test_default_context_consume_while() {
    let mut ctx = DefaultContext::new("hello world");
    let result = ctx.consume_while(|c| c.is_alphabetic());
    assert_eq!(result, "hello");
    assert_eq!(ctx.peek(), Some(' '));
}

#[test]
fn test_default_context_position() {
    let mut ctx = DefaultContext::new("a\nb");
    assert_eq!(ctx.position(), Position::new());
    ctx.advance(); // 'a'
    assert_eq!(ctx.position().column, 2);
    ctx.advance(); // '\n'
    assert_eq!(ctx.position().line, 2);
    assert_eq!(ctx.position().column, 1);
}

#[test]
fn test_default_context_checkpoint_restore() {
    let mut ctx = DefaultContext::new("hello");
    ctx.advance(); // 'h'
    ctx.advance(); // 'e'

    let checkpoint = ctx.checkpoint();
    ctx.advance(); // 'l'
    ctx.advance(); // 'l'

    ctx.restore(checkpoint);
    assert_eq!(ctx.peek(), Some('l'));
    assert_eq!(ctx.position().column, 3);
}

#[test]
fn test_default_context_is_eof() {
    let mut ctx = DefaultContext::new("hi");
    assert!(!ctx.is_eof());
    ctx.advance();
    assert!(!ctx.is_eof());
    ctx.advance();
    assert!(ctx.is_eof());
}

#[test]
fn test_default_context_empty() {
    let ctx = DefaultContext::new("");
    assert!(ctx.is_eof());
    assert_eq!(ctx.peek(), None);
}
