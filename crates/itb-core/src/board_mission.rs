//! Consultas de tabuleiro que os scripts de missão usam — Etapa 13, bloco 1 da fila de uso (`itb-uso`).
//! Lidas na 1.2.93, com o `libmain.so` como guia. Provas em `re/evidencias/v1.2.93/rotinas_fila_uso.md`.

use crate::board_occupancy::{consts::*, OccupancyBoard, PawnIdx, PawnState, Point, Tile};
use crate::board_occupancy::OccupancyError;
use crate::rng::MsvcRand;

/// Tipo que `GetPawnCount(int)` não conta.
const BONUS_DEBRIS: &str = "BonusDebris";

/// Campos do `SpaceDamage` do item (casa +0x2884) que `BoardSpace::IsDangerousItem` (0x5A11E0) testa. Os
/// deslocamentos dentro do `SpaceDamage` vêm do registro luabind das propriedades: `iDamage` +0x08 (casa +0x288C),
/// `iPush` +0x0C, `iShield` +0x10, `iFire` +0x18, `iFrozen` +0x1C, `iSmoke` +0x24, `iAcid` +0x28 e `sPawn` +0xA4
/// (o teste lê o tamanho, +0xB4 = casa +0x2938). O padrão é o de `SpaceDamage()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemDamage {
    pub damage: i32,
    pub push: i32,
    pub shield: i32,
    pub fire: i32,
    pub frozen: i32,
    pub smoke: i32,
    pub acid: i32,
    pub pawn: String,
}

impl Default for ItemDamage {
    fn default() -> Self {
        Self { damage: 0, push: 4, shield: 0, fire: 0, frozen: 0, smoke: 0, acid: 0, pawn: String::new() }
    }
}

impl ItemDamage {
    /// 0x5A11E9..0x5A122F, nesta ordem: `iDamage`, `iPush != 4`, `iShield`, `iFire`, `iSmoke`, `sPawn`, `iAcid`,
    /// `iFrozen` (a mesma ordem de `SpaceDamage::IsActive` no Android).
    pub fn does_something(&self) -> bool {
        self.damage != 0 || self.push != 4 || self.shield != 0 || self.fire != 0 || self.smoke != 0
            || !self.pawn.is_empty() || self.acid != 0 || self.frozen != 0
    }
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

    pub fn tile_is_dangerous_item(t: &Tile) -> bool { !t.item.is_empty() && t.item_damage.does_something() }

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

    /// 0x56CE40 `Board::IsSafe(Point)`: verdadeiro quando nenhum perigo se aplica, nesta ordem: `IsPod`, ácido da casa
    /// (+0x2AF1), `IsFire`, `IsTargeted`, projéteis e alertas pendentes na casa (vetores +0x2644 e +0x7E0, que o executor
    /// do Rust nunca deixa cheios), terreno 3 (+0x2AE0, sem a exceção de `IsTerrain`), `IsSpawning`, `IsSmoke` e o
    /// slot 0x0C, `IsBlocked(p, 0)`. No fim, `!IsTargeted(p)` de novo.
    pub fn is_safe(&self, p: Point) -> bool {
        if self.is_pod(p) || self.tile(p).acid != 0 || self.is_fire(p) || self.is_targeted(p) { return false; }
        if self.tile(p).terrain == TERRAIN_WATER || self.is_spawning(p) || self.is_smoke(p) || self.is_blocked(p, 0) {
            return false;
        }
        !self.is_targeted(p)
    }

    /// 0x56D5C0 `Board::StopWeather()`: `RainCloud::SetWeather(0, 0, (0,0), (8,8), -1.0f)` sobre Board+0x2DA0.
    pub fn stop_weather(&mut self) {
        self.set_weather(0, 0, Point::new(0, 0), Point::new(8, 8), -1.0);
    }

    /// 0x561D20 `Board::RemovePawn(Pawn*)`: marca +0x2630 na casa da unidade e a tira da lista da casa (primeira
    /// ocorrência), limpa o cache do Dijkstra (o Rust não tem), zera o tabuleiro da unidade (+0x944) e a tira da lista
    /// do tabuleiro. As casas extras de unidades grandes (slot 0x14) ainda não existem no Rust.
    pub fn remove_pawn(&mut self, i: PawnIdx) {
        let p = self.pawns[i].get_space();
        let t = self.tile_mut_or_sentinel(p);
        t.hit_0x2630 = true;
        if let Some(k) = t.pawns.iter().position(|&j| j == i) { t.pawns.remove(k); }
        if let Some(k) = self.pawn_list.iter().position(|&j| j == i) { self.pawn_list.remove(k); }
    }

    /// 0x561F00 `Board::RemovePawn(Point)`: enquanto a casa tiver unidade presente (`HasPawn(true)`), tira a primeira
    /// da lista da casa.
    pub fn remove_pawns_at(&mut self, p: Point) {
        while self.tile_has_pawn(self.tile(p), true) {
            let first = self.tile(p).pawns[0];
            self.remove_pawn(first);
        }
    }

    /// 0x5A0330 `BoardSpace::Clear()`: mata as unidades que restam na lista da casa (`Pawn::Kill(false)` embutida),
    /// `OnInit(0, 0)` e esvazia `custom` (+0x27B4).
    pub fn tile_clear(&mut self, p: Point) {
        for i in self.tile(p).pawns.clone() { self.pawn_kill(i); }
        self.tile_on_init(p, TERRAIN_ROAD);
        self.tile_mut_or_sentinel(p).custom_tile.clear();
    }

    /// 0x574450 `Board::SetWall(bool, Point, int dir)`: parede `dir` da casa (+0x2AC4 + dir) e a oposta,
    /// `(dir + 2) % 4`, na vizinha (tabela de direções 0x8D1A40), ou na sentinela.
    pub fn set_wall(&mut self, v: bool, p: Point, dir: usize) {
        self.tile_mut_or_sentinel(p).edge_block_0x2ac4[dir] = v;
        let d = crate::apply::DIR_VEC[dir];
        let n = Point::new(p.x.wrapping_add(d.x), p.y.wrapping_add(d.y));
        self.tile_mut_or_sentinel(n).edge_block_0x2ac4[(dir + 2) % 4] = v;
    }

    /// 0x565D40 `Board::ClearSpace(Point)`: `RemovePawn(p)`, `BoardSpace::Clear`, as 4 paredes desligadas e, se a casa
    /// está na lista de surgimento (Board+0x2D50), sai dela e da fila paralela (+0x2D44), e +0x27E8 = 0.
    pub fn clear_space(&mut self, p: Point) {
        self.remove_pawns_at(p);
        self.tile_clear(p);
        for d in 0..4 { self.set_wall(false, p, d); }
        if let Some(k) = self.spawn_list.iter().position(|&q| q == p) {
            self.spawn_list.remove(k);
            if k < self.spawn_queue.len() { self.spawn_queue.remove(k); }
            self.tile_mut_or_sentinel(p).spawning_0x27e8 = false;
        }
    }

    /// Lista de uma zona como `std::map::operator[]` (0x4C84C0) a lê: zona ausente é inserida vazia, o que muda o
    /// `GetZone` seguinte (sem a zona "enemy" calculada).
    fn zone_entry(&mut self, name: &str) -> Vec<Point> {
        self.mission.zones.entry(name.to_string()).or_default().clone()
    }

    /// 0x56EE40 `Board::AddPawn(Pawn*, string zona)`: casas da zona com `!IsBlocked(p, 0)`, `rand() % n` e
    /// `AddPawn(pawn, casa)`. Sem casa, registra "Zone '%s' was not valid for spawning a unit" e devolve `None`; a
    /// unidade não entra.
    pub fn add_pawn_zone(&mut self, pawn: PawnState, zone: &str, rng: &mut MsvcRand) -> Result<Option<PawnIdx>, OccupancyError> {
        let cand: Vec<Point> = self.zone_entry(zone).into_iter().filter(|&p| !self.is_blocked(p, 0)).collect();
        if cand.is_empty() { return Ok(None); }
        let p = cand[(rng.rand() % cand.len() as i32) as usize];
        self.board_add_pawn_at(pawn, p, rng).map(Some)
    }

    /// 0x5690E0 `Board::SpawnPawn(Pawn*, string zona)`: casas da zona com `!IsBlocked(p, GetPathProf)`, `rand() % n` e
    /// `SpawnPawn(pawn, casa)`; sem casa, `SpawnPawn(pawn, (-1, -1))`.
    pub fn spawn_pawn_zone(&mut self, pawn: PawnState, zone: &str, rng: &mut MsvcRand) -> Result<PawnIdx, OccupancyError> {
        let prof = pawn.get_path_prof();
        let cand: Vec<Point> = self.zone_entry(zone).into_iter().filter(|&p| !self.is_blocked(p, prof)).collect();
        let p = if cand.is_empty() { Point::INVALID } else { cand[(rng.rand() % cand.len() as i32) as usize] };
        self.spawn_pawn(pawn, p, rng)
    }

    /// 0x5727C0 `Board::SetItem(Point, string)` → `BoardSpace::SetItem` (0x5A0C50): com nome não vazio, grava o nome
    /// (+0x2AF4) e o `SpaceDamage` da chave `Damage` da tabela Lua do item (+0x2884; o host o lê). A imagem (`Image`,
    /// +0x2880) fica para os gráficos.
    pub fn set_item(&mut self, p: Point, name: &str, damage: ItemDamage) {
        if name.is_empty() { return; }
        let t = self.tile_mut_or_sentinel(p);
        t.item = name.to_string();
        t.item_damage = damage;
    }

    /// 0x568880 `Board::SetTerrainIcon(Point, string)`: grava o ícone de terreno (+0x2684).
    pub fn set_terrain_icon(&mut self, p: Point, icon: &str) {
        self.tile_mut_or_sentinel(p).terrain_icon = icon.to_string();
    }

    /// 0x573240 `Board::IsEnvironmentDanger(Point)`: a casa tem imagem de aviso (tamanho de +0x265C != 0).
    pub fn is_environment_danger(&self, p: Point) -> bool { !self.tile(p).image_mark.is_empty() }

    /// 0x571800 `Board::MarkSpaceImage(Point, string, GL_Color)`: imagem (+0x265C) e cor (+0x2674) na casa, depois de
    /// chamar `Tutorial::AddTutorial("Tutorial_Environment", x, y)`.
    pub fn mark_space_image(&mut self, p: Point, image: &str, color: [u32; 4]) {
        let t = self.tile_mut_or_sentinel(p);
        t.image_mark = image.to_string();
        t.image_mark_color = color;
    }

    /// 0x5743B0 `Board::IsWall(Point, int dir, int perfil)`: parede na direção (dir 4 = qualquer); com parede, falso
    /// para os perfis 1, 3, 5 e 6 (`perfil & 0xF`) e verdadeiro para os outros.
    pub fn is_wall(&self, p: Point, dir: i32, prof: i32) -> bool {
        let walls = self.tile(p).edge_block_0x2ac4;
        let any = if dir == 4 { walls.iter().any(|&w| w) } else { walls.get(dir as usize).copied().unwrap_or(false) };
        any && !matches!(prof & 0xF, 1 | 3 | 5 | 6)
    }

    /// 0x56C390 `Board::AddTeleport(Point, Point)`: grupo = 1 se a lista de teleportes (Board+0x7470) estava vazia,
    /// senão 2; os dois pontos entram na lista; `BoardSpace::Clear` nas duas casas; e `BoardSpace::SetTeleport(grupo)`
    /// (embutida) grava +0x2BA0 em cada uma, salvo em buraco ou água (com a exceção de `IsTerrain`).
    pub fn add_teleport(&mut self, a: Point, b: Point) {
        let group = if self.teleports.is_empty() { 1 } else { 2 };
        self.teleports.push(a);
        self.teleports.push(b);
        self.tile_clear(a);
        self.tile_clear(b);
        for p in [a, b] {
            let t = self.tile(p);
            if t.terrain != TERRAIN_HOLE && !Self::tile_is_terrain(t, TERRAIN_WATER) {
                self.tile_mut_or_sentinel(p).int_0x2ba0 = group;
            }
        }
    }

    /// 0x565F00 `Board::SetCustomTile(Point, string)`: grava o nome em `custom` (+0x27B4).
    pub fn set_custom_tile(&mut self, p: Point, name: &str) {
        self.tile_mut_or_sentinel(p).custom_tile = name.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn casa_segura() {
        let mut b = OccupancyBoard::new(4, 1).unwrap();
        assert!(b.is_safe(Point::new(0, 0)));
        b.tile_mut(Point::new(0, 0)).unwrap().terrain = TERRAIN_WATER;
        assert!(!b.is_safe(Point::new(0, 0)));
        b.tile_mut(Point::new(1, 0)).unwrap().acid = 1;
        assert!(!b.is_safe(Point::new(1, 0)));
        b.tile_mut(Point::new(2, 0)).unwrap().smoke = 1;
        assert!(!b.is_safe(Point::new(2, 0)));
        b.add_pawn(pawn("Firefly1", TEAM_ENEMY, Point::new(3, 0))).unwrap();
        assert!(!b.is_safe(Point::new(3, 0)));
        b.stop_weather();
        assert_eq!(b.weather.as_ref().map(|w| (w.kind, w.time)), Some((0, -1.0)));
    }

    #[test]
    fn limpar_casa_e_zonas() {
        let mut b = OccupancyBoard::new(8, 8).unwrap();
        let p = Point::new(3, 3);
        b.add_pawn(pawn("Firefly1", TEAM_ENEMY, p)).unwrap();
        b.add_pawn(pawn("Hornet1", TEAM_ENEMY, Point::new(4, 4))).unwrap();
        b.tile_mut(p).unwrap().fire = 1;
        b.set_wall(true, p, 1);
        assert!(b.tile(Point::new(4, 3)).edge_block_0x2ac4[3]);
        b.spawn_list.push(p);
        b.spawn_queue.push(1);
        b.clear_space(p);
        assert!(b.tile(p).pawns.is_empty() && b.tile(p).fire == 0 && !b.tile(p).edge_block_0x2ac4[1]);
        assert!(!b.tile(Point::new(4, 3)).edge_block_0x2ac4[3]);
        assert_eq!(b.pawn_list, vec![1]);
        assert!(b.spawn_list.is_empty() && b.spawn_queue.is_empty());

        // zona ausente: entra vazia no mapa, e GetZone("enemy") deixa de usar o padrão
        let mut rng = MsvcRand::new(5);
        assert_eq!(b.add_pawn_zone(pawn("Firefly1", TEAM_ENEMY, Point::INVALID), "enemy", &mut rng).unwrap(), None);
        assert!(b.get_zone("enemy").is_empty());
        b.mission.zones.insert("z".into(), vec![Point::new(4, 4), Point::new(5, 5), Point::new(6, 6)]);
        let mut copia = rng.clone();
        let i = b.add_pawn_zone(pawn("Firefly1", TEAM_ENEMY, Point::INVALID), "z", &mut rng).unwrap().unwrap();
        // (4,4) está ocupada: sobram (5,5) e (6,6)
        assert_eq!(b.pawns[i].space, [Point::new(5, 5), Point::new(6, 6)][(copia.rand() % 2) as usize]);
    }

    #[test]
    fn paredes_teleportes_e_avisos() {
        let mut b = OccupancyBoard::new(4, 4).unwrap();
        let p = Point::new(1, 1);
        assert!(!b.is_wall(p, 4, 0));
        b.set_wall(true, p, 2);
        assert!(b.is_wall(p, 2, 0) && b.is_wall(p, 4, 2) && !b.is_wall(p, 4, 1) && !b.is_wall(p, 1, 0));
        b.add_pawn(pawn("Firefly1", TEAM_ENEMY, Point::new(2, 2))).unwrap();
        b.add_teleport(Point::new(0, 0), Point::new(2, 2));
        b.add_teleport(Point::new(3, 0), Point::new(3, 3));
        assert_eq!((b.tile(Point::new(0, 0)).int_0x2ba0, b.tile(Point::new(3, 3)).int_0x2ba0), (1, 2));
        assert!(b.pawns[0].is_dead() && b.is_dangerous(Point::new(2, 2)));
        assert!(!b.is_environment_danger(p));
        b.mark_space_image(p, "combat/tile_icon/tile_lightning.png", [0; 4]);
        assert!(b.is_environment_danger(p));
    }

    #[test]
    fn item_perigoso() {
        let mut b = OccupancyBoard::new(1, 1).unwrap();
        let p = Point::new(0, 0);
        assert!(!b.is_dangerous_item(p));
        b.set_item(p, "Item_Mine", ItemDamage::default());
        assert!(b.is_item(p) && !b.is_dangerous_item(p));
        b.set_item(p, "Item_Mine", ItemDamage { damage: 3, ..Default::default() });
        assert!(b.is_dangerous_item(p));
        b.set_item(p, "", ItemDamage::default());
        assert!(b.is_dangerous_item(p), "nome vazio não muda nada");
        b.set_item(p, "Item_Pawn", ItemDamage { pawn: "Wall".into(), ..Default::default() });
        assert!(b.is_dangerous_item(p));
    }
}
