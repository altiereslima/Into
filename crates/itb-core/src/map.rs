//! Mapa de missão aplicado ao tabuleiro — Etapa 13.
//!
//! Os `maps/<nome>.map` são tabelas Lua (o host as lê; ver `itb-lua/src/map.rs`). Esta parte reproduz o
//! efeito de `Board::LoadScript` (1.2.93: 0x562220), na ordem do laço das casas:
//! `loc`, `pawn` (guardado), `terrain` (0x5A0270: limpa a casa e chama `SetTerrain`), `populated`, `fire`,
//! `poison`, `lava`, `custom`, `health_max`, `health_min`, unidade. As outras chaves do laço (`cracked`,
//! `vines`, `smoke`, `item`...) só aparecem em jogos salvos, não nos mapas.
//! Provas em `re/evidencias/v1.2.93/rotinas_mapa.md`.

use crate::board_occupancy::{consts::*, OccupancyBoard, OccupancyError, Point};
use std::collections::BTreeMap;

/// Valores de `BoardSpace::BoardSpace` (1.2.93: 0x599D70) para casas que o mapa não lista.
const INITIAL_HEALTH: i32 = 4;
/// Padrão de `health_max` em `LoadScript` (0x56309C).
const DEFAULT_HEALTH_MAX: i32 = 2;

/// Uma entrada de `["map"]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapTile {
    pub loc: Point,
    pub terrain: i32,
    pub populated: i32,
    pub fire: i32,
    pub poison: i32,
    pub lava: bool,
    pub custom: String,
    pub health_max: i32,
    /// Vida atual da casa; o padrão é `health_max`.
    pub health_min: Option<i32>,
    /// Tipo da unidade que nasce na casa. A chave `team` do arquivo não é lida por `LoadScript`.
    pub pawn: String,
}

impl MapTile {
    pub fn new(loc: Point) -> Self {
        Self { loc, terrain: TERRAIN_ROAD, populated: 0, fire: 0, poison: 0, lava: false, custom: String::new(),
               health_max: DEFAULT_HEALTH_MAX, health_min: None, pawn: String::new() }
    }
}

/// Conteúdo de um `.map`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MapData {
    pub name: String,
    pub size: Point,
    pub enemy_kills: i32,
    pub tiles: Vec<MapTile>,
    pub zones: BTreeMap<String, Vec<Point>>,
    pub tags: Vec<String>,
    pub blocked_spawns: Vec<(Point, i32)>,
}

/// Dados de missão guardados no tabuleiro.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MissionInfo {
    /// Board+0x2C1C.
    pub name: String,
    /// Board+0x7504.
    pub enemy_kills: i32,
    /// Board+0x2C90, lido por `Board:GetZone`.
    pub zones: BTreeMap<String, Vec<Point>>,
    pub tags: Vec<String>,
    /// Board+0x7458, `std::map` gravado por `Board::SetBlockSpawn(Point, int)`.
    pub block_spawn: BTreeMap<Point, i32>,
    /// Unidades pedidas pelo mapa (0x56EC00), na ordem do laço. O host as cria com a fábrica de unidades,
    /// que depende das tabelas Lua, e esvazia esta lista.
    pub pending_pawns: Vec<(Point, String)>,
}

impl OccupancyBoard {
    /// `Board:GetZone` (1.2.93: 0x56D960 → 0x56DEA0): cópia da lista. Zona ausente dá lista vazia, menos
    /// "enemy": as 3 últimas colunas, linhas 2 a 5, em ordem de coluna.
    pub fn get_zone(&self, name: &str) -> Vec<Point> {
        if let Some(list) = self.mission.zones.get(name) { return list.clone(); }
        if name != "enemy" { return Vec::new(); }
        let w = self.get_size().x;
        (w - 3..w).flat_map(|x| (2..6).map(move |y| Point::new(x, y))).collect()
    }

    /// Monta o tabuleiro de uma missão a partir de um mapa.
    pub fn from_map(map: &MapData) -> Result<Self, OccupancyError> {
        let mut b = Self::new(map.size.x, map.size.y)?;
        for y in 0..map.size.y {
            for x in 0..map.size.x {
                let t = b.tile_mut(Point::new(x, y))?;
                t.health = INITIAL_HEALTH;
                t.max_health = INITIAL_HEALTH;
                t.byte_0x2710 = true;
            }
        }
        b.mission = MissionInfo { name: map.name.clone(), enemy_kills: map.enemy_kills, zones: map.zones.clone(),
                                  tags: map.tags.clone(), block_spawn: map.blocked_spawns.iter().copied().collect(), pending_pawns: Vec::new() };
        for tile in &map.tiles {
            b.load_map_tile(tile);
        }
        // `Board::Load` (0x565860): depois de `LoadScript`, 0x573A20 guarda o terreno das vizinhas (só gráficos)
        // e 0x573260 refaz a lista de prédios.
        b.update_building_list();
        Ok(b)
    }

    fn load_map_tile(&mut self, m: &MapTile) {
        let p = m.loc;
        // 0x5A0270: chama SetTerrain e depois limpa o estado da casa
        self.tile_mut_or_sentinel(p).pawns.clear();
        self.set_terrain(p, m.terrain);
        let t = self.tile_mut_or_sentinel(p);
        t.shield_0x2aa4 = 0;
        t.vines = 0;
        t.frozen_0x2aa1 = false;
        t.fire = 0;
        t.pod_0x2ae4 = 0;
        t.acid = 0;
        t.smoke = 0;
        t.cracked = 0;
        t.item_0x2b04 = 0;
        t.byte_0x2710 = m.populated == 1;
        t.fire = m.fire;
        self.tile_set_acid(p, m.poison == 1);
        self.tile_mut_or_sentinel(p).lava_0x2ac8 = m.lava;
        if m.lava {
            self.set_terrain(p, TERRAIN_WATER);
            self.tile_mut_or_sentinel(p).fire = 0;
        }
        self.tile_mut_or_sentinel(p).custom_tile = m.custom.clone();
        let current = m.health_min.unwrap_or(m.health_max).min(m.health_max);
        self.tile_set_health(p, current, m.health_max);
        if !m.pawn.is_empty() {
            self.mission.pending_pawns.push((p, m.pawn.clone()));
        }
    }

    /// 0x5AF190 (1.2.93): grava a vida e ajusta por terreno. Entulho fica com no máximo 0; montanha tem
    /// máximo 2; prédio único, máximo 1.
    pub(crate) fn tile_set_health(&mut self, p: Point, current: i32, max: i32) {
        let t = self.tile_mut_or_sentinel(p);
        t.health = current;
        t.max_health = max;
        if t.terrain == TERRAIN_RUBBLE {
            t.health = t.max_health.min(0);
        }
        if t.terrain == TERRAIN_MOUNTAIN {
            t.max_health = 2;
            t.health = t.health.clamp(0, 2);
        }
        if t.unique_0x27ac != 0 {
            t.max_health = 1;
            t.health = t.health.clamp(0, 1);
        }
    }
}
