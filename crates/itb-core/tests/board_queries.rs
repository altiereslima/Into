use itb_core::{board_queries::{BoardSnapshot, CellState, BoardError, ORIGINAL_ABSENT_PAWN_TEAM}, board::BoardBounds};
fn test_board() -> BoardSnapshot { BoardSnapshot::new(8,8,CellState {terrain: 9, fire_state: 2, smoke_state: 1, vines_state: 1, acid_state: 1, cracked_state: 1, active_pawn_team: None}).unwrap() }

#[test] fn rejects_bad_dimensions() {
    assert!(matches!(BoardSnapshot::new(0,8,CellState::default()),Err(BoardError::InvalidDimensions)));
    assert!(matches!(BoardSnapshot::new(8,-1,CellState::default()),Err(BoardError::InvalidDimensions)));
    assert!(matches!(BoardSnapshot::new(i32::MAX,i32::MAX,CellState::default()),Err(BoardError::ExcessiveArea)));
}
#[test] fn bounds_old_behavior_preserved() { assert_eq!(test_board().bounds(),BoardBounds{width:8,height:8}); }
#[test] fn terrain_and_building_reuse_same_field() {
    let mut b=test_board();
    b.set_cell(1,2,CellState{terrain:1,..CellState::default()}).unwrap();
    b.set_cell(2,2,CellState{terrain:2,..CellState::default()}).unwrap();
    assert_eq!(b.get_terrain(1,2),1);
    assert!(b.is_building(1,2));
    assert!(!b.is_building(2,2));
    assert!(!b.is_building(0,0));
}
#[test] fn invalid_getterrain_reads_sentinel_not_zero_or_panic() {
    let b=test_board();
    for (x,y) in [(-1,0),(8,0),(0,-1),(0,8),(i32::MAX,0)] {
        assert_eq!(b.get_terrain(x,y),9);
    }
}
#[test] fn flags_are_nonzero_not_strictly_one() {
    let mut b=test_board();
    b.set_cell(0,1,CellState{fire_state:-3,smoke_state:255,vines_state:4,..CellState::default()}).unwrap();
    assert!(b.is_fire(0,1));assert!(b.is_smoke(0,1));assert!(b.is_vines(0,1));
    assert!(!b.is_fire(2,1));assert!(!b.is_smoke(2,1));assert!(!b.is_vines(2,1));
}
#[test] fn sentinel_also_supplies_out_of_bounds_flags() {
    let b=test_board();
    assert!(b.is_fire(-1,3));assert!(b.is_smoke(-1,3));assert!(b.is_vines(-1,3));
}
#[test] fn edge_matches_exe_comparison_even_out_of_bounds() {
    let b=test_board();
    for (x,y) in [(0,0),(7,7),(0,99),(-7,0),(1,7),(7,1)] {assert!(b.is_edge(x,y),"({x},{y})");}
    for (x,y) in [(1,1),(-1,1),(8,1),(-1,-1),(8,8)] {assert!(!b.is_edge(x,y),"({x},{y})");}
}
#[test] fn team_defaults_to_native_constant_when_no_active_pawn() {
    let mut b=test_board();
    b.set_cell(3,4,CellState{active_pawn_team:Some(0),..CellState::default()}).unwrap();
    assert_eq!(b.get_pawn_team(3,4),0);
    assert_eq!(b.get_pawn_team(2,4),ORIGINAL_ABSENT_PAWN_TEAM);
    assert_eq!(b.get_pawn_team(-1,-1),ORIGINAL_ABSENT_PAWN_TEAM);
}
#[test] fn set_cell_does_not_modify_sentinel() {
    let mut b=test_board();
    assert_eq!(b.set_cell(-1,0,CellState::default()),Err(BoardError::OutOfBounds));
    assert_eq!(b.get_terrain(-1,0),9);
}

#[test] fn acid_and_cracked_are_byte_flags() {
    let mut b=test_board();
    b.set_cell(3,3,CellState{acid_state:1,cracked_state:255,..CellState::default()}).unwrap();
    assert!(b.is_acid(3,3)); assert!(b.is_cracked(3,3));
    assert!(!b.is_acid(4,4)); assert!(!b.is_cracked(4,4));
    assert!(b.is_acid(-1,-1)); assert!(b.is_cracked(-1,-1)); // sentinela
}
