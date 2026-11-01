//! Consultas de tabuleiro que os scripts de missão usam — Etapa 13, bloco 1 da fila de uso (`itb-uso`).
//! Lidas na 1.2.93, com o `libmain.so` como guia. Provas em `re/evidencias/v1.2.93/rotinas_fila_uso.md`.

use crate::board_occupancy::{consts::*, OccupancyBoard, Point, Tile};

/// Tipo que `GetPawnCount(int)` não conta.
const BONUS_DEBRIS: &str = "BonusDebris";

/// Campos do `SpaceDamage` guardado com o item da casa que `BoardSpace::IsDangerousItem` (0x5A11E0) testa.
/// Os nomes dos campos ainda não foram casados com os de `SpaceDamage`; o padrão é o de um item inofensivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDamage {
    pub i_0x288c: i32,
    /// Comparado com 4 (`DIR_NONE`): provavelmente o empurrão.
    pub i_0x2890: i32,
    pub i_0x2894: i32,
    pub i_0x289c: i32,
    pub i_0x28a0: i32,
    pub i_0x28a8: i32,
    pub i_0x28ac: i32,
    pub i_0x2938: i32,
}

impl Default for ItemDamage {
    fn default() -> Self {
        Self { i_0x288c: 0, i_0x2890: 4, i_0x2894: 0, i_0x289c: 0, i_0x28a0: 0, i_0x28a8: 0, i_0x28ac: 0, i_0x2938: 0 }
    }
}

impl ItemDamage {
    /// Algum campo diferente do inofensivo (0x5A11E9..0x5A122F).
    pub fn does_something(&self) -> bool { *self != Self::default() }
}

impl OccupancyBoard {
    /// 0x5700A0 `Board::IsPowered(Point)`: prédio único (+0x27AC != 0) está com vida cheia; os outros, casa povoada
    /// (+0x2710) com prédio (terreno 1).
    pub fn is_powered(&self, p: Point) -> bool {
        let t = self.tile(p);
        if t.unique_0x27ac != 0 { return t.health == t.max_health; }
        t.byte_0x2710 && t.terrain == TERRAIN_BUILDING
    }

    /// 0x56D650 `Board::IsFrozen(Point)`: casa congelada (+0x2AA1), ou a primeira unidade da lista da casa congelada
    /// (+0x8D1), se `Tile::HasPawn(true)`.
    pub fn is_frozen_at(&self, p: Point) -> bool {
        let t = self.tile(p);
        if t.frozen_0x2aa1 { return true; }
        self.tile_has_pawn(t, true) && t.pawns.first().is_some_and(|&i| self.pawns[i].status.frozen)
    }

    /// 0x5725C0 `Board::IsDangerousItem(Point)` → `BoardSpace::IsDangerousItem` (0x5A11E0): há item (+0x2B04) e o
    /// dano guardado com ele faz algo.
    pub fn is_dangerous_item(&self, p: Point) -> bool { Self::tile_is_dangerous_item(self.tile(p)) }

    pub fn tile_is_dangerous_item(t: &Tile) -> bool { t.item_0x2b04 != 0 && t.item_damage.does_something() }

    /// 0x571630 `Board::GetPawnCount(int time)`. Na lista do tabuleiro, fora "BonusDebris": `vivas` = do time e não
    /// mortas; `congeladas` = do time e congeladas (mortas inclusive). Com time −1, `vivas` soma a fila de surgimento
    /// (e o motor registra "Already Spawning"). Sem nenhuma, 0; senão `(int) max(vivas − congeladas / 3.0f, 1.0f)`.
    pub fn get_pawn_count(&self, team: i32) -> i32 {
        let (mut alive, mut frozen) = (0i32, 0i32);
        for &i in &self.pawn_list {
            let p = &self.pawns[i];
            let counts = p.type_name != BONUS_DEBRIS;
            if p.is_team(team) && !p.is_dead() && counts { alive += 1; }
            if p.is_team(team) && p.status.frozen && counts { frozen += 1; }
        }
        if team == -1 { alive += self.spawn_queue.len() as i32; }
        if alive == 0 { return 0; }
        (alive as f32 - frozen as f32 / 3.0f32).max(1.0f32) as i32
    }

    /// 0x566FF0 `Board::GetEnemyCount()`: `GetPawnCount(TEAM_ENEMY)`.
    pub fn get_enemy_count(&self) -> i32 { self.get_pawn_count(TEAM_ENEMY) }

    /// 0x567000 `Board::GetPawnCount(string)`: unidades da lista do tabuleiro, mortas inclusive, cujo tipo sem o último
    /// caractere é igual a `name` ("Firefly1" → "Firefly"). Tipo vazio não perde caractere (`substr(0, npos)`).
    pub fn get_pawn_count_type(&self, name: &str) -> i32 {
        self.pawn_list.iter().filter(|&&i| {
            let t = &self.pawns[i].type_name;
            let base = match t.char_indices().last() { Some((k, _)) => &t[..k], None => t.as_str() };
            base == name
        }).count() as i32
    }

    /// 0x56E6D0 `Board::IsTargeted(Point)`: alguma unidade da lista, com ataque na fila, atinge a casa
    /// (`SkillManager::IsTargeted`). O Rust ainda não tem a fila de ataques (`SkillManager+0x20..+0x28`, turno
    /// inimigo), então nenhuma unidade tem ataque enfileirado e a resposta é sempre falso.
    pub fn is_targeted(&self, _p: Point) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_occupancy::PawnState;

    fn pawn(ty: &str, team: i32, at: Point) -> PawnState {
        PawnState { type_name: ty.into(), team, health: 2, max_health: 2, space: at, ..Default::default() }
    }

    #[test]
    fn contagem_de_unidades() {
        let mut b = OccupancyBoard::new(8, 8).unwrap();
        assert_eq!(b.get_enemy_count(), 0);
        for (k, ty) in ["Firefly1", "Firefly2", "Hornet1", "BonusDebris"].iter().enumerate() {
            b.add_pawn(pawn(ty, TEAM_ENEMY, Point::new(k as i32, 0))).unwrap();
        }
        assert_eq!(b.get_enemy_count(), 3);
        // uma congelada vale 2/3: 3 - 1/3 = 2,67 → 2
        b.pawns[0].status.frozen = true;
        assert_eq!(b.get_enemy_count(), 2);
        // morta não conta como viva, mas congelada conta: 2 - 1/3 → 1
        b.pawns[1].health = 0;
        assert_eq!(b.get_enemy_count(), 1);
        // o mínimo é 1 quando há alguma viva
        b.pawns[2].status.frozen = true;
        assert_eq!(b.get_enemy_count(), 1);
        assert_eq!(b.get_pawn_count_type("Firefly"), 2);
        assert_eq!(b.get_pawn_count_type("Hornet"), 1);
        assert_eq!(b.get_pawn_count_type("BonusDebri"), 1);
    }

    #[test]
    fn energia_e_gelo_da_casa() {
        let mut b = OccupancyBoard::new(3, 1).unwrap();
        let p = Point::new(0, 0);
        {
            let t = b.tile_mut(p).unwrap();
            t.terrain = TERRAIN_BUILDING;
            t.byte_0x2710 = true;
        }
        assert!(b.is_powered(p));
        b.tile_mut(p).unwrap().byte_0x2710 = false;
        assert!(!b.is_powered(p));
        {
            let t = b.tile_mut(p).unwrap();
            t.unique_0x27ac = 1;
            t.health = 1;
            t.max_health = 2;
        }
        assert!(!b.is_powered(p));
        b.tile_mut(p).unwrap().health = 2;
        assert!(b.is_powered(p));

        let q = Point::new(1, 0);
        assert!(!b.is_frozen_at(q));
        b.add_pawn(pawn("Firefly1", TEAM_ENEMY, q)).unwrap();
        let i = b.tile(q).pawns[0];
        b.pawns[i].status.frozen = true;
        assert!(b.is_frozen_at(q));
        b.tile_mut(Point::new(2, 0)).unwrap().frozen_0x2aa1 = true;
        assert!(b.is_frozen_at(Point::new(2, 0)));
    }

    #[test]
    fn item_perigoso() {
        let mut b = OccupancyBoard::new(1, 1).unwrap();
        let p = Point::new(0, 0);
        assert!(!b.is_dangerous_item(p));
        b.tile_mut(p).unwrap().item_0x2b04 = 1;
        assert!(!b.is_dangerous_item(p));
        b.tile_mut(p).unwrap().item_damage.i_0x288c = 1;
        assert!(b.is_dangerous_item(p));
    }
}
