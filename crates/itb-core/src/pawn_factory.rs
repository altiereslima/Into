//! Criação de unidades e entrada no tabuleiro — Etapa 13. Lido na 1.2.93.
//!
//! `Pawn::OnInit` (0x629700) copia propriedades do tipo para o objeto; o host as lê da tabela Lua e as
//! entrega em `PawnType`. `Board::AddPawn` (0x56E830) põe a unidade na casa e refaz a ordem da lista do
//! tabuleiro. Provas em `re/evidencias/v1.2.93/rotinas_unidades.md`.

use crate::apply::{BoardEvent, Hit};
use crate::board_occupancy::{consts::*, OccupancyBoard, OccupancyError, PawnIdx, PawnState, PawnStatusBytes, Point};
use crate::rng::MsvcRand;

/// Propriedades de um tipo de unidade, nas tabelas de `pawns.lua`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PawnType {
    pub name: String,
    pub health: i32,
    pub move_speed: i32,
    pub corpse: bool,
    pub flying: bool,
    pub teleporter: bool,
    pub jumper: bool,
    pub burrows: bool,
    pub massive: bool,
    pub pushable: bool,
    pub ignore_flip: bool,
    pub neutral: bool,
    pub default_faction: i32,
    pub non_grid: bool,
    pub armor: bool,
    pub ignore_fire: bool,
    /// `GetSkillList`, sem o "Move" que `CreateEmptyPawn` adiciona antes.
    pub skills: Vec<String>,
}

impl PawnState {
    /// `Pawn::Pawn` (0x627F30) + `Pawn::OnInit`.
    pub fn from_type(t: &PawnType, id: i32, team: i32) -> Self {
        let mut skills = vec!["Move".to_string()];
        skills.extend(t.skills.iter().cloned());
        let mut p = PawnState {
            id,
            team,
            type_name: t.name.clone(),
            health: t.health,
            max_health: t.health,
            flag_0xf80: t.corpse,
            state_0x9e0: 5,
            neutral: t.neutral,
            field_0x10bc: t.default_faction,
            non_grid_structure: t.non_grid,
            base_move_0x63a8f0: t.move_speed,
            status: PawnStatusBytes { powered: true, byte_0x91e: t.pushable as u8, ignore_flip: t.ignore_flip, ..Default::default() },
            skills,
            ..Default::default()
        };
        p.movement.flying = t.flying;
        p.movement.teleporter = t.teleporter;
        p.movement.jumper = t.jumper;
        p.movement.burrows = t.burrows;
        p.movement.massive = t.massive;
        p.damage.byte_0x99d = t.massive as u8;
        p.damage.query_0x63c100 = t.armor;
        p.damage.query_0x638bd0 = t.ignore_fire;
        p
    }

    /// `Pawn::SetActive(bool)` 0x634C00: sem efeito se não mudar ou sem energia; unidade que não é mech,
    /// só tem "Move" e não anda fica inativa.
    pub fn set_active(&mut self, active: bool) {
        if active == self.active_0x91c || !self.status.powered { return; }
        self.active_0x91c = if !self.mech && self.skills.len() == 1 && self.get_move_speed() == 0 { false } else { active };
    }

    /// Em qual das três passadas de `Board::AddPawn` a unidade entra na lista nova.
    fn in_list_pass(&self, pass: i32) -> bool {
        if self.team != TEAM_PLAYER { return pass == 1; }
        if self.is_team(8) { return pass == 0; }
        if pass == 0 && !self.neutral { return true; }
        if self.type_name == "Filler_Pawn" { return pass == 0; }
        pass == 2 && self.neutral
    }
}

impl OccupancyBoard {
    /// `Board::AddPawn(Pawn*, Point)`, com a casa já escolhida.
    pub fn board_add_pawn(&mut self, mut pawn: PawnState, loc: Point) -> Result<PawnIdx, OccupancyError> {
        pawn.space = loc;
        pawn.set_active(true);
        let idx = self.add_pawn(pawn)?;
        self.reorder_pawn_list();
        Ok(idx)
    }

    /// `Board::AddPawn` completa: casa inválida usa `GetAvailableSpace`.
    pub fn board_add_pawn_at(&mut self, pawn: PawnState, loc: Point, rng: &mut MsvcRand) -> Result<PawnIdx, OccupancyError> {
        let loc = if self.is_valid(loc) { loc } else { self.get_available_space(&pawn, rng).ok_or(OccupancyError::OutOfBounds)? };
        self.board_add_pawn(pawn, loc)
    }

    /// 0x572E70: a casa serve para esta unidade surgir? `team` 9 relaxa a regra da floresta.
    /// Os times 3 e -3 têm outro caminho, que não foi lido.
    pub fn is_available(&self, team: i32, prof: i32, p: Point) -> bool {
        let t = self.tile(p);
        if t.item_0x2b04 != 0 || t.pod_0x2ae4 == 1 { return false; }
        if matches!(self.mission.block_spawn.get(&p), Some(1 | 2)) { return false; }
        if self.is_dangerous(p) { return false; }
        let side = if team == 9 { TEAM_ENEMY } else { team };
        if side == 3 || side == -3 { return false; }
        if side == TEAM_PLAYER {
            if self.is_blocked(p, prof) || t.acid != 0 || t.danger_0x2af0 || t.fire != 0 || t.terrain == TERRAIN_HOLE { return false; }
        } else {
            if self.is_blocked(p, PATH_GROUND) || t.terrain == TERRAIN_ICE || t.terrain == TERRAIN_WATER { return false; }
            if t.terrain == TERRAIN_FOREST && team != 9 { return false; }
            if t.acid != 0 || t.danger_0x2af0 { return false; }
        }
        !self.spawn_list.contains(&p)
    }

    /// `Board::GetDropZone(false)` 0x560A90: zona "deployment" filtrada; sem ela, colunas 1 a 3 sem as
    /// bordas de cima e de baixo. Casas com mech contam. O caminho para menos de 3 casas não foi lido.
    pub fn get_drop_zone(&self) -> Vec<Point> {
        let usable = |p: Point| self.is_available(TEAM_PLAYER, PATH_MASSIVE, p) || self.get_pawn_at(p).is_some_and(|i| self.pawns[i].mech);
        let zone = self.get_zone("deployment");
        if !zone.is_empty() { return zone.into_iter().filter(|&p| usable(p)).collect(); }
        let s = self.get_size();
        (1..s.x.min(4)).flat_map(|x| (1..s.y - 1).map(move |y| Point::new(x, y))).filter(|&p| usable(p)).collect()
    }

    /// `Board::GetAvailableSpace(Pawn*)` 0x572A00. Sorteia com `rand() % n`. Floresta sorteada vira chão.
    pub fn get_available_space(&mut self, pawn: &PawnState, rng: &mut MsvcRand) -> Option<Point> {
        let (team, prof) = (pawn.team, pawn.get_path_prof());
        let zone = if pawn.is_team(TEAM_PLAYER) { self.get_drop_zone() } else { self.get_zone("enemy") };
        let mut spots: Vec<Point> = zone.iter().copied().filter(|&p| self.is_available(team, prof, p)).collect();
        if spots.is_empty() && !self.flag_0x2ca8 && team != TEAM_PLAYER {
            spots = zone.iter().copied().filter(|&p| self.is_available(9, prof, p)).collect();
        }
        if spots.is_empty() {
            // tabuleiro inteiro, ficando só com a coluna disponível mais à direita; sem mexer na floresta
            let s = self.get_size();
            for x in 0..s.x {
                for y in 0..s.y {
                    let p = Point::new(x, y);
                    if self.is_available(team, prof, p) {
                        if spots.first().is_some_and(|f| f.x < x) { spots.clear(); }
                        spots.push(p);
                    }
                }
            }
            if spots.is_empty() { return None; }
            return Some(spots[rng.rand() as usize % spots.len()]);
        }
        let p = spots[rng.rand() as usize % spots.len()];
        if self.tile(p).terrain == TERRAIN_FOREST { self.set_terrain(p, TERRAIN_ROAD); }
        Some(p)
    }

    /// `Board::SpawnPawn(Pawn*, Point)` 0x568CA0. Inimigo em casa comum entra na fila e surge depois.
    /// Em água, unidade que não voa segue um caminho ainda não lido; aqui ela só entra na casa.
    pub fn spawn_pawn(&mut self, mut pawn: PawnState, loc: Point, rng: &mut MsvcRand) -> Result<PawnIdx, OccupancyError> {
        let terrain = self.tile(loc).terrain;
        if terrain == TERRAIN_HOLE || terrain == TERRAIN_WATER {
            let idx = self.board_add_pawn_at(pawn, loc, rng)?;
            self.pawns[idx].state_0x9e0 = 1;
            return Ok(idx);
        }
        if pawn.team == TEAM_PLAYER {
            let idx = self.board_add_pawn_at(pawn, loc, rng)?;
            self.pawns[idx].set_active(true);
            return Ok(idx);
        }
        // 0x5693A0
        let loc = if self.is_valid(loc) { loc } else { self.get_available_space(&pawn, rng).ok_or(OccupancyError::OutOfBounds)? };
        pawn.space = Point::INVALID;
        let idx = self.pawns.len();
        self.pawns.push(pawn);
        self.spawn_queue.push(idx);
        self.spawn_list.push(loc);
        Ok(idx)
    }

    /// `Board::SpawnQueued()` 0x569410 → `UpdateSpawning(false, false)` 0x566470 até esgotar a fila.
    /// Casa bloqueada: a unidade fica na fila, e quem está na casa toma 1 de dano, salvo mech com
    /// `Passive_Burrows`.
    pub fn spawn_queued(&mut self) {
        if self.flag_0x2ca8 { return; }
        let mut cursor = 0;
        while cursor < self.spawn_queue.len() {
            let (idx, p) = (self.spawn_queue[cursor], self.spawn_list[cursor]);
            if self.is_blocked(p, PATH_GROUND) {
                self.events.push(BoardEvent::EmergeBlocked(p));
                let protected = self.get_pawn_at(p).is_some_and(|i| self.pawns[i].is_team(TEAM_MECH))
                    && self.passive_skills.iter().any(|s| s == "Passive_Burrows");
                if !protected { self.damage_space(&Hit::new(p, 1)); }
                cursor += 1;
                continue;
            }
            self.pawns[idx].set_active(true);
            self.pawns[idx].space = p;
            self.pawn_list.push(idx);
            if let Ok(t) = self.tile_mut(p) {
                t.pawns.push(idx);
                t.spawning_0x27e8 = false;
            }
            self.reorder_pawn_list();
            self.spawn_queue.remove(cursor);
            self.spawn_list.remove(cursor);
        }
    }

    /// 0x573D60: unidades do time na ordem da lista. Morta só entra se for cadáver ou mech.
    pub fn get_pawns(&self, team: i32) -> Vec<PawnIdx> {
        self.pawn_list.iter().copied().filter(|&i| {
            let p = &self.pawns[i];
            (!p.is_dead() || p.is_corpse(self.global_list_has_12) || p.mech) && p.is_team(team)
        }).collect()
    }

    /// `Board::SetBlockSpawn` 0x56C0B0.
    pub fn set_block_spawn(&mut self, p: Point, kind: i32) { self.mission.block_spawn.insert(p, kind); }

    /// `Board::ClearBlockSpawns` 0x56C0E0: tipo 1 volta a 0, e a lista Board+0x7460 é esvaziada.
    pub fn clear_block_spawns(&mut self) {
        for v in self.mission.block_spawn.values_mut() {
            if *v == 1 { *v = 0; }
        }
        self.danger_list_a.clear();
    }

    /// Fim de `Board::AddPawn`: jogador não neutro, bots e `Filler_Pawn`; depois outros times; depois
    /// neutros do time do jogador. Dentro de cada grupo a ordem atual é mantida.
    fn reorder_pawn_list(&mut self) {
        let mut next = Vec::with_capacity(self.pawn_list.len());
        for pass in 0..3 {
            next.extend(self.pawn_list.iter().copied().filter(|&i| self.pawns[i].in_list_pass(pass)));
        }
        self.pawn_list = next;
    }
}
