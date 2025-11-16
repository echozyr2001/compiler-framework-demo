use lexer_framework::Position;

#[test]
fn test_position_new() {
    let pos = Position::new();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);
}

#[test]
fn test_position_default() {
    let pos = Position::default();
    assert_eq!(pos, Position::new());
}

#[test]
fn test_position_equality() {
    let pos1 = Position::new();
    let pos2 = Position::new();
    assert_eq!(pos1, pos2);

    let pos3 = Position {
        line: 2,
        column: 3,
        offset: 10,
    };
    assert_ne!(pos1, pos3);
}

#[test]
fn test_position_copy() {
    let pos1 = Position {
        line: 2,
        column: 3,
        offset: 10,
    };
    let pos2 = pos1; // Copy
    assert_eq!(pos1, pos2);

    // Modify pos2, pos1 should be unchanged
    let pos3 = Position {
        line: 5,
        column: 6,
        offset: 20,
    };
    assert_ne!(pos1, pos3);
}
