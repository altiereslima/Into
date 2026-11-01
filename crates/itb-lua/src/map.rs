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
use itb_core::board_occupancy::consts::TEAM_NONE;
use itb_core::board_occupancy::{OccupancyBoard, Point};
use itb_core::map::{MapData, MapTile};
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

    /// `Board::Load`: executa `maps/<nome>.map` e lê a global `<nome>`.
    pub fn read_map_file(&self, name: &str) -> mlua::Result<LoadedMap> {
        self.run_map_file(&format!("maps/{name}.map"))?;
        let table: Table = self.lua.globals().get(name)?;
        read_map(&table)
    }

    /// Monta o tabuleiro da missão e o entrega aos scripts com `SetBoard`.
    pub fn load_map(&self, name: &str) -> mlua::Result<(SharedBoard, LoadedMap)> {
        let loaded = self.read_map_file(name)?;
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
}
