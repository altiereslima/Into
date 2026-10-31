use itb_core::board::{
    BoardBounds, ObservedBoardPrefix, ORIGINAL_HEIGHT_OFFSET, ORIGINAL_WIDTH_OFFSET,
};

#[test]
fn eleven_original_boundary_checks() {
    // Mesmos 11 casos do teste C++ da ETAPA 01, aprovado anteriormente.
    let mut board = BoardBounds { width: 8, height: 8 };
    assert!(board.is_valid(0, 0));
    assert!(board.is_valid(7, 7));
    assert!(!board.is_valid(-1, 0));
    assert!(!board.is_valid(0, -1));
    assert!(!board.is_valid(8, 0));
    assert!(!board.is_valid(0, 8));
    assert!(!board.is_valid(999, 999));
    board.width = 5;
    board.height = 3;
    assert!(board.is_valid(4, 2));
    assert!(!board.is_valid(5, 2));
    assert!(!board.is_valid(4, 3));
    board.width = 0;
    assert!(!board.is_valid(0, 0));
}

#[test]
fn signed_integer_boundaries() {
    let board = BoardBounds { width: i32::MAX, height: i32::MAX };
    assert!(board.is_valid(i32::MAX - 1, i32::MAX - 1));
    assert!(!board.is_valid(i32::MAX, 0));
    assert!(!board.is_valid(i32::MIN, 0));
    assert!(!board.is_valid(0, i32::MIN));
    assert!(!BoardBounds { width: -1, height: 8 }.is_valid(0, 0));
    assert!(!BoardBounds { width: 8, height: -1 }.is_valid(0, 0));
}

#[test]
fn documented_offsets_match_repr_c_prefix() {
    assert_eq!(std::mem::offset_of!(ObservedBoardPrefix, width_at_0x48), ORIGINAL_WIDTH_OFFSET);
    assert_eq!(std::mem::offset_of!(ObservedBoardPrefix, height_at_0x4c), ORIGINAL_HEIGHT_OFFSET);
}
