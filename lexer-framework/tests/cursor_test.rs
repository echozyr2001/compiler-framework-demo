use lexer_framework::{Cursor, Position};

#[test]
fn test_cursor_new() {
    let cursor = Cursor::new("hello");
    assert_eq!(cursor.offset(), 0);
    assert_eq!(cursor.position(), Position::new());
    assert!(!cursor.is_eof());
}

#[test]
fn test_cursor_peek() {
    let cursor = Cursor::new("hello");
    assert_eq!(cursor.peek(), Some('h'));
    assert_eq!(cursor.offset(), 0); // Should not advance
}

#[test]
fn test_cursor_advance() {
    let mut cursor = Cursor::new("hello");
    assert_eq!(cursor.advance(), Some('h'));
    assert_eq!(cursor.offset(), 1);
    assert_eq!(cursor.position().column, 2);
    assert_eq!(cursor.peek(), Some('e'));
}

#[test]
fn test_cursor_advance_multiple() {
    let mut cursor = Cursor::new("hello");
    cursor.advance();
    cursor.advance();
    assert_eq!(cursor.peek(), Some('l'));
    assert_eq!(cursor.offset(), 2);
}

#[test]
fn test_cursor_is_eof() {
    let mut cursor = Cursor::new("hi");
    assert!(!cursor.is_eof());
    cursor.advance();
    assert!(!cursor.is_eof());
    cursor.advance();
    assert!(cursor.is_eof());
    assert_eq!(cursor.peek(), None);
    assert_eq!(cursor.advance(), None);
}

#[test]
fn test_cursor_empty_string() {
    let mut cursor = Cursor::new("");
    assert!(cursor.is_eof());
    assert_eq!(cursor.peek(), None);
    assert_eq!(cursor.advance(), None);
}

#[test]
fn test_cursor_position_tracking() {
    let mut cursor = Cursor::new("a\nb\nc");
    
    // First line
    assert_eq!(cursor.position().line, 1);
    assert_eq!(cursor.position().column, 1);
    cursor.advance(); // 'a'
    assert_eq!(cursor.position().line, 1);
    assert_eq!(cursor.position().column, 2);
    
    // Newline
    cursor.advance(); // '\n'
    assert_eq!(cursor.position().line, 2);
    assert_eq!(cursor.position().column, 1);
    
    // Second line
    cursor.advance(); // 'b'
    assert_eq!(cursor.position().line, 2);
    assert_eq!(cursor.position().column, 2);
}

#[test]
fn test_cursor_consume_while() {
    let mut cursor = Cursor::new("hello world");
    let result = cursor.consume_while(|c| c.is_alphabetic());
    assert_eq!(result, "hello");
    assert_eq!(cursor.peek(), Some(' '));
}

#[test]
fn test_cursor_consume_while_empty() {
    let mut cursor = Cursor::new("123");
    let result = cursor.consume_while(|c| c.is_alphabetic());
    assert_eq!(result, "");
    assert_eq!(cursor.peek(), Some('1'));
}

#[test]
fn test_cursor_peek_str() {
    let cursor = Cursor::new("hello");
    assert_eq!(cursor.peek_str(3), "hel");
    assert_eq!(cursor.offset(), 0); // Should not advance
}

#[test]
fn test_cursor_peek_str_unicode() {
    let cursor = Cursor::new("你好");
    assert_eq!(cursor.peek_str(1), "你");
    assert_eq!(cursor.peek_str(2), "你好");
}

#[test]
fn test_cursor_peek_str_emoji() {
    let cursor = Cursor::new("😀🎉");
    assert_eq!(cursor.peek_str(1), "😀");
    assert_eq!(cursor.peek_str(2), "😀🎉");
}

#[test]
fn test_cursor_checkpoint_restore() {
    let mut cursor = Cursor::new("hello");
    cursor.advance(); // 'h'
    cursor.advance(); // 'e'
    
    let checkpoint = cursor.checkpoint();
    assert_eq!(cursor.offset(), 2);
    
    cursor.advance(); // 'l'
    cursor.advance(); // 'l'
    assert_eq!(cursor.offset(), 4);
    
    cursor.restore(checkpoint);
    assert_eq!(cursor.offset(), 2);
    assert_eq!(cursor.peek(), Some('l'));
}

#[test]
fn test_cursor_reset() {
    let mut cursor = Cursor::new("hello");
    cursor.advance();
    cursor.advance();
    cursor.reset();
    
    assert_eq!(cursor.offset(), 0);
    assert_eq!(cursor.position(), Position::new());
    assert_eq!(cursor.peek(), Some('h'));
}

#[test]
fn test_cursor_remaining() {
    let mut cursor = Cursor::new("hello world");
    cursor.advance_by(6); // "hello "
    assert_eq!(cursor.remaining(), "world");
}

#[test]
fn test_cursor_advance_by() {
    let mut cursor = Cursor::new("hello");
    let count = cursor.advance_by(3);
    assert_eq!(count, 3);
    assert_eq!(cursor.peek(), Some('l'));
}

#[test]
fn test_cursor_advance_by_beyond_eof() {
    let mut cursor = Cursor::new("hi");
    let count = cursor.advance_by(10);
    assert_eq!(count, 2);
    assert!(cursor.is_eof());
}

#[test]
fn test_cursor_unicode_chinese() {
    let mut cursor = Cursor::new("你好世界");
    assert_eq!(cursor.peek(), Some('你'));
    cursor.advance();
    assert_eq!(cursor.peek(), Some('好'));
    assert_eq!(cursor.position().column, 2);
}

#[test]
fn test_cursor_unicode_emoji() {
    let mut cursor = Cursor::new("😀🎉🚀");
    assert_eq!(cursor.peek(), Some('😀'));
    cursor.advance();
    assert_eq!(cursor.peek(), Some('🎉'));
    assert_eq!(cursor.position().column, 2);
}

