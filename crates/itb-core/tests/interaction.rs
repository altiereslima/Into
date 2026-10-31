use itb_core::{board_queries::{BoardSnapshot,CellState},interaction::{BoardInteraction,InteractionEvent},input::{GameAction,Tile}};
fn board()->BoardSnapshot {
 let mut b=BoardSnapshot::new(8,8,CellState::default()).unwrap();
 b.set_cell(2,2,CellState{active_pawn_team:Some(0),..CellState::default()}).unwrap();b
}
fn t(x:i32,y:i32)->Tile {Tile{col:x,row:y}}
#[test] fn starts_at_center_and_steps_clamp() {
 let b=board();let mut it=BoardInteraction::new(&b);
 assert_eq!(it.cursor(),t(4,4));
 for _ in 0..50 {it.apply(&b,GameAction::CursorStep{dx:-1,dy:-1});}
 assert_eq!(it.cursor(),t(0,0));
 for _ in 0..50 {it.apply(&b,GameAction::CursorStep{dx:1,dy:1});}
 assert_eq!(it.cursor(),t(7,7));
}
#[test] fn selecting_empty_does_not_select_pawn() {
 let b=board();let mut it=BoardInteraction::new(&b);
 assert!(it.apply(&b,GameAction::SelectTile(t(0,0))).iter().all(|e|!matches!(e,InteractionEvent::Selected(_))));
 assert_eq!(it.selection(),None);
}
#[test] fn pawn_selection_and_move_intent_only() {
 let b=board();let mut it=BoardInteraction::new(&b);
 let r=it.apply(&b,GameAction::SelectTile(t(2,2)));
 assert!(r.contains(&InteractionEvent::Selected(t(2,2))));
 let r=it.apply(&b,GameAction::SelectTile(t(3,2)));
 assert!(r.contains(&InteractionEvent::MoveRequested{from:t(2,2),to:t(3,2)}));
 assert_eq!(it.selection(),Some(t(2,2)));
 assert_eq!(b.get_pawn_team(2,2),0);
 assert_eq!(b.get_pawn_team(3,2),2); // NENHUMA movimentação efetiva
}
#[test] fn cancel_and_same_tile_clear_selection() {
 let b=board();let mut it=BoardInteraction::new(&b);
 it.apply(&b,GameAction::SelectTile(t(2,2)));
 assert_eq!(it.apply(&b,GameAction::Confirm),vec![InteractionEvent::SelectionCleared]);
 it.apply(&b,GameAction::SelectTile(t(2,2)));
 assert_eq!(it.apply(&b,GameAction::Cancel),vec![InteractionEvent::SelectionCleared]);
}
#[test] fn ignores_invalid_mouse_selection_and_hover() {
 let b=board();let mut it=BoardInteraction::new(&b);
 assert!(it.apply(&b,GameAction::SelectTile(t(-1,9))).is_empty());
 assert_eq!(it.cursor(),t(4,4));
 assert!(it.apply(&b,GameAction::HoverTile(Some(t(-1,9)))).is_empty());
 assert_eq!(it.hover(),None);
}
#[test] fn emits_turn_and_undo_requests_without_changing_board() {
 let b=board();let mut it=BoardInteraction::new(&b);
 assert_eq!(it.apply(&b,GameAction::EndTurn),vec![InteractionEvent::EndTurnRequested]);
 assert_eq!(it.apply(&b,GameAction::Undo),vec![InteractionEvent::UndoRequested]);
 assert_eq!(it.apply(&b,GameAction::ResetTurn),vec![InteractionEvent::ResetRequested]);
 assert_eq!(b.get_pawn_team(2,2),0);
}
