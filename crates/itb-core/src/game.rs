//! Estado de `GameMap` lido pelos scripts — Etapa 13. Lido na 1.2.93.

use crate::apply::health_add;
use crate::board_occupancy::consts::TEAM_NONE;
use crate::rng::MsvcRand;
use std::cell::RefCell;
use std::rc::Rc;

/// `PowerBar` do `GameMap` (+0xC03C; 0x8B4F50 aponta para ele): energia da rede, defesa e semente própria.
/// Construtor 0x5C2B9A: defesa-base 15 (+0xE0) e bônus 0 (+0xEC). O save grava `overflow` (+0x138) e `seed` (+0x13C).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PowerGrid {
    /// +0x04 e +0x08: energia atual e máxima (o `ValueBar` lido por `Game:GetPower`).
    pub value: i32,
    pub max: i32,
    pub defense_base_0xe0: i32,
    pub defense_bonus_0xec: i32,
    pub overflow_0x138: i32,
    /// +0x135 (BYTE): com ele ligado, a rede nunca resiste.
    pub flag_0x135: bool,
    /// +0x13C: semente da defesa (`seed` no save).
    pub seed_0x13c: i32,
    /// O motor termina `PowerBar::Damage` com `srand(QueryPerformanceCounter em ms)`. Como o relógio não se reproduz,
    /// o host escolhe o valor.
    pub clock_seed: u32,
}

impl Default for PowerGrid {
    fn default() -> Self {
        Self { value: 0, max: 0, defense_base_0xe0: 15, defense_bonus_0xec: 0, overflow_0x138: 0, flag_0x135: false,
               seed_0x13c: -1, clock_seed: 0 }
    }
}

pub type SharedGrid = Rc<RefCell<PowerGrid>>;

impl PowerGrid {
    /// 0x5C5950: `overflow + bônus + base`.
    pub fn defense(&self) -> i32 { self.overflow_0x138 + self.defense_bonus_0xec + self.defense_base_0xe0 }

    /// 0x5C59F0 `PowerBar::Damage(int)`: `srand(semente)`; com `rand() % 100 < defesa` e +0x135 desligada, a rede
    /// resiste (só evento e som); senão a energia cai (0x5C5400 → `ValueBar::AnimatedModify(−dano)`). Depois a
    /// semente avança por `unique_rand` (até 10 sorteios iguais à atual; se ainda igual, +2) e o motor faz
    /// `srand(relógio)`. Devolve se o dano passou.
    pub fn damage(&mut self, amount: i32, rng: &mut MsvcRand) -> bool {
        rng.srand(self.seed_0x13c as u32);
        let r = rng.rand() % 100;
        let resisted = r < self.defense() && !self.flag_0x135;
        if !resisted {
            let max = self.max;
            health_add(&mut self.value, max, -amount);
        }
        let mut next = rng.rand();
        let mut tries = 0;
        while next == self.seed_0x13c && tries < 10 {
            next = rng.rand();
            tries += 1;
        }
        self.seed_0x13c = if next == self.seed_0x13c { next + 2 } else { next };
        rng.srand(self.clock_seed);
        !resisted
    }
}

/// Campos de `GameMap` usados pelos acessores já reconstruídos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    /// +0x5238: índice da ilha; 0x5F4976 e 0x5F9014 gravam -1.
    pub sector_0x5238: i32,
    /// +0xC204 aponta para o `BoardPlayer` da missão: contagem de turnos (+0x154) e time (+0x158).
    pub turn: Option<(i32, i32)>,
    /// `PowerBar` em +0xC03C (energia atual +0xC040, máximo +0xC044), compartilhado com o tabuleiro da missão.
    pub grid: SharedGrid,
    /// Campo +0x30 do perfil do jogador, lido por `IsIslandUnlocked`.
    pub profile_islands_0x30: i32,
}

impl Default for GameState {
    fn default() -> Self { Self { sector_0x5238: -1, turn: None, grid: Rc::default(), profile_islands_0x30: 0 } }
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
        let mut g = self.grid.borrow_mut();
        g.max = max;
        g.value = g.value.min(max).max(0);
    }

    /// Energia (atual, máxima).
    pub fn power(&self) -> (i32, i32) {
        let g = self.grid.borrow();
        (g.value, g.max)
    }
}
