//! Consultas de tabuleiro que os scripts de missão usam — Etapa 13, bloco 1 da fila de uso (`itb-uso`).
//! Lidas na 1.2.93, com o `libmain.so` como guia. Provas em `re/evidencias/v1.2.93/rotinas_fila_uso.md`.

use crate::board_occupancy::{consts::*, OccupancyBoard, Point, Tile};
use crate::rng::MsvcRand;

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
    /// 0x5700A0 `Board::IsPowered(Point)`: prédio único (nome em +0x279C, tamanho +0x27AC != 0) está com vida cheia;
    /// os outros, casa povoada (+0x2710) com prédio (terreno 1).
    pub fn is_powered(&self, p: Point) -> bool {
        let t = self.tile(p);
        if !t.unique_building.is_empty() { return t.health == t.max_health; }
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

    /// 0x573EA0 `Board::GetDistanceToBuilding(Point)`: menor distância de Manhattan até um prédio da lista
    /// (`UpdateBuildings`, Board+0x2C44), ou `INT_MAX` sem prédios.
    pub fn get_distance_to_building(&self, p: Point) -> i32 {
        self.buildings.iter().map(|b| (p.x - b.x).abs() + (p.y - b.y).abs()).fold(i32::MAX, i32::min)
    }

    /// 0x573C00 `Board::GetDistanceToPawn(Point, int time)`: mapa de distâncias (0x4CEA50 → Dijkstra 0x4CEC40) a
    /// partir de `p`, com alcance 6 e o próprio `time` como perfil de caminho (os scripts passam 1 ou 6, que viram
    /// voador ou saltador); depois a menor distância até a casa de uma unidade da lista com `IsTeam(time)`, ou
    /// `INT_MAX`.
    pub fn get_distance_to_pawn(&self, p: Point, team: i32) -> i32 {
        let rm = self.reach_map(p, 6, team);
        self.pawn_list.iter().filter(|&&i| self.pawns[i].is_team(team))
            .filter_map(|&i| rm.dist.get(&self.pawns[i].get_space()).copied())
            .fold(i32::MAX, i32::min)
    }

    /// 0x5747C0 `Board::IsDamaged(Point)`: vida atual (+0x78) diferente do máximo (+0x7C).
    pub fn is_damaged(&self, p: Point) -> bool {
        let t = self.tile(p);
        t.health != t.max_health
    }

    /// 0x5739C0 `Board::GetMechDamage()`: soma de máximo − vida de cada mech (+0x9E4) da lista, mortos inclusive.
    /// O máximo vem de 0x6350A0, que antes o recalcula por pilotos (`Zoltan_Skill`); pilotos ainda não existem no Rust.
    pub fn get_mech_damage(&self) -> i32 {
        self.pawn_list.iter().map(|&i| &self.pawns[i]).filter(|p| p.mech).map(|p| p.max_health - p.health).sum()
    }

    /// 0x5744F0 `Board::AddUniqueBuilding(string)`: copia a lista de prédios e, enquanto houver, sorteia
    /// `rand() % n`, tira a casa da cópia e, se ela ainda não tem prédio único, chama
    /// `BoardSpace::SetUniqueBuilding` (0x5A1990) e devolve a casa. Sem nenhuma, (-1, -1).
    pub fn add_unique_building(&mut self, name: &str, rng: &mut MsvcRand) -> Point {
        let mut cand = self.buildings.clone();
        while !cand.is_empty() {
            let k = (rng.rand() % cand.len() as i32) as usize;
            let p = cand.remove(k);
            if self.tile(p).unique_building.is_empty() {
                self.tile_set_unique_building(p, name);
                return p;
            }
        }
        Point::INVALID
    }

    /// 0x5A1990 `BoardSpace::SetUniqueBuilding(string)`: máximo 1, vida atual presa em 0..1, nome em +0x279C. As
    /// imagens `combat/structures/<nome>_on.png` e `_broken.png` (+0x2794, +0x2798) ficam para os gráficos.
    pub fn tile_set_unique_building(&mut self, p: Point, name: &str) {
        let t = self.tile_mut_or_sentinel(p);
        t.max_health = 1;
        t.health = t.health.clamp(0, 1);
        t.unique_building = name.to_string();
    }

    /// 0x565BC0 `Board::MarkSpaceDesc(Point, string)` e 0x565C90 `(Point, string, bool)` → `BoardSpace::MarkDescription`
    /// (0x5AEB00): descrição da casa em +0x26F8 (a mesma que `IsCrackable` compara com "supervolcano") e +0x2BB9 = 0.
    /// O `bool` não é usado.
    pub fn mark_space_desc(&mut self, p: Point, desc: &str) {
        let t = self.tile_mut_or_sentinel(p);
        t.str_0x26f8 = desc.to_string();
        t.byte_0x2bb9 = false;
    }

    /// 0x569850 `Board::IsBusy()` → `GetBusyState` (0x569860): animações, efeitos e filas pendentes. O executor do Rust
    /// termina tudo na hora, então nunca há nada pendente quando um script pergunta.
    pub fn is_busy(&self) -> bool { false }
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
            t.unique_building = "str_bar1".into();
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
    fn distancias() {
        let mut b = OccupancyBoard::new(8, 8).unwrap();
        assert_eq!(b.get_distance_to_building(Point::new(0, 0)), i32::MAX);
        b.tile_mut(Point::new(5, 5)).unwrap().terrain = TERRAIN_BUILDING;
        b.tile_mut(Point::new(1, 6)).unwrap().terrain = TERRAIN_BUILDING;
        b.update_building_list();
        assert_eq!(b.get_distance_to_building(Point::new(0, 0)), 7);
        assert_eq!(b.get_distance_to_pawn(Point::new(0, 0), TEAM_PLAYER), i32::MAX);
        b.add_pawn(pawn("PunchMech", TEAM_PLAYER, Point::new(2, 0))).unwrap();
        b.add_pawn(pawn("Firefly1", TEAM_ENEMY, Point::new(1, 0))).unwrap();
        assert_eq!(b.get_distance_to_pawn(Point::new(0, 0), TEAM_PLAYER), 2);
        // além do alcance 6, a casa fica com INT_MAX
        assert_eq!(b.get_distance_to_pawn(Point::new(0, 7), TEAM_PLAYER), i32::MAX);
    }

    #[test]
    fn predio_unico_e_dano() {
        let mut b = OccupancyBoard::new(4, 1).unwrap();
        for x in [1, 3] {
            let t = b.tile_mut(Point::new(x, 0)).unwrap();
            t.terrain = TERRAIN_BUILDING;
            t.health = 2;
            t.max_health = 2;
        }
        b.update_building_list();
        let mut rng = MsvcRand::new(9);
        let mut copia = rng.clone();
        let p = b.add_unique_building("str_bar1", &mut rng);
        assert_eq!(p, b.buildings[(copia.rand() % 2) as usize]);
        assert_eq!((b.tile(p).health, b.tile(p).max_health, b.tile(p).unique_building.as_str()), (1, 1, "str_bar1"));
        let q = b.add_unique_building("str_bar2", &mut rng);
        assert_ne!(p, q);
        assert_eq!(b.add_unique_building("str_bar3", &mut rng), Point::INVALID);
        assert!(!b.is_damaged(p));
        b.tile_mut(p).unwrap().health = 0;
        assert!(b.is_damaged(p));
        b.add_pawn(PawnState { mech: true, ..pawn("PunchMech", TEAM_PLAYER, Point::new(0, 0)) }).unwrap();
        b.pawns[0].health = 1;
        assert_eq!(b.get_mech_damage(), 1);
        b.mark_space_desc(Point::new(0, 0), "supervolcano");
        assert!(!b.is_crackable(Point::new(0, 0)));
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
