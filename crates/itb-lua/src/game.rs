//! `Game` (classe `GameMap`) exposto à Lua. Métodos ainda não reconstruídos respondem como stubs.

use crate::values::{arg_int, no_overload};
use itb_core::game::GameState;
use mlua::{Lua, MetaMethod, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub type SharedGame = Rc<RefCell<GameState>>;

#[derive(Clone)]
pub struct LGame(pub SharedGame);

/// `ValueBar` devolvido por `Game:GetPower()`: cópia do objeto de vida.
#[derive(Clone, Copy)]
pub struct LValueBar { pub value: i32, pub max: i32 }

fn stub(lua: &Lua, class: &str, key: &str) -> mlua::Result<Value> {
    match lua.named_registry_value::<Option<mlua::Table>>(&format!("itb_methods_{class}"))? {
        Some(t) => t.get(key),
        None => Ok(Value::Nil),
    }
}

impl UserData for LGame {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("GetSector", |_, g, ()| Ok(g.0.borrow().get_sector()));
        m.add_method("GetTeamTurn", |_, g, ()| Ok(g.0.borrow().get_team_turn()));
        m.add_method("GetTurnCount", |_, g, ()| Ok(g.0.borrow().get_turn_count()));
        m.add_method("IsIslandUnlocked", |_, g, n: Value| {
            let n = arg_int(&n).ok_or_else(|| no_overload("IsIslandUnlocked(int)"))?;
            Ok(g.0.borrow().is_island_unlocked(n))
        });
        m.add_method("GetPower", |_, g, ()| {
            let (value, max) = g.0.borrow().power;
            Ok(LValueBar { value, max })
        });
        m.add_meta_method(MetaMethod::Index, |lua, _, k: String| stub(lua, "GameMap", &k));
    }
}

impl UserData for LValueBar {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("GetValue", |_, v, ()| Ok(v.value));
        m.add_method("GetMax", |_, v, ()| Ok(v.max));
        m.add_meta_method(MetaMethod::Index, |lua, _, k: String| stub(lua, "ValueBar", &k));
    }
}

/// Nomes ligados, para o relatório de cobertura.
pub const GAME_DONE: &[&str] = &["GetSector", "GetTeamTurn", "GetTurnCount", "IsIslandUnlocked", "GetPower"];
