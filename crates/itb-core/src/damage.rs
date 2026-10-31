//! Previsão de dano letal: `Board:IsDeadly(SpaceDamage, Pawn)` (Etapa 10).
//! Provas: `re/evidencias/rotinas_alcance.md`.

use crate::board_occupancy::{OccupancyBoard, PawnIdx, PawnState, Point};
use crate::board_terrain::{TERRAIN_QUERY_CRACKED, TERRAIN_QUERY_DAMAGED_ICE};

/// `DAMAGE_ZERO` (registrado pelo EXE em `_G`).
pub const DAMAGE_ZERO: i32 = 500;
/// `DAMAGE_DEATH` (registrado pelo EXE em `_G`).
pub const DAMAGE_DEATH: i32 = 1000;

impl PawnState {
    /// 0x00633AD0 `Pawn:IsShield()`: `0x0062F2F0()` ou byte +0x8D4.
    pub fn is_shield(&self) -> bool { self.damage.query_0x62f2f0 || self.damage.shield_0x8d4 }

    /// 0x006313E0 `Pawn::IsBoosted()`: byte +0x8D5, ou habilidade 11, ou "Arrogant_Boost" com vida cheia.
    pub fn is_boosted(&self) -> bool {
        let d = &self.damage;
        d.boosted_0x8d5 || d.query_0x638c70_11 || (d.query_arrogant_boost && self.health == self.max_health)
    }

    /// Ajuste de `IsDeadly` (0x00573A26) quando o atacante está com impulso: dano positivo
    /// (exceto `DAMAGE_DEATH`) +1; cura de −1..−9 fica 1 mais forte. `DAMAGE_ZERO` não muda.
    pub fn boost_damage(damage: i32) -> i32 {
        if damage == DAMAGE_ZERO { return damage; }
        let mut d = damage;
        if damage > 0 && damage != DAMAGE_DEATH { d += 1; }
        if (damage.wrapping_add(9) as u32) <= 8 { d -= 1; }
        d
    }
}

impl OccupancyBoard {
    /// 0x006362A0 `Pawn::WouldDie(std::vector<SpaceDamage>)`: soma os danos na ordem.
    ///
    /// Escudo (0x0062F2F0 ou byte +0x8D4) e depois gelo (+0x8D1) absorvem o primeiro dano positivo
    /// que não seja `DAMAGE_ZERO` (sobra só `DAMAGE_DEATH`). `DAMAGE_ZERO` vale 0. Dano positivo
    /// sobre gelo danificado (`IsTerrain(18)`) mata quem não voa e tem byte +0x99D == 0; sobre chão
    /// rachado (`IsTerrain(16)`) mata quem não voa. Blindagem tira 1 (mínimo 0) e ácido (+0x8D3)
    /// dobra. Dano negativo cura. Resultado: vida final <= 0.
    pub fn pawn_would_die(&self, idx: PawnIdx, damages: &[i32]) -> bool {
        let p = &self.pawns[idx];
        let mut hp = p.health;
        let mut shield = p.is_shield();
        let mut frozen = p.status.frozen;
        if p.is_dead() { return false; }
        for &orig in damages {
            let mut d = orig;
            if shield && d != DAMAGE_ZERO && d > 0 {
                if d != DAMAGE_DEATH { d = 0; }
                shield = false;
            } else if frozen && d != DAMAGE_ZERO && d > 0 {
                if d != DAMAGE_DEATH { d = 0; }
                frozen = false;
            }
            if d == DAMAGE_ZERO {
                d = 0;
            } else if d > 0 {
                let flies = p.is_flying();
                if self.is_terrain(p.space, TERRAIN_QUERY_DAMAGED_ICE) && !flies {
                    if p.damage.byte_0x99d == 0 { return true; }
                    if p.is_dead() && !p.is_corpse(self.global_list_has_12) { return true; }
                }
                if self.is_terrain(p.get_space(), TERRAIN_QUERY_CRACKED) && !flies {
                    return true; // o dano gravado (d) já é > 0 e != DAMAGE_ZERO aqui
                }
                if p.damage.query_0x63c100 { d = (d - 1).max(0); }
                if p.status.acid { d = d.wrapping_add(d); }
            }
            hp = hp.wrapping_sub(d);
        }
        hp <= 0
    }

    /// 0x005739E0 `Board:IsDeadly(SpaceDamage, Pawn*)`.
    ///
    /// Com atacante sob impulso, ajusta o dano (`boost_damage`). Sem unidade em `IsPawnSpace(loc)`
    /// → falso. Senão: `WouldDie` da PRIMEIRA unidade da casa (`GetPawn(Point)`) com esse dano.
    pub fn is_deadly(&self, loc: Point, damage: i32, attacker: Option<PawnIdx>) -> bool {
        let mut d = damage;
        if let Some(a) = attacker {
            if self.pawns[a].is_boosted() { d = PawnState::boost_damage(d); }
        }
        if !self.is_pawn_space(loc) { return false; }
        match self.get_pawn_at(loc) {
            Some(i) => self.pawn_would_die(i, &[d]),
            None => false,
        }
    }
}
