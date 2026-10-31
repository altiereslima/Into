//! Camada NOVA para ligar GameAction (teclado/mouse/controle) a consultas do tabuleiro.
//! Não implementa a movimentação nativa, combate nem as regras de turno do EXE.
use crate::{board_queries::BoardSnapshot, input::{GameAction, Tile}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionEvent {
    Cursor(Tile), Hover(Option<Tile>), Selected(Tile), SelectionCleared,
    MoveRequested { from: Tile, to: Tile }, UndoRequested,
    ResetRequested, EndTurnRequested, MenuRequested, ZoomRequested(i8),
}

#[derive(Clone, Debug)]
pub struct BoardInteraction {
    cursor: Tile,
    selected: Option<Tile>,
    hover: Option<Tile>,
}
impl BoardInteraction {
    pub fn new(board: &BoardSnapshot) -> Self {
        let bounds = board.bounds();
        Self { cursor: Tile { col: bounds.width / 2, row: bounds.height / 2 }, selected: None, hover: None }
    }
    pub fn cursor(&self) -> Tile { self.cursor }
    pub fn selection(&self) -> Option<Tile> { self.selected }
    pub fn hover(&self) -> Option<Tile> { self.hover }

    /// Emite *intenções*, sem fazer movimentos ilegais ou inventar pathfinding.
    pub fn apply(&mut self, board: &BoardSnapshot, action: GameAction) -> Vec<InteractionEvent> {
        let mut out = Vec::new();
        match action {
            GameAction::CursorStep { dx, dy } => {
                let b = board.bounds();
                let next = Tile {
                    col: self.cursor.col.saturating_add(i32::from(dx)).clamp(0, b.width - 1),
                    row: self.cursor.row.saturating_add(i32::from(dy)).clamp(0, b.height - 1),
                };
                if self.cursor != next { self.cursor = next; out.push(InteractionEvent::Cursor(next)); }
            }
            GameAction::HoverTile(tile) => {
                let valid = tile.filter(|t| board.is_valid(t.col, t.row));
                if self.hover != valid { self.hover = valid; out.push(InteractionEvent::Hover(valid)); }
            }
            GameAction::SelectTile(tile) => {
                if board.is_valid(tile.col, tile.row) {
                    self.cursor = tile;
                    out.push(InteractionEvent::Cursor(tile));
                    out.extend(self.confirm_tile(board, tile));
                }
            }
            GameAction::Confirm => out.extend(self.confirm_tile(board, self.cursor)),
            GameAction::Cancel => {
                if self.selected.take().is_some() { out.push(InteractionEvent::SelectionCleared); }
            }
            GameAction::Undo => out.push(InteractionEvent::UndoRequested),
            GameAction::ResetTurn => out.push(InteractionEvent::ResetRequested),
            GameAction::EndTurn => out.push(InteractionEvent::EndTurnRequested),
            GameAction::ToggleMenu => out.push(InteractionEvent::MenuRequested),
            GameAction::Zoom(amount) => out.push(InteractionEvent::ZoomRequested(amount)),
        }
        out
    }
    fn confirm_tile(&mut self, board: &BoardSnapshot, tile: Tile) -> Vec<InteractionEvent> {
        if let Some(from) = self.selected {
            if from == tile { self.selected = None; vec![InteractionEvent::SelectionCleared] }
            else { vec![InteractionEvent::MoveRequested { from, to: tile }] }
        } else if board.get_cell(tile.col, tile.row).active_pawn_team.is_some() {
            self.selected = Some(tile);
            vec![InteractionEvent::Selected(tile)]
        } else { vec![] }
    }
}
