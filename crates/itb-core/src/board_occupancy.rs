//! Ocupação do tabuleiro por unidades (Pawn) — Etapa 05.
//!
//! Reconstrução semântica, a partir de `Breach.exe` v1.2.88, das rotinas nativas
//! expostas à Lua que respondem "quem está em (x,y)?", "a unidade N está viva?",
//! "esta casa bloqueia o caminho?". Endereços e desmontagens em
//! `evidencias/etapa05_disasm/` e `evidencias/rotinas_ocupacao.md`.
//!
//! O modelo NÃO replica o layout C++. Ponteiros `Pawn*` do EXE viram índices
//! (`PawnIdx`) numa arena; a ordem da arena é a ordem da lista do `PawnHolder`
//! (Board+0x3C..0x40), usada por `GetPawn(id)`. Cada casa guarda a sua própria
//! lista de unidades (tile+0xA0..0xA4), que pode divergir da arena, como no EXE.
//!
//! Dependência parcial declarada: `Pawn::IsCorpse` (0x00629540) chama
//! `Pawn::0x00639E10(12)`, ainda não reconstruída. O resultado dessa chamada é um
//! dado de entrada (`PawnState::query_0x639e10_12`).

use crate::board::BoardBounds;

/// Constantes registradas pelo próprio EXE em `_G` (ver `evidencias/lua_constants.csv`).
pub mod consts {
    pub const TEAM_PLAYER: i32 = 1;
    pub const TEAM_NONE: i32 = 2;
    pub const TEAM_MECH: i32 = 4;
    pub const TEAM_ENEMY: i32 = 6;
    pub const PATH_GROUND: i32 = 0;
    pub const PATH_FLYER: i32 = 1;
    pub const PATH_MASSIVE: i32 = 2;
    pub const PATH_PROJECTILE: i32 = 3;
    pub const PATH_ROADRUNNER: i32 = 4;
    pub const PATH_BURROWER: i32 = 7;
    /// Sem nome Lua; valores que `GetPathProf` usa para teleporte e salto.
    pub const PATH_TELEPORT: i32 = 5;
    pub const PATH_JUMPER: i32 = 6;
    pub const PATH_PHASING: i32 = 9;
    pub const TERRAIN_ROAD: i32 = 0;
    pub const TERRAIN_BUILDING: i32 = 1;
    pub const TERRAIN_RUBBLE: i32 = 2;
    pub const TERRAIN_WATER: i32 = 3;
    pub const TERRAIN_MOUNTAIN: i32 = 4;
    pub const TERRAIN_ICE: i32 = 5;
    pub const TERRAIN_FOREST: i32 = 6;
    pub const TERRAIN_SAND: i32 = 7;
    pub const TERRAIN_HOLE: i32 = 9;
    pub const TERRAIN_FIRE: i32 = 11;
    pub const TERRAIN_ACID: i32 = 12;
    pub const TERRAIN_LAVA: i32 = 14;
}

/// `Point` do motor: dois `int` passados por valor (mesma ABI de `(int x, int y)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self { Self { x, y } }
    /// Valor devolvido por `GetPawnSpace` quando não há unidade (0x00567993).
    pub const INVALID: Point = Point { x: -1, y: -1 };
}

/// Índice de uma unidade na arena (substitui `Pawn*`).
pub type PawnIdx = usize;

/// Campos de `Pawn` lidos pelas rotinas desta etapa (offsets do objeto original).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PawnState {
    /// +0x9A4 — `GetId` (0x006370D0).
    pub id: i32,
    /// +0x0B0 — `GetTeam` (0x00639FC0), `SetTeam` (0x0062F2D0).
    pub team: i32,
    /// +0x8A8 — `GetHealth` (0x004F6410).
    pub health: i32,
    /// +0x8AC — comparado em `IsDamaged` (0x00638D50).
    pub max_health: i32,
    /// +0x8EC/+0x8F0 — `GetSpace` (0x00635FA0, slot 0x28 da vftable de Pawn).
    pub space: Point,
    /// +0x9E4 — `IsMech` (0x00639040), `SetMech` (0x00639050).
    pub mech: bool,
    /// +0xF80 — byte testado por `IsCorpse`; significado ainda não nomeado.
    pub flag_0xf80: bool,
    /// +0x9E0 — inteiro testado por `IsCorpse` contra 2, 3 e 4; não nomeado.
    pub state_0x9e0: i32,
    /// +0x10E8 — inteiro comparado com 12 em `IsCorpse`; não nomeado.
    pub value_0x10e8: i32,
    /// Resultado de `Pawn::0x00639E10(12)` (não reconstruída; dado de entrada).
    pub query_0x639e10_12: bool,
    /// +0x10BC — testado por `IsTeam(8)` (TEAM_BOTS) e `IsTeam(7)`; não nomeado.
    pub field_0x10bc: i32,
    /// +0x97C — gravado por `SetNeutral` (0x0063ABF0); testado por `IsTeam(5)`.
    pub neutral: bool,
    /// Resultado de `Pawn::0x0063A410()` usado por `IsTeam(-1)` (não reconstruída; dado de entrada).
    pub query_0x63a410: bool,
    /// Resultado de `Pawn::IsNonGridStructure()` (0x0063A8B0, ainda não reconstruída) — barra a passagem.
    pub non_grid_structure: bool,
    /// Características de movimento lidas da definição do tipo de unidade (tabelas Lua de pawns.lua)
    /// por `GetPathProf` (0x0062F700): "Teleporter", `IsJumper`, "Burrows", `IsFlying`, "Road_Runner", `IsMassive`.
    pub movement: MovementTraits,
    /// Bytes de estado lidos por acessores Lua: +0x8D0 `IsFire`, +0x8D1 `IsFrozen`, +0x8D3 `IsAcid`,
    /// +0x8DC `IsInfected`, +0x10C0 `IsPowered`, +0x91E (`IsGuarding` devolve `byte == 0`).
    pub status: PawnStatusBytes,
    /// +0xF81 (BYTE): agarrado (`IsGrappled` = byte e não morto); zera `GetMoveSpeed`.
    pub grappled_0xf81: bool,
    /// +0xA64: velocidade fixa (usada por `GetMoveSpeed` se > 0).
    pub move_speed_override_0xa64: i32,
    /// `Pawn:GetBaseMove()` (0x0063A8F0: tipo + bônus de piloto/itens; ainda não reconstruída).
    pub base_move_0x63a8f0: i32,
    /// Habilidades consultadas por `Pawn:IsAbility(nome)` (0x0063AB20 → 0x0061D180; dado de entrada).
    pub abilities: Vec<String>,
    /// +0x978: direção de empurrão pendente (`None` = 4, valor inicial gravado em 0x006275A3).
    pub pending_push_0x978: Option<i32>,
    /// +0x94 `std::string` com o tipo da unidade (ex.: "SatelliteRocket"), lido por `IsCrackable`.
    pub type_name: String,
    /// Entradas de `Pawn::WouldDie` (0x006362A0) e `IsBoosted` (0x006313E0); ver `damage.rs`.
    pub damage: DamageTraits,
    /// Lista de armas do `SkillManager`, com "Move" na primeira posição.
    pub skills: Vec<String>,
    /// +0x91C (BYTE): calculado em `Board::AddPawn`; o significado exato ainda não foi lido.
    pub active_0x91c: bool,
}

/// Campos e consultas de `Pawn` lidos pelo cálculo de dano letal (Etapa 10).
/// Os `query_*` são resultados de rotinas ainda não reconstruídas (dados de entrada).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DamageTraits {
    /// +0x8D5 (BYTE): impulso ("boost") ativo.
    pub boosted_0x8d5: bool,
    /// +0x8D4 (BYTE): escudo.
    pub shield_0x8d4: bool,
    /// +0x99D (BYTE): com 0, gelo rachado sob a unidade a mata (cai na água).
    pub byte_0x99d: u8,
    /// `Pawn::0x00638C70(11)` (habilidade de piloto 11) — usada por `IsBoosted`.
    pub query_0x638c70_11: bool,
    /// `vftable[1]("Arrogant_Boost")` — usada por `IsBoosted` (só vale com vida cheia).
    pub query_arrogant_boost: bool,
    /// `Pawn::0x0062F2F0()` — escudo implícito (mech + byte +0x91D + estado global).
    pub query_0x62f2f0: bool,
    /// `Pawn::0x0063C100()` — blindagem (reduz 1 de dano).
    pub query_0x63c100: bool,
    /// `Pawn::0x0062C5F0()`: passiva "Passive_PlayerTurnShield" ativa nesta unidade (anula dano).
    pub query_0x62c5f0: bool,
    /// `Pawn::0x00638BD0()`: imune a fogo (só pega fogo com "Passive_FireBoost" sendo mech).
    pub query_0x638bd0: bool,
    /// Tipo (+0x874) byte +0x374: ignora fogo por completo.
    pub type_ignores_fire_0x374: bool,
    /// Nomes aceitos por `vftable[1](nome)` (ex.: "Thick", "Retaliation").
    pub vslot1_names: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PawnStatusBytes {
    pub fire: bool,
    pub frozen: bool,
    pub acid: bool,
    pub infected: bool,
    pub powered: bool,
    pub byte_0x91e: u8,
    /// Campo `IgnoreFlip` da definição do tipo. `Pawn::OnInit` grava o inverso em +0x91F (`IsFlippable`).
    pub ignore_flip: bool,
    /// +0x8D6: ferido (`iInjure`).
    pub injured_0x8d6: bool,
    /// +0x921: morreu nesta ação (gravado por `Kill` e `ApplyDamage`).
    pub died_0x921: bool,
}

/// Entradas de `Pawn:GetPathProf()`. Cada uma vem de uma consulta ainda não reconstruída em detalhe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MovementTraits {
    // Valores CRUS das propriedades do tipo; `flying` também vale com o byte +0x1314.
    pub teleporter: bool,
    pub jumper: bool,
    pub burrows: bool,
    pub flying: bool,
    pub road_runner: bool,
    pub massive: bool,
}

impl PawnState {
    /// 0x004F6410 `GetHealth`: `return [this+0x8A8]`.
    pub fn get_health(&self) -> i32 { self.health }
    /// 0x00629530 `IsDead` (vftable slot 0x10): `[this+0x8A8] <= 0`.
    pub fn is_dead(&self) -> bool { self.health <= 0 }
    /// 0x00638D50 `IsDamaged`: `[+0x8A8] != [+0x8AC]` (inclusive se vida > máximo).
    pub fn is_damaged(&self) -> bool { self.health != self.max_health }
    /// `vftable[1](nome)`.
    pub fn vslot1(&self, name: &str) -> bool { self.damage.vslot1_names.contains(&name) }
    /// 0x00639FC0 `GetTeam`.
    pub fn get_team(&self) -> i32 { self.team }
    /// 0x0062F2D0 `SetTeam(int)`: grava sem validar.
    pub fn set_team(&mut self, team: i32) { self.team = team; }
    /// 0x0062F840 `IsPlayer`: `team == 1`.
    pub fn is_player(&self) -> bool { self.team == consts::TEAM_PLAYER }
    /// 0x0062F870 `IsEnemy`: `team == 6`.
    pub fn is_enemy(&self) -> bool { self.team == consts::TEAM_ENEMY }
    /// 0x006370D0 `GetId`.
    pub fn get_id(&self) -> i32 { self.id }
    /// 0x00639040 `IsMech`.
    pub fn is_mech(&self) -> bool { self.mech }
    /// 0x00639050 `SetMech()`: grava 1, sem parâmetro (não existe "desmarcar").
    pub fn set_mech(&mut self) { self.mech = true; }
    /// 0x00635FA0 `GetSpace` (slot 0x28): copia +0x8EC/+0x8F0.
    pub fn get_space(&self) -> Point { self.space }

    /// 0x006398E0 `IsFire`.
    pub fn is_fire(&self) -> bool { self.status.fire }
    /// 0x00636B70 `IsFrozen`.
    pub fn is_frozen(&self) -> bool { self.status.frozen }
    /// 0x006398D0 `IsAcid`.
    pub fn is_acid(&self) -> bool { self.status.acid }
    /// 0x006372F0 `IsInfected`.
    pub fn is_infected(&self) -> bool { self.status.infected }
    /// 0x00637370 `IsPowered`.
    pub fn is_powered(&self) -> bool { self.status.powered }
    /// 0x0063A250 `IsGuarding`: `[+0x91E] == 0` (byte zerado = protegido contra empurrão).
    pub fn is_guarding(&self) -> bool { self.status.byte_0x91e == 0 }

    /// 0x0062F700 `GetPathProf()`: `(time << 4) | perfil`, com perfil por prioridade:
    /// teleporte (5, só se não morto) > saltador (6) > escavador (7) > voador (1) > Road_Runner (4) > massivo (2) > chão (0).
    pub fn get_path_prof(&self) -> i32 {
        let m = &self.movement;
        let base = if m.teleporter && !self.is_dead() { consts::PATH_TELEPORT }
            else if self.is_jumper() { consts::PATH_JUMPER }
            else if m.burrows { consts::PATH_BURROWER }
            else if self.is_flying() { consts::PATH_FLYER }
            else if m.road_runner { consts::PATH_ROADRUNNER }
            else if m.massive { consts::PATH_MASSIVE }
            else { consts::PATH_GROUND };
        self.team.wrapping_shl(4) | base
    }

    /// 0x0063AF60 `Pawn:IsTeleporter()`: propriedade "Teleporter" do tipo e não morto.
    pub fn is_teleporter(&self) -> bool { self.movement.teleporter && !self.is_dead() }
    /// 0x0063AFC0 `Pawn:IsJumper()`: propriedade "Jumper" do tipo e não morto.
    pub fn is_jumper(&self) -> bool { self.movement.jumper && !self.is_dead() }
    /// 0x006365D0 `Pawn:IsBurrower()`: propriedade "Burrows" do tipo (sem checar morte).
    pub fn is_burrower(&self) -> bool { self.movement.burrows }
    /// 0x0063AC00 `Pawn:IsFlying()`: (`IsAbility("Flying")` OU byte +0x1314) e não morto.
    pub fn is_flying(&self) -> bool { self.movement.flying && !self.is_dead() }

    /// 0x00631490 `Pawn:IsGrappled()`: byte +0xF81 e não morto.
    pub fn is_grappled(&self) -> bool { self.grappled_0xf81 && !self.is_dead() }
    /// 0x00637650 `Pawn:IsAbility(string)`.
    pub fn is_ability(&self, name: &str) -> bool { self.abilities.iter().any(|a| a == name) }

    /// 0x0063A860 `Pawn:GetMoveSpeed()`: 0 se morto, agarrado ou sem energia (+0x10C0);
    /// senão +0xA64 se > 0; senão `GetBaseMove()` (0x0063A8F0).
    pub fn get_move_speed(&self) -> i32 {
        if self.is_dead() || self.is_grappled() || !self.status.powered { return 0; }
        if self.move_speed_override_0xa64 > 0 { return self.move_speed_override_0xa64; }
        self.base_move_0x63a8f0
    }

    /// 0x00639FD0 `Pawn::IsTeam(int)` (usada por `Board:IsPawnTeam`).
    pub fn is_team(&self, t: i32) -> bool {
        match t {
            2 => true,                                            // TEAM_NONE/TEAM_ANY
            8 => self.field_0x10bc == 1,                          // TEAM_BOTS
            7 => self.team == consts::TEAM_ENEMY && self.field_0x10bc == 0,
            6 => self.team >= consts::TEAM_ENEMY,                 // TEAM_ENEMY: time >= 6
            -1 => self.is_team(6) && self.query_0x63a410,         // TEAM_ENEMY_MAJOR
            4 => self.mech || self.team == 4,                     // TEAM_MECH
            5 => (!self.neutral && self.team == consts::TEAM_PLAYER) || self.team == 5,
            _ => self.team == t,
        }
    }

    /// 0x00629540 `IsCorpse` — fluxo completo; a chamada 0x00639E10(12) é dado de entrada.
    /// `global_list_has_12`: a lista global de inteiros em 0x008CF84C..0x008CF850 contém 12.
    pub fn is_corpse(&self, global_list_has_12: bool) -> bool {
        if (self.mech || self.flag_0xf80) && !matches!(self.state_0x9e0, 2..=4) {
            return true;
        }
        if self.value_0x10e8 == 12 || global_list_has_12 {
            return self.query_0x639e10_12;
        }
        false
    }
}

/// Casa do tabuleiro: só os campos que as rotinas de ocupação/consulta leem.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Tile {
    /// +0x2AE0 (mesmo campo de `GetTerrain`).
    pub terrain: i32,
    /// +0x2AEC `IsFire` (DWORD != 0).
    pub fire: i32,
    /// +0x2B25 `IsSmoke` (BYTE).
    pub smoke: u8,
    /// +0x2AA0 `IsVines` (BYTE).
    pub vines: u8,
    /// +0x2AF1 `IsAcid` (BYTE).
    pub acid: u8,
    /// +0x2B26 `IsCracked` (BYTE).
    pub cracked: u8,
    /// +0x23AC (BYTE) e +0x23CC (int): exceção da água em `IsPassable` — com o byte ligado e o
    /// contador < 15 a água não barra o movimento. Significado ainda não nomeado.
    pub flag_0x23ac: bool,
    pub counter_0x23cc: i32,
    /// +0x27B4 `std::string`; 0x005AE510 procura "conveyor" nela.
    pub custom_tile: String,
    /// +0x2AC4..+0x2AC7 (BYTE ×4): paredes por direção (UP, RIGHT, DOWN, LEFT) que barram a
    /// SAÍDA da casa para perfis que não sejam voador/projétil/saltador/teleporte (0x00573540).
    pub edge_block_0x2ac4: [bool; 4],
    /// +0x2AC8 (BYTE): água marcada como lava (`IsTerrain(p, TERRAIN_LAVA)`).
    pub lava_0x2ac8: bool,
    /// Objeto de vida da casa (+0x74; vida atual +0x78, máximo +0x7C): prédios, montanhas, gelo.
    /// `IsTerrain(p, 18)` = gelo com vida atual 1. (Etapa 10 chamava este campo de `health_0x7c`.)
    pub health: i32,
    pub max_health: i32,
    /// +0x2AA4: contagem do escudo da casa (prédio/montanha; 2 ao ativar, 0 ao remover).
    pub shield_0x2aa4: i32,
    /// +0x2AA1 (BYTE): prédio/montanha congelado.
    pub frozen_0x2aa1: bool,
    /// +0x2630 (BYTE): casa atingida.
    pub hit_0x2630: bool,
    /// +0x27AC (DWORD): prédio "único" (objetivo) — não vira entulho.
    pub unique_0x27ac: i32,
    /// +0x2710 (BYTE): chave `populated` do mapa; `AddBuilding` também liga. Vale 1 na casa recém-criada.
    pub byte_0x2710: bool,
    /// +0x2640 (BYTE): com ele, a unidade da casa não barra empurrões.
    pub byte_0x2640: bool,
    /// +0x2AE4: 1 na casa da cápsula (`IsPod`).
    pub pod_0x2ae4: i32,
    /// +0x2B04 (DWORD): `IsItem` (!= 0).
    pub item_0x2b04: i32,
    /// +0x2AF0 (BYTE): `IsDangerous`; chave `spikes` nos jogos salvos.
    pub danger_0x2af0: bool,
    /// +0x27E8 (BYTE): `IsSpawning`.
    pub spawning_0x27e8: bool,
    /// +0x26F8 `std::string`: `IsCrackable` recusa "supervolcano".
    pub str_0x26f8: String,
    /// +0x2BA0 (DWORD): `IsCrackable` recusa != 0.
    pub int_0x2ba0: i32,
    /// +0xA0..+0xA4 `std::vector<Pawn*>` na ordem original.
    pub pawns: Vec<PawnIdx>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OccupancyError { InvalidDimensions, OutOfBounds, UnknownPawn }

/// Tabuleiro com unidades. `sentinel` corresponde à casa em Board+0x5C,
/// usada pelo EXE quando a coordenada é inválida.
#[derive(Clone, Debug)]
pub struct OccupancyBoard {
    bounds: BoardBounds,
    tiles: Vec<Tile>,
    pub sentinel: Tile,
    /// Arena de unidades. Os índices ficam estáveis; a ordem do motor está em `pawn_list`.
    pub pawns: Vec<PawnState>,
    /// Lista do PawnHolder (Board+0x3C..0x40), na ordem que `Board::AddPawn` refaz.
    pub pawn_list: Vec<PawnIdx>,
    /// Estado global lido por `IsCorpse` (lista em 0x008CF84C contém 12).
    pub global_list_has_12: bool,
    /// Board+0x2D70/0x2D74: casa da cápsula (`IsPod`).
    pub pod_space: Point,
    /// Board+0x2D50..: lista de casas com surgimento (`IsSpawning`).
    pub spawn_list: Vec<Point>,
    /// Board+0x2D44..: unidades na fila de surgimento, em paralelo com `spawn_list`. Já estão na arena,
    /// mas não na lista do tabuleiro nem em uma casa.
    pub spawn_queue: Vec<PawnIdx>,
    /// Board+0x2CA8: lido por `GetAvailableSpace` e `SpawnQueued`; significado ainda não identificado.
    pub flag_0x2ca8: bool,
    /// Board+0x7460.. e Board+0x7470..: listas de casas perigosas (`IsDangerous`).
    pub danger_list_a: Vec<Point>,
    pub danger_list_b: Vec<Point>,
    /// Passivas do esquadrão para `IsPassiveSkill` lido pelas regras de dano (ex.: "Passive_ForceAmp").
    pub passive_skills: Vec<String>,
    /// Efeitos que o motor original delega a sistemas ainda não reconstruídos (rede elétrica, itens,
    /// cápsulas, criação de unidades...) ou que só têm efeito visual. Ver `apply.rs`.
    pub events: Vec<crate::apply::BoardEvent>,
    /// Unidades que receberam `Pawn::Push` desde a última consulta do executor (o relógio de 0,4 s
    /// do empurrão começa ou recomeça nesse momento).
    pub pushes_started: Vec<PawnIdx>,
    /// Dados da missão carregada de um `.map`.
    pub mission: crate::map::MissionInfo,
}

impl OccupancyBoard {
    pub fn new(width: i32, height: i32) -> Result<Self, OccupancyError> {
        if width <= 0 || height <= 0 || (width as i64) * (height as i64) > 1_048_576 {
            return Err(OccupancyError::InvalidDimensions);
        }
        let len = (width as usize) * (height as usize);
        Ok(Self { bounds: BoardBounds { width, height }, tiles: vec![Tile::default(); len],
                  sentinel: Tile::default(), pawns: Vec::new(), pawn_list: Vec::new(), global_list_has_12: false,
                  pod_space: Point::INVALID, spawn_list: Vec::new(), spawn_queue: Vec::new(), flag_0x2ca8: false, danger_list_a: Vec::new(), danger_list_b: Vec::new(),
                  passive_skills: Vec::new(), events: Vec::new(), pushes_started: Vec::new(), mission: Default::default() })
    }

    /// Board vftable slot 0x08 → 0x00561370 (`IsValid`).
    pub fn is_valid(&self, p: Point) -> bool { self.bounds.is_valid(p.x, p.y) }

    fn index(&self, p: Point) -> Option<usize> {
        self.is_valid(p).then(|| (p.y as usize) * (self.bounds.width as usize) + p.x as usize)
    }

    /// Acesso `[Board+0x50][x].begin + y*0x2BBC`, ou a sentinela Board+0x5C.
    pub fn tile(&self, p: Point) -> &Tile {
        match self.index(p) { Some(i) => &self.tiles[i], None => &self.sentinel }
    }

    pub fn tile_mut(&mut self, p: Point) -> Result<&mut Tile, OccupancyError> {
        let i = self.index(p).ok_or(OccupancyError::OutOfBounds)?;
        Ok(&mut self.tiles[i])
    }

    /// Ajuda de montagem (não é rotina do EXE): cria unidade na arena e a põe na casa.
    pub fn add_pawn(&mut self, pawn: PawnState) -> Result<PawnIdx, OccupancyError> {
        let space = pawn.space;
        let idx = self.pawns.len();
        if self.index(space).is_none() { return Err(OccupancyError::OutOfBounds); }
        self.pawns.push(pawn);
        self.pawn_list.push(idx);
        self.tile_mut(space)?.pawns.push(idx);
        Ok(idx)
    }

    pub fn pawn(&self, idx: PawnIdx) -> &PawnState { &self.pawns[idx] }

    // ----------------------------------------------------------- rotinas da casa

    /// 0x0059F050 `Tile::HasPawn(bool)` (interna, não exposta à Lua).
    /// `false`: lista não vazia. `true`: existe unidade não morta OU cadáver que ocupa.
    pub fn tile_has_pawn(&self, tile: &Tile, check_alive: bool) -> bool {
        if !check_alive { return !tile.pawns.is_empty(); }
        tile.pawns.iter().any(|&i| {
            let p = &self.pawns[i];
            !p.is_dead() || p.is_corpse(self.global_list_has_12)
        })
    }

    /// 0x005AE490 `Tile::GetPawn(bool)`: primeira unidade não morta; se todas
    /// estiverem mortas, devolve a ÚLTIMA da lista (variável sobrescrita no laço).
    pub fn tile_get_pawn(&self, tile: &Tile, check_alive: bool) -> Option<PawnIdx> {
        if !self.tile_has_pawn(tile, check_alive) { return None; }
        let mut last = None;
        for &i in &tile.pawns {
            if !self.pawns[i].is_dead() { return Some(i); }
            last = Some(i);
        }
        last
    }

    /// 0x005AE510 `Tile::IsTerrainBlocked(int pathProf)` (interna).
    pub fn tile_terrain_blocks(tile: &Tile, path_prof: i32) -> bool {
        use consts::*;
        let prof = path_prof & 0xF;
        if prof == PATH_BURROWER && tile.custom_tile.contains("conveyor") { return true; }
        match tile.terrain {
            TERRAIN_BUILDING => prof != PATH_PHASING,
            TERRAIN_WATER => prof == PATH_GROUND || prof == PATH_BURROWER,
            TERRAIN_MOUNTAIN => true,
            TERRAIN_ICE => prof == PATH_BURROWER,
            TERRAIN_HOLE => !matches!(prof, PATH_PHASING | PATH_PROJECTILE | PATH_FLYER | 8 | 5),
            // 2, 6, 7, 8 (tabela) e qualquer valor fora de 1..=9 (ex.: 0, 11, 12, 14).
            _ => false,
        }
    }

    // ----------------------------------------------------------- consultas de casa (Etapas 01–03)
    // Mesma semântica de `board_queries::BoardSnapshot`, agora sobre o modelo unificado.

    /// 0x0055DC10 `GetSize()`: (largura, altura) = [this+0x48], [this+0x4C].
    pub fn get_size(&self) -> Point { Point::new(self.bounds.width, self.bounds.height) }
    /// 0x00503B60 `GetTerrain` (casa sentinela se inválida).
    pub fn get_terrain(&self, p: Point) -> i32 { self.tile(p).terrain }
    /// 0x00567D00 `IsBuilding`: terreno == 1.
    pub fn is_building(&self, p: Point) -> bool { self.get_terrain(p) == consts::TERRAIN_BUILDING }
    /// 0x00567C50 `IsFire`.
    pub fn is_fire(&self, p: Point) -> bool { self.tile(p).fire != 0 }
    /// 0x005653E0 `IsSmoke`.
    pub fn is_smoke(&self, p: Point) -> bool { self.tile(p).smoke != 0 }
    /// 0x005653C0 `IsVines`.
    pub fn is_vines(&self, p: Point) -> bool { self.tile(p).vines != 0 }
    /// 0x00567CB0 `IsAcid`.
    pub fn is_acid(&self, p: Point) -> bool { self.tile(p).acid != 0 }
    /// 0x00569920 `IsCracked`.
    pub fn is_cracked(&self, p: Point) -> bool { self.tile(p).cracked != 0 }
    /// 0x0056FB20 `IsEdge`: sem `IsValid`, compara com 0 e largura/altura − 1.
    pub fn is_edge(&self, p: Point) -> bool {
        p.x == 0 || p.y == 0 || p.x == self.bounds.width.wrapping_sub(1) || p.y == self.bounds.height.wrapping_sub(1)
    }

    // ----------------------------------------------------------- API Lua: Board

    /// 0x0056EE70 `Board:IsPawnSpace(Point, bool)` (WeaponTarget slot 0x4C).
    pub fn is_pawn_space_ex(&self, p: Point, check_alive: bool) -> bool {
        if !self.is_valid(p) { return false; }
        self.tile_has_pawn(self.tile(p), check_alive)
    }

    /// 0x0056EE50 `Board:IsPawnSpace(Point)`: chama o slot 0x4C com `true`.
    pub fn is_pawn_space(&self, p: Point) -> bool { self.is_pawn_space_ex(p, true) }

    /// 0x00570E00 `Board:GetPawn(Point)`: se `IsPawnSpace(p)`, devolve o PRIMEIRO
    /// item da lista da casa (mesmo morto, se outro item satisfez o filtro).
    pub fn get_pawn_at(&self, p: Point) -> Option<PawnIdx> {
        if !self.is_pawn_space(p) { return None; }
        let tile = self.tile(p);
        if !self.tile_has_pawn(tile, true) { return None; }
        tile.pawns.first().copied()
    }

    /// 0x00567750 `Board:GetPawn(Point, bool)`.
    pub fn get_pawn_at_ex(&self, p: Point, check_alive: bool) -> Option<PawnIdx> {
        if !self.is_pawn_space_ex(p, check_alive) { return None; }
        self.tile_get_pawn(self.tile(p), check_alive)
    }

    /// 0x00567700 `Board:GetPawn(int id)` (PawnHolder slot 0): primeira da arena com o id.
    pub fn get_pawn_by_id(&self, id: i32) -> Option<PawnIdx> {
        self.pawn_list.iter().copied().find(|&i| self.pawns[i].id == id)
    }

    /// 0x00567840 `Board:IsPawnAlive(int id)`: existe e `!IsDead()`.
    /// Cadáveres (IsCorpse) contam como NÃO vivos aqui.
    pub fn is_pawn_alive(&self, id: i32) -> bool {
        self.get_pawn_by_id(id).is_some_and(|i| !self.pawns[i].is_dead())
    }

    /// 0x00567970 `Board:GetPawnSpace(int id)`: `GetSpace()` ou (-1,-1).
    /// Usa a posição gravada na unidade, não a casa onde ela está listada.
    pub fn get_pawn_space(&self, id: i32) -> Point {
        self.get_pawn_by_id(id).map_or(Point::INVALID, |i| self.pawns[i].get_space())
    }

    /// 0x00568950 `Board:GetPawnTeam(Point)`: time do PRIMEIRO item da casa
    /// se `HasPawn(true)`; senão 2 (`TEAM_NONE`). `IsValid` só escolhe entre a casa
    /// real e a sentinela (Board+0x5C); não há retorno antecipado.
    pub fn get_pawn_team(&self, p: Point) -> i32 {
        let tile = self.tile(p);
        if self.tile_has_pawn(tile, true) {
            if let Some(&first) = tile.pawns.first() { return self.pawns[first].team; }
        }
        consts::TEAM_NONE
    }

    /// 0x0056ED50 / 0x0056EDA0 `Board:IsPawnTeam(Point, int)`: sem unidade em `IsPawnSpace(p)`
    /// → `team == 2`; senão `GetPawn(p)->IsTeam(team)` (primeiro item da casa).
    pub fn is_pawn_team(&self, p: Point, team: i32) -> bool {
        if !self.is_pawn_space(p) { return team == consts::TEAM_NONE; }
        match self.get_pawn_at(p) { Some(i) => self.pawns[i].is_team(team), None => false }
    }

    /// 0x0055F030 `Board:IsBlocked(Point, int pathProf)`.
    /// Fora do tabuleiro: bloqueado. Terreno: tabela 0x005AE510. Unidade: bloqueia,
    /// exceto para `PATH_PHASING` (9), que só é barrado por `TEAM_ENEMY` (6).
    /// Nota: compara `pathProf` inteiro com 9 (sem a máscara 0xF do terreno).
    pub fn is_blocked(&self, p: Point, path_prof: i32) -> bool {
        if !self.is_valid(p) { return true; }
        if Self::tile_terrain_blocks(self.tile(p), path_prof) { return true; }
        if self.is_pawn_space(p) {
            if path_prof != consts::PATH_PHASING { return true; }
            return self.get_pawn_team(p) == consts::TEAM_ENEMY;
        }
        false
    }
}
