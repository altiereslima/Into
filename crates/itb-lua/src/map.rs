//! Carga dos `maps/*.map`, que são tabelas Lua, como o motor faz.
//!
//! `Board::LoadMapData` (1.2.93: 0x56CF60) executa `maps/maphelper.lua`, depois cada arquivo da pasta, e
//! chama `AddMap(nome)`. `Board::Load` (0x565860) executa `maps/<nome>.map` e passa a global `<nome>` a
//! `Board::LoadScript` (0x562220). Chaves que o host não modela vão para `unsupported`; nos 376 mapas
//! originais não há nenhuma, mas os jogos salvos usam várias.

use crate::host::{Host, Outcome};
use crate::records::arg_str;
use crate::values::{arg_int, LPoint};
use crate::world::SharedBoard;
use itb_core::board_occupancy::consts::{TEAM_NONE, TERRAIN_ACID, TERRAIN_FOREST, TERRAIN_ICE, TERRAIN_SAND};
use itb_core::board_occupancy::{OccupancyBoard, Point};
use itb_core::map::{MapData, MapTile};
use itb_core::mission_setup::{rain_subtype, TERRAIN_CRACK, WEATHER_RAIN};
use mlua::{Function, Table, Value};
use std::cell::RefCell;
use std::rc::Rc;

const TOP_KEYS: &[&str] = &["version", "dimensions", "name", "enemy_kills", "map", "spawns", "spawn_ids", "spawn_points", "zones", "tags",
                            "blocked_points", "blocked_type"];
/// `team` é gravada por `SaveScript`, mas `LoadScript` não a lê.
const TILE_KEYS: &[&str] = &["loc", "terrain", "populated", "fire", "poison", "lava", "custom", "health_max", "health_min", "pawn", "team"];

/// Um mapa lido e as chaves que o host ainda não sabe aplicar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedMap {
    pub data: MapData,
    pub unsupported: Vec<String>,
}

fn bad(what: String) -> mlua::Error { mlua::Error::RuntimeError(what) }

fn point(v: Value) -> mlua::Result<Point> {
    match v {
        Value::UserData(u) => {
            let p = u.borrow::<LPoint>()?;
            Ok(Point::new(p.0.x, p.0.y))
        }
        other => Err(bad(format!("esperava Point, veio {}", other.type_name()))),
    }
}

fn int_or(t: &Table, key: &str, default: i32) -> mlua::Result<i32> {
    match t.get::<Value>(key)? {
        Value::Nil => Ok(default),
        v => arg_int(&v).ok_or_else(|| bad(format!("{key}: esperava número"))),
    }
}

fn str_or_empty(t: &Table, key: &str) -> mlua::Result<String> {
    match t.get::<Value>(key)? {
        Value::Nil => Ok(String::new()),
        v => arg_str(&v).ok_or_else(|| bad(format!("{key}: esperava texto"))),
    }
}

fn points(t: &Table) -> mlua::Result<Vec<Point>> {
    t.clone().sequence_values::<Value>().map(|v| point(v?)).collect()
}

fn read_tile(t: &Table, unsupported: &mut Vec<String>) -> mlua::Result<MapTile> {
    for pair in t.clone().pairs::<String, Value>() {
        let (k, _) = pair?;
        if !TILE_KEYS.contains(&k.as_str()) { unsupported.push(format!("map.{k}")); }
    }
    let health_min = match t.get::<Value>("health_min")? {
        Value::Nil => None,
        v => Some(arg_int(&v).ok_or_else(|| bad("health_min: esperava número".into()))?),
    };
    Ok(MapTile {
        loc: point(t.get("loc")?)?,
        terrain: int_or(t, "terrain", 0)?,
        populated: int_or(t, "populated", 0)?,
        fire: int_or(t, "fire", 0)?,
        poison: int_or(t, "poison", 0)?,
        lava: matches!(t.get::<Value>("lava")?, Value::Boolean(true)),
        custom: str_or_empty(t, "custom")?,
        health_max: int_or(t, "health_max", 2)?,
        health_min,
        pawn: str_or_empty(t, "pawn")?,
    })
}

/// Converte a tabela Lua do mapa em `MapData`.
pub fn read_map(t: &Table) -> mlua::Result<LoadedMap> {
    let mut unsupported = Vec::new();
    for pair in t.clone().pairs::<String, Value>() {
        let (k, _) = pair?;
        if !TOP_KEYS.contains(&k.as_str()) { unsupported.push(k); }
    }
    let mut data = MapData {
        name: str_or_empty(t, "name")?,
        size: point(t.get("dimensions")?)?,
        enemy_kills: int_or(t, "enemy_kills", 0)?,
        ..Default::default()
    };
    if let Some(map) = t.get::<Option<Table>>("map")? {
        for tile in map.sequence_values::<Table>() {
            data.tiles.push(read_tile(&tile?, &mut unsupported)?);
        }
    }
    for key in ["spawns", "spawn_ids", "spawn_points"] {
        if t.get::<Option<Table>>(key)?.is_some_and(|l| l.raw_len() > 0) { unsupported.push(key.to_string()); }
    }
    if let Some(zones) = t.get::<Option<Table>>("zones")? {
        for pair in zones.pairs::<String, Table>() {
            let (name, list) = pair?;
            data.zones.insert(name, points(&list)?);
        }
    }
    if let Some(tags) = t.get::<Option<Table>>("tags")? {
        for tag in tags.sequence_values::<String>() { data.tags.push(tag?); }
    }
    if let (Some(pts), Some(types)) = (t.get::<Option<Table>>("blocked_points")?, t.get::<Option<Table>>("blocked_type")?) {
        let kinds: Vec<i32> = types.sequence_values::<i32>().collect::<mlua::Result<_>>()?;
        data.blocked_spawns = points(&pts)?.into_iter().zip(kinds).collect();
    }
    Ok(LoadedMap { data, unsupported })
}

impl Host {
    fn run_map_file(&self, rel: &str) -> mlua::Result<()> {
        match self.run_file(rel) {
            Outcome::Ok => Ok(()),
            Outcome::Missing => Err(bad(format!("{rel} não existe"))),
            Outcome::Error(e) => Err(bad(format!("{rel}: {e}"))),
        }
    }

    /// `Board::LoadMapData`: devolve os nomes registrados. A ordem é a alfabética, a mesma que o NTFS
    /// costuma dar à listagem do motor (SUPOSIÇÃO; ver docs/DIVERGENCIAS.md).
    pub fn load_map_data(&self) -> mlua::Result<Vec<String>> {
        self.run_map_file("maps/maphelper.lua")?;
        let mut names: Vec<String> = std::fs::read_dir(self.game_dir.join("maps")).map_err(mlua::Error::external)?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_suffix(".map")).map(str::to_string))
            .collect();
        names.sort();
        let add: Function = self.lua.globals().get("AddMap")?;
        for name in &names {
            self.run_map_file(&format!("maps/{name}.map"))?;
            add.call::<()>(name.as_str())?;
        }
        Ok(names)
    }

    /// Executa `maps/<nome>.map` e lê a global `<nome>`.
    pub fn read_map_file(&self, name: &str) -> mlua::Result<LoadedMap> {
        self.run_map_file(&format!("maps/{name}.map"))?;
        let table: Table = self.lua.globals().get(name)?;
        read_map(&table)
    }

    /// `Board::Load` (0x565860): executa o arquivo, chama `AddMap(nome)` e lê a tabela.
    fn load_map_file(&self, name: &str) -> mlua::Result<LoadedMap> {
        self.run_map_file(&format!("maps/{name}.map"))?;
        if let Some(add) = self.lua.globals().get::<Option<Function>>("AddMap")? {
            add.call::<()>(name)?;
        }
        let table: Table = self.lua.globals().get(name)?;
        read_map(&table)
    }

    /// Monta o tabuleiro da missão e o entrega aos scripts com `SetBoard`.
    pub fn load_map(&self, name: &str) -> mlua::Result<(SharedBoard, LoadedMap)> {
        let loaded = self.load_map_file(name)?;
        let mut board = OccupancyBoard::from_map(&loaded.data).map_err(|e| bad(format!("{name}: {e:?}")))?;
        // 0x56EC00 → PawnFactory::CreatePawn(string) com time 2 → Board::AddPawn
        for (loc, ty) in std::mem::take(&mut board.mission.pending_pawns) {
            let pawn = crate::pawns::create_pawn(&self.lua, &ty, TEAM_NONE)?;
            board.board_add_pawn(pawn, loc).map_err(|e| bad(format!("{name}: {e:?}")))?;
        }
        let shared = Rc::new(RefCell::new(board));
        self.set_board(shared.clone())?;
        Ok((shared, loaded))
    }

    /// Chama uma função Lua global como os ajudantes 0x597060 (`(string, int)`) e 0x597190 (`(string)`):
    /// um erro dentro da chamada vira 0, depois da mensagem "Something went wrong".
    fn call_int(&self, func: &str, args: impl mlua::IntoLuaMulti) -> mlua::Result<i32> {
        let f: Function = self.lua.globals().get(func)?;
        match f.call::<Value>(args) {
            Ok(v) => Ok(arg_int(&v).unwrap_or(0)),
            Err(e) => {
                self.report.borrow_mut().console.push(format!("Something went wrong in {func}: {e}"));
                Ok(0)
            }
        }
    }

    /// 0x449640: `CallMethod(obj, func)` pela pilha Lua (`lua_pcall`). Só um booleano conta; erro ou outro
    /// tipo dá falso, depois da mensagem "Something went wrong in <obj>::<func>".
    pub fn call_method_bool(&self, obj: &str, func: &str) -> mlua::Result<bool> {
        let f: Function = self.lua.globals().get("CallMethod")?;
        match f.call::<Value>((obj, func)) {
            Ok(Value::Boolean(b)) => Ok(b),
            Ok(_) | Err(_) => {
                self.report.borrow_mut().console.push(format!("Something went wrong in {obj}::{func}"));
                Ok(false)
            }
        }
    }

    /// `BoardPlayer::SetupBuildingHealth(int setor)` (0x591540): `getBuildingHealth(setor)` vira o vetor
    /// de `Board::SetBuildingHP`.
    pub fn setup_building_health(&self, board: &SharedBoard, sector: i32) -> mlua::Result<()> {
        let f: Function = self.lua.globals().get("getBuildingHealth")?;
        let percent: Vec<i32> = match f.call::<Value>(sector) {
            Ok(Value::Table(t)) => t.sequence_values::<Value>().map(|v| Ok(arg_int(&v?).unwrap_or(0))).collect::<mlua::Result<_>>()?,
            Ok(_) => Vec::new(),
            Err(e) => {
                self.report.borrow_mut().console.push(format!("Something went wrong in getBuildingHealth: {e}"));
                Vec::new()
            }
        };
        board.borrow_mut().set_building_hp(&percent, &mut self.rng.borrow_mut());
        Ok(())
    }

    /// `BoardPlayer::RandomizeEnvironment` (0x58B380). `setor` é a string global 0x8D1A88 ("grass",
    /// "snow"...) e `missao`, o nome que `CallMethod` resolve (BoardPlayer+0xFD4).
    ///
    /// 1. Para floresta, areia, gelo e ácido (tabela 0x8392D0), nesta ordem: `getEnvironmentChance(setor,
    ///    tipo)` e `RandomizeTerrain(tipo, chance)`.
    /// 2. Se `CallMethod(missao, "IsCracksAllowed")`: `RandomizeTerrain(16, getCrackChance(setor))`.
    /// 3. `rand() % 100 < getRainChance(setor)`, sempre gastando o `rand()`: chuva em todo o tabuleiro.
    pub fn randomize_environment(&self, board: &SharedBoard, sector: &str, mission: &str) -> mlua::Result<()> {
        for kind in [TERRAIN_FOREST, TERRAIN_SAND, TERRAIN_ICE, TERRAIN_ACID] {
            let chance = self.call_int("getEnvironmentChance", (sector, kind))?;
            board.borrow_mut().randomize_terrain(kind, chance, &mut self.rng.borrow_mut());
        }
        if self.call_method_bool(mission, "IsCracksAllowed")? {
            let chance = self.call_int("getCrackChance", sector)?;
            board.borrow_mut().randomize_terrain(TERRAIN_CRACK, chance, &mut self.rng.borrow_mut());
        }
        let chance = self.call_int("getRainChance", sector)?;
        if self.rng.borrow_mut().rand() % 100 < chance {
            board.borrow_mut().set_weather(WEATHER_RAIN, rain_subtype(sector), Point::new(0, 0), Point::new(8, 8), -1.0);
        }
        Ok(())
    }

    /// `BoardPlayer::LoadMission` (0x580D50) até `InitializeMission`, exclusive. O mapa vem de
    /// `BoardPlayer::SelectMap` (0x582A60), ainda não reconstruído: quem chama informa o nome.
    pub fn load_mission(&self, map: &str, sector: i32, sector_type: &str, mission: &str) -> mlua::Result<(SharedBoard, LoadedMap)> {
        let (board, loaded) = self.load_map(map)?;
        self.setup_building_health(&board, sector)?;
        if mission != "Mission_ActiveTutorial" {
            self.randomize_environment(&board, sector_type, mission)?;
        }
        Ok((board, loaded))
    }
}
