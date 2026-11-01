//! `PAWN_FACTORY`: cria unidades a partir das tabelas Lua de `pawns.lua`. Lido na 1.2.93.
//!
//! `PawnFactory::CreatePawn` 0x6423F0 → `CreateEmptyPawn` 0x642670: tipo inexistente vira "PunchMech", time 2
//! usa `GetDefaultTeam`, o id vem de um contador global (0x88FD64, 4 no arquivo) e "Move" é a primeira arma.
//! As propriedades vêm dos getters `Get<Prop>` gerados por `CreateClass`, como em 0x4492B0 e 0x4493D0.

use crate::values::{arg_int, no_overload};
use crate::world::LPawn;
use itb_core::board_occupancy::consts::TEAM_NONE;
use itb_core::board_occupancy::{OccupancyBoard, PawnState};
use itb_core::pawn_factory::PawnType;
use itb_core::rng::MsvcRand;
use mlua::{Function, Lua, MultiValue, Table, Value};
use std::cell::RefCell;
use std::rc::Rc;

const ID_COUNTER: &str = "itb_pawn_id";
const INITIAL_ID_COUNTER: i32 = 4;

fn prop(ty: &Option<Table>, name: &str) -> mlua::Result<Value> {
    let Some(t) = ty else { return Ok(Value::Nil) };
    match t.get::<Value>(format!("Get{name}"))? {
        Value::Function(f) => f.call(t.clone()),
        _ => Ok(Value::Nil),
    }
}

fn prop_bool(ty: &Option<Table>, name: &str) -> mlua::Result<bool> {
    Ok(matches!(prop(ty, name)?, Value::Boolean(true)))
}

fn prop_int(ty: &Option<Table>, name: &str) -> mlua::Result<i32> {
    Ok(arg_int(&prop(ty, name)?).unwrap_or(0))
}

/// Propriedade do tipo de uma unidade lida na hora, como `LuaData::GetBool`/`GetInt` (0x4493D0, 0x4492B0) sobre o
/// nome do tipo (Pawn+0x94).
pub fn type_bool(lua: &Lua, type_name: &str, name: &str) -> mlua::Result<bool> {
    prop_bool(&lua.globals().get::<Option<Table>>(type_name)?, name)
}

pub fn type_int(lua: &Lua, type_name: &str, name: &str) -> mlua::Result<i32> {
    prop_int(&lua.globals().get::<Option<Table>>(type_name)?, name)
}

fn next_id(lua: &Lua) -> mlua::Result<i32> {
    let id = lua.named_registry_value::<Option<i32>>(ID_COUNTER)?.unwrap_or(INITIAL_ID_COUNTER) + 1;
    lua.set_named_registry_value(ID_COUNTER, id)?;
    Ok(id)
}

/// Criar uma unidade gasta 3 `rand()`: medido em todos os 145 tipos com o teste diferencial. Um deles está em
/// `Pawn::Pawn` (1.2.93: 0x628D9A, grava em +0x924); os outros dois ficam em construtores não localizados.
const RAND_POR_UNIDADE: usize = 3;

/// `PawnFactory::CreatePawn(string, int)`.
pub fn create_pawn(lua: &Lua, requested: &str, team: i32) -> mlua::Result<PawnState> {
    if let Some(rng) = lua.app_data_ref::<Rc<RefCell<MsvcRand>>>() {
        let mut rng = rng.borrow_mut();
        for _ in 0..RAND_POR_UNIDADE { rng.rand(); }
    }
    let globals = lua.globals();
    let (name, ty) = match globals.get::<Value>(requested)? {
        Value::Table(t) => (requested.to_string(), Some(t)),
        _ => ("PunchMech".to_string(), globals.get::<Option<Table>>("PunchMech")?),
    };
    let team = if team == TEAM_NONE { prop_int(&ty, "DefaultTeam")? } else { team };
    let mut skills = Vec::new();
    if let Value::Table(list) = prop(&ty, "SkillList")? {
        for s in list.sequence_values::<String>() { skills.push(s?); }
    }
    let t = PawnType {
        name,
        health: prop_int(&ty, "Health")?,
        move_speed: prop_int(&ty, "MoveSpeed")?,
        corpse: prop_bool(&ty, "Corpse")?,
        flying: prop_bool(&ty, "Flying")?,
        teleporter: prop_bool(&ty, "Teleporter")?,
        jumper: prop_bool(&ty, "Jumper")?,
        burrows: prop_bool(&ty, "Burrows")?,
        massive: prop_bool(&ty, "Massive")?,
        pushable: prop_bool(&ty, "Pushable")?,
        ignore_flip: prop_bool(&ty, "IgnoreFlip")?,
        neutral: prop_bool(&ty, "Neutral")?,
        default_faction: prop_int(&ty, "DefaultFaction")?,
        non_grid: prop_bool(&ty, "NonGrid")?,
        armor: prop_bool(&ty, "Armor")?,
        ignore_fire: prop_bool(&ty, "IgnoreFire")?,
        skills,
    };
    Ok(PawnState::from_type(&t, next_id(lua)?, team))
}

/// A unidade criada ainda não está em tabuleiro: fica numa arena própria até `Board:AddPawn`.
fn detached(state: PawnState) -> mlua::Result<LPawn> {
    let mut limbo = OccupancyBoard::new(1, 1).map_err(|e| mlua::Error::RuntimeError(format!("{e:?}")))?;
    limbo.pawns.push(state);
    Ok(LPawn { board: Rc::new(RefCell::new(limbo)), idx: 0 })
}

/// Liga `PAWN_FACTORY:CreatePawn` no lugar do stub.
pub fn install(lua: &Lua) -> mlua::Result<()> {
    let methods: Table = lua.named_registry_value("itb_methods_PawnFactory")?;
    let create: Function = lua.create_function(|lua, args: MultiValue| {
        let a: Vec<Value> = args.into_iter().skip(1).collect();
        let sig = || no_overload("CreatePawn(string), CreatePawn(string,int)");
        let name = a.first().and_then(crate::records::arg_str).ok_or_else(sig)?;
        let team = match a.get(1) { None => TEAM_NONE, Some(v) => arg_int(v).ok_or_else(sig)? };
        detached(create_pawn(lua, &name, team)?)
    })?;
    methods.set("CreatePawn", create)
}
