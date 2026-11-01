//! Estado de `GameMap` lido pelos scripts — Etapa 13. Lido na 1.2.93.

use crate::board_occupancy::consts::TEAM_NONE;

/// Campos de `GameMap` usados pelos acessores já reconstruídos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameState {
    /// +0x5238: índice da ilha; 0x5F4976 e 0x5F9014 gravam -1.
    pub sector_0x5238: i32,
    /// +0xC204 aponta para o `BoardPlayer` da missão: contagem de turnos (+0x154) e time (+0x158).
    pub turn: Option<(i32, i32)>,
    /// Vida do objeto em +0xC03C: atual +0xC040, máximo +0xC044.
    pub power: (i32, i32),
    /// Campo +0x30 do perfil do jogador, lido por `IsIslandUnlocked`.
    pub profile_islands_0x30: i32,
}

impl Default for GameState {
    fn default() -> Self { Self { sector_0x5238: -1, turn: None, power: (0, 0), profile_islands_0x30: 0 } }
}

impl GameState {
    /// 0x60D9A0: `clamp(+0x5238 + 1, 1, 4)`.
    pub fn get_sector(&self) -> i32 { self.sector_0x5238.saturating_add(1).clamp(1, 4) }
    /// 0x60D960: sem `BoardPlayer`, `TEAM_NONE`.
    pub fn get_team_turn(&self) -> i32 { self.turn.map_or(TEAM_NONE, |t| t.1) }
    /// 0x60D980: sem `BoardPlayer`, 0.
    pub fn get_turn_count(&self) -> i32 { self.turn.map_or(0, |t| t.0) }
    /// 0x60A110: `n <= perfil+0x30`.
    pub fn is_island_unlocked(&self, n: i32) -> bool { n <= self.profile_islands_0x30 }
    /// 0x609BFE e 0x60CA65: `getCityPower` grava o máximo e prende a vida atual em 0..máximo.
    pub fn set_power_max(&mut self, max: i32) {
        self.power.1 = max;
        self.power.0 = self.power.0.min(max).max(0);
    }
}
