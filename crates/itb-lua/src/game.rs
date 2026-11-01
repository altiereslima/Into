//! `Game` (classe `GameMap`) exposto à Lua. Métodos ainda não reconstruídos respondem como stubs.

use crate::values::{arg_int, no_overload};
use itb_core::game::GameState;
use mlua::{Lua, MetaMethod, MultiValue, UserData, UserDataMethods, Value};
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
        // As 8 formas (0x60DA30..0x60E0C0) completam `(texto | texto, p1, p2, tipo, a, b, c)` com tipo 0, a 0, b 1 e
        // c 0, e chegam a `BoardPlayer::AddObjective` → `ObjectiveList::AddObjective` (0x5B6A90), que acrescenta a
        // entrada à lista. Sem `BoardPlayer` (GameMap+0xC204), nada. A missão em curso é a do tabuleiro em `Board`.
        m.add_method("AddObjective", |lua, _, args: MultiValue| {
            let sig = || no_overload("AddObjective(string[, string, string][, ObjType[, int, int[, int]]])");
            let a: Vec<Value> = args.into_iter().collect();
            let strings = a.iter().take_while(|v| matches!(v, Value::String(_))).count();
            if strings != 1 && strings != 3 { return Err(sig()); }
            let texts: Vec<String> = a[..strings].iter().filter_map(crate::records::arg_str).collect();
            let ints: Vec<i32> = a[strings..].iter().map(|v| arg_int(v).ok_or_else(sig)).collect::<mlua::Result<_>>()?;
            if !matches!(ints.len(), 0 | 1 | 3 | 4) { return Err(sig()); }
            let get = |i: usize, d: i32| ints.get(i).copied().unwrap_or(d);
            let entry = itb_core::map::ObjectiveEntry { texts, kind: get(0, 0), a: get(1, 0), b: get(2, 1), c: get(3, 0) };
            if let Ok(Value::UserData(u)) = lua.globals().get::<Value>("Board") {
                if let Ok(b) = u.borrow::<crate::world::LBoard>() {
                    b.0.borrow_mut().mission.objectives.push(entry);
                }
            }
            Ok(())
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
pub const GAME_DONE: &[&str] = &["GetSector", "GetTeamTurn", "GetTurnCount", "IsIslandUnlocked", "GetPower", "AddObjective"];
