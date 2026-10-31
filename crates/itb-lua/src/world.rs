//! `Board` e `BoardPawn` (Pawn) expostos à Lua, apoiados em `itb_core::board_occupancy`.
//!
//! O motor original entrega o tabuleiro aos scripts com `SetBoard(board)` e a unidade ativa com
//! `SetPawn(pawn)`. Aqui `LBoard` compartilha o mesmo `OccupancyBoard` com as unidades `LPawn`.
//! Métodos ainda não reconstruídos respondem como stubs (registram uso, devolvem nil).

use crate::values::{arg_int, no_overload, LPoint};
use itb_core::board_occupancy::{OccupancyBoard, Point as BPoint};
use itb_core::geometry::IPoint;
use mlua::{IntoLua, Lua, MetaMethod, MultiValue, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub type SharedBoard = Rc<RefCell<OccupancyBoard>>;

fn to_b(p: IPoint) -> BPoint { BPoint::new(p.x, p.y) }
fn from_b(p: BPoint) -> IPoint { IPoint::new(p.x, p.y) }

fn pt(v: &Value) -> Option<IPoint> {
    match v { Value::UserData(u) => u.borrow::<LPoint>().ok().map(|p| p.0), _ => None }
}

/// Aceita `(Point)` ou `(int, int)` — as duas formas registradas para várias consultas.
fn point_or_xy(a: &[Value]) -> Option<(IPoint, usize)> {
    if let Some(p) = a.first().and_then(pt) { return Some((p, 1)); }
    match (a.first().and_then(arg_int), a.get(1).and_then(arg_int)) {
        (Some(x), Some(y)) => Some((IPoint::new(x, y), 2)),
        _ => None,
    }
}

fn stub_lookup(lua: &Lua, class: &str, k: &str) -> mlua::Result<Value> {
    match lua.named_registry_value::<Option<mlua::Table>>(&format!("itb_methods_{class}"))? {
        Some(t) => t.get(k),
        None => Ok(Value::Nil),
    }
}

#[derive(Clone)]
pub struct LBoard(pub SharedBoard);

#[derive(Clone)]
pub struct LPawn {
    pub board: SharedBoard,
    pub idx: usize,
}

impl LBoard {
    fn pawn(&self, idx: Option<usize>) -> Option<LPawn> { idx.map(|idx| LPawn { board: self.0.clone(), idx }) }
}

macro_rules! q {
    ($m:ident, $name:literal, $sig:literal, |$b:ident, $p:ident| $body:expr) => {
        $m.add_method($name, |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let ($p, _) = point_or_xy(&a).ok_or_else(|| no_overload($sig))?;
            let $b = s.0.borrow();
            let $p = to_b($p);
            Ok($body)
        });
    };
}

impl UserData for LBoard {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        q!(m, "IsValid", "IsValid(Point), IsValid(int,int)", |b, p| b.is_valid(p));
        q!(m, "GetTerrain", "GetTerrain(Point), GetTerrain(int,int)", |b, p| b.get_terrain(p));
        q!(m, "IsBuilding", "IsBuilding(Point), IsBuilding(int,int)", |b, p| b.is_building(p));
        q!(m, "IsFire", "IsFire(Point)", |b, p| b.is_fire(p));
        q!(m, "IsSmoke", "IsSmoke(Point)", |b, p| b.is_smoke(p));
        q!(m, "IsVines", "IsVines(Point)", |b, p| b.is_vines(p));
        q!(m, "IsAcid", "IsAcid(Point)", |b, p| b.is_acid(p));
        q!(m, "IsCracked", "IsCracked(Point)", |b, p| b.is_cracked(p));
        q!(m, "IsEdge", "IsEdge(Point)", |b, p| b.is_edge(p));
        q!(m, "GetPawnTeam", "GetPawnTeam(Point)", |b, p| b.get_pawn_team(p));
        m.add_method("GetSize", |_, s, ()| Ok(LPoint(from_b(s.0.borrow().get_size()))));
        m.add_method("IsPawnSpace", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let p = a.first().and_then(pt).ok_or_else(|| no_overload("IsPawnSpace(Point), IsPawnSpace(Point,bool)"))?;
            let b = s.0.borrow();
            Ok(match a.get(1) {
                None => b.is_pawn_space(to_b(p)),
                Some(Value::Boolean(alive)) => b.is_pawn_space_ex(to_b(p), *alive),
                _ => return Err(no_overload("IsPawnSpace(Point,bool)")),
            })
        });
        m.add_method("IsBlocked", |_, s, (p, prof): (LPoint, Value)| {
            let prof = arg_int(&prof).ok_or_else(|| no_overload("IsBlocked(Point,int)"))?;
            Ok(s.0.borrow().is_blocked(to_b(p.0), prof))
        });
        m.add_method("IsPawnTeam", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let (p, used) = point_or_xy(&a).ok_or_else(|| no_overload("IsPawnTeam(Point,int), IsPawnTeam(int,int,int)"))?;
            let team = a.get(used).and_then(arg_int).ok_or_else(|| no_overload("IsPawnTeam(Point,int)"))?;
            Ok(s.0.borrow().is_pawn_team(to_b(p), team))
        });
        m.add_method("GetPawn", |lua, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let idx = {
                let b = s.0.borrow();
                match a.as_slice() {
                    [p] if pt(p).is_some() => b.get_pawn_at(to_b(pt(p).unwrap())),
                    [p, Value::Boolean(alive)] if pt(p).is_some() => b.get_pawn_at_ex(to_b(pt(p).unwrap()), *alive),
                    [id] => b.get_pawn_by_id(arg_int(id).ok_or_else(|| no_overload("GetPawn(int)"))?),
                    _ => return Err(no_overload("GetPawn(Point), GetPawn(Point,bool), GetPawn(int)")),
                }
            };
            match s.pawn(idx) { Some(p) => p.into_lua(lua), None => Ok(Value::Nil) }
        });
        m.add_method("IsPawnAlive", |_, s, id: Value| {
            Ok(s.0.borrow().is_pawn_alive(arg_int(&id).ok_or_else(|| no_overload("IsPawnAlive(int)"))?))
        });
        m.add_method("GetPawnSpace", |_, s, id: Value| {
            Ok(LPoint(from_b(s.0.borrow().get_pawn_space(arg_int(&id).ok_or_else(|| no_overload("GetPawnSpace(int)"))?))))
        });
        m.add_meta_method(MetaMethod::Index, |lua, _, k: String| stub_lookup(lua, "Board", &k));
    }
}

impl UserData for LPawn {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        fn with<R>(p: &LPawn, f: impl FnOnce(&itb_core::board_occupancy::PawnState) -> R) -> R {
            f(&p.board.borrow().pawns[p.idx])
        }
        m.add_method("GetHealth", |_, p, ()| Ok(with(p, |s| s.get_health())));
        m.add_method("IsDead", |_, p, ()| Ok(with(p, |s| s.is_dead())));
        m.add_method("IsDamaged", |_, p, ()| Ok(with(p, |s| s.is_damaged())));
        m.add_method("GetTeam", |_, p, ()| Ok(with(p, |s| s.get_team())));
        m.add_method("IsPlayer", |_, p, ()| Ok(with(p, |s| s.is_player())));
        m.add_method("IsEnemy", |_, p, ()| Ok(with(p, |s| s.is_enemy())));
        m.add_method("GetId", |_, p, ()| Ok(with(p, |s| s.get_id())));
        m.add_method("IsMech", |_, p, ()| Ok(with(p, |s| s.is_mech())));
        m.add_method("GetSpace", |_, p, ()| Ok(LPoint(from_b(with(p, |s| s.get_space())))));
        m.add_method("SetTeam", |_, p, t: Value| {
            let t = arg_int(&t).ok_or_else(|| no_overload("SetTeam(int)"))?;
            p.board.borrow_mut().pawns[p.idx].set_team(t);
            Ok(())
        });
        m.add_method("SetMech", |_, p, ()| { p.board.borrow_mut().pawns[p.idx].set_mech(); Ok(()) });
        m.add_meta_method(MetaMethod::Eq, |_, a, b: LPawnRef| Ok(Rc::ptr_eq(&a.board, &b.0.board) && a.idx == b.0.idx));
        m.add_meta_method(MetaMethod::Index, |lua, _, k: String| stub_lookup(lua, "BoardPawn", &k));
    }
}

/// Auxiliar para comparar pawns (`pawn1 == pawn2`).
pub struct LPawnRef(pub LPawn);

impl mlua::FromLua for LPawnRef {
    fn from_lua(v: Value, _: &Lua) -> mlua::Result<Self> {
        match v {
            Value::UserData(u) => Ok(LPawnRef(u.borrow::<LPawn>()?.clone())),
            o => Err(mlua::Error::FromLuaConversionError { from: o.type_name(), to: "BoardPawn".into(), message: None }),
        }
    }
}

/// Nomes ligados nesta etapa (para o relatório de cobertura).
pub const BOARD_DONE: &[&str] = &[
    "IsValid", "GetTerrain", "IsBuilding", "IsFire", "IsSmoke", "IsVines", "IsAcid", "IsCracked", "IsEdge",
    "GetPawnTeam", "GetSize", "IsPawnSpace", "IsBlocked", "IsPawnTeam", "GetPawn", "IsPawnAlive", "GetPawnSpace",
];
pub const PAWN_DONE: &[&str] = &[
    "GetHealth", "IsDead", "IsDamaged", "GetTeam", "IsPlayer", "IsEnemy", "GetId", "IsMech", "GetSpace", "SetTeam", "SetMech",
];
