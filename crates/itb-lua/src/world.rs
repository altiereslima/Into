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
        // 0x00573160 → 0x004CEC60
        m.add_method("GetSimpleReachable", |_, s, (p, range, corners): (LPoint, Value, Value)| {
            let range = arg_int(&range).ok_or_else(|| no_overload("GetSimpleReachable(Point,int,bool)"))?;
            let corners = match corners { Value::Boolean(b) => b, Value::Nil => false, _ => return Err(no_overload("GetSimpleReachable(Point,int,bool)")) };
            let v = s.0.borrow().get_simple_reachable(to_b(p.0), range, corners);
            Ok(crate::skill::LPointList(v.into_iter().map(from_b).collect()))
        });
        // WeaponTarget slot 0x80 → 0x005731F0 → 0x004CD680
        m.add_method("GetSimplePath", |_, s, (a, b): (LPoint, LPoint)| {
            let v = s.0.borrow().get_simple_path(to_b(a.0), to_b(b.0));
            Ok(crate::skill::LPointList(v.into_iter().map(from_b).collect()))
        });
        q!(m, "IsPod", "IsPod(Point)", |b, p| b.is_pod(p));
        q!(m, "IsItem", "IsItem(Point)", |b, p| b.is_item(p));
        q!(m, "IsDangerous", "IsDangerous(Point)", |b, p| b.is_dangerous(p));
        q!(m, "IsSpawning", "IsSpawning(Point)", |b, p| b.is_spawning(p));
        q!(m, "IsCrackable", "IsCrackable(Point)", |b, p| b.is_crackable(p));
        // WeaponTarget slot 0x7C → 0x0055F0D0 → 0x005AE5D0
        m.add_method("IsTerrain", |_, s, (p, t): (LPoint, Value)| {
            let t = arg_int(&t).ok_or_else(|| no_overload("IsTerrain(Point,int)"))?;
            Ok(s.0.borrow().is_terrain(to_b(p.0), t))
        });
        // 0x00573280 → 0x004CDA00 → Dijkstra 0x004CDE60
        m.add_method("GetReachable", |_, s, (p, range, prof): (LPoint, Value, Value)| {
            let sig = || no_overload("GetReachable(Point,int,int)");
            let range = arg_int(&range).ok_or_else(sig)?;
            let prof = arg_int(&prof).ok_or_else(sig)?;
            let v = s.0.borrow().get_reachable(to_b(p.0), range, prof);
            Ok(crate::skill::LPointList(v.into_iter().map(from_b).collect()))
        });
        // 0x005733D0 → A* 0x004CE630
        m.add_method("GetPath", |_, s, (a, b, prof): (LPoint, LPoint, Value)| {
            let prof = arg_int(&prof).ok_or_else(|| no_overload("GetPath(Point,Point,int)"))?;
            let v = s.0.borrow().get_path(to_b(a.0), to_b(b.0), prof);
            Ok(crate::skill::LPointList(v.into_iter().map(from_b).collect()))
        });
        // 0x005739E0 `IsDeadly(SpaceDamage, Pawn*)` — Pawn pode ser nil (ponteiro nulo).
        m.add_method("IsDeadly", |_, s, (d, pawn): (Value, Value)| {
            let sig = || no_overload("IsDeadly(SpaceDamage,Pawn)");
            let d = crate::skill::damage_arg(&d).ok_or_else(sig)?;
            let attacker = match &pawn {
                Value::Nil => None,
                Value::UserData(u) => {
                    let lp = u.borrow::<LPawn>().map_err(|_| sig())?;
                    if !Rc::ptr_eq(&lp.board, &s.0) { return Err(sig()); }
                    Some(lp.idx)
                }
                _ => return Err(sig()),
            };
            let loc = match d.get("loc") { Some(crate::records::Field::Point(p)) => *p, _ => return Err(sig()) };
            Ok(s.0.borrow().is_deadly(to_b(loc), d.int("iDamage"), attacker))
        });
        // 0x0055F200 DamageSpace(SpaceDamage) / 0x0055F5B0 DamageSpace(Point, int). Os empurrões
        // gerados são resolvidos logo em seguida (o original espera a animação).
        m.add_method("DamageSpace", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let sig = || no_overload("DamageSpace(SpaceDamage), DamageSpace(Point,int)");
            let mut b = s.0.borrow_mut();
            match a.as_slice() {
                [d] => {
                    let rec = crate::skill::damage_arg(d).ok_or_else(sig)?;
                    b.damage_space(&crate::skill::hit_from_record(&rec));
                }
                [p, n] => {
                    let p = pt(p).ok_or_else(sig)?;
                    let n = arg_int(n).ok_or_else(sig)?;
                    if b.is_valid(to_b(p)) {
                        b.tile_apply_damage(to_b(p), &itb_core::apply::Hit::new(BPoint::INVALID, n), 0);
                    }
                }
                _ => return Err(sig()),
            }
            b.resolve_pending_pushes();
            Ok(())
        });
        // 0x0055F7B0 AddEffect(SkillEffect) / 0x0055F970 AddEffect(SpaceDamage): o motor enfileira e roda
        // ao longo dos quadros; o host roda a fila até ficar ocioso, na hora.
        m.add_method("AddEffect", |_, s, v: Value| {
            let sig = || no_overload("AddEffect(SkillEffect), AddEffect(SpaceDamage)");
            let hits: Vec<itb_core::apply::Hit> = match &v {
                Value::UserData(u) if u.is::<crate::skill::LSkillEffect>() => {
                    let fx = u.borrow::<crate::skill::LSkillEffect>()?;
                    let list = fx.effect.borrow();
                    list.iter().map(crate::skill::hit_from_record).collect()
                }
                other => vec![crate::skill::hit_from_record(&crate::skill::damage_arg(other).ok_or_else(sig)?)],
            };
            let mut ex = itb_core::executor::Executor::default();
            ex.add_effect(hits);
            ex.run_until_idle(&mut s.0.borrow_mut());
            Ok(())
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
        m.add_method("IsFire", |_, p, ()| Ok(with(p, |s| s.is_fire())));
        m.add_method("IsFrozen", |_, p, ()| Ok(with(p, |s| s.is_frozen())));
        m.add_method("IsAcid", |_, p, ()| Ok(with(p, |s| s.is_acid())));
        m.add_method("IsInfected", |_, p, ()| Ok(with(p, |s| s.is_infected())));
        m.add_method("IsPowered", |_, p, ()| Ok(with(p, |s| s.is_powered())));
        m.add_method("IsGuarding", |_, p, ()| Ok(with(p, |s| s.is_guarding())));
        m.add_method("GetPathProf", |_, p, ()| Ok(with(p, |s| s.get_path_prof())));
        m.add_method("IsShield", |_, p, ()| Ok(with(p, |s| s.is_shield())));
        m.add_method("IsBoosted", |_, p, ()| Ok(with(p, |s| s.is_boosted())));
        m.add_method("IsFlying", |_, p, ()| Ok(with(p, |s| s.is_flying())));
        m.add_method("IsTeleporter", |_, p, ()| Ok(with(p, |s| s.is_teleporter())));
        m.add_method("IsJumper", |_, p, ()| Ok(with(p, |s| s.is_jumper())));
        m.add_method("IsBurrower", |_, p, ()| Ok(with(p, |s| s.is_burrower())));
        m.add_method("GetBaseMove", |_, p, ()| Ok(with(p, |s| s.base_move_0x63a8f0)));
        m.add_method("IsGrappled", |_, p, ()| Ok(with(p, |s| s.is_grappled())));
        m.add_method("IsAbility", |_, p, name: Value| {
            let n = crate::records::arg_str(&name).ok_or_else(|| no_overload("IsAbility(string)"))?;
            Ok(with(p, |s| s.is_ability(&n)))
        });
        m.add_method("GetMoveSpeed", |_, p, ()| Ok(with(p, |s| s.get_move_speed())));
        m.add_method("IsCorpse", |_, p, ()| {
            let b = p.board.borrow();
            Ok(b.pawns[p.idx].is_corpse(b.global_list_has_12))
        });
        m.add_method("GetType", |_, p, ()| Ok(with(p, |s| s.type_name.clone())));
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
    "GetPawnTeam", "GetSize", "GetSimpleReachable", "GetSimplePath", "GetReachable", "GetPath", "IsDeadly", "IsTerrain", "IsPod", "IsItem",
    "IsDangerous", "IsSpawning", "IsCrackable", "DamageSpace", "AddEffect", "IsPawnSpace", "IsBlocked", "IsPawnTeam", "GetPawn", "IsPawnAlive", "GetPawnSpace",
];
/// Respondidos com dados do estado do jogo (rotina original ainda não reconstruída).
pub const PAWN_HOST: &[&str] = &["GetBaseMove", "IsAbility"];
pub const PAWN_DONE: &[&str] = &[
    "GetHealth", "IsDead", "IsDamaged", "GetTeam", "IsPlayer", "IsEnemy", "GetId", "IsMech", "GetSpace", "SetTeam", "SetMech", "GetPathProf", "IsFire", "IsFrozen", "IsAcid", "IsInfected", "IsPowered", "IsGuarding",
    "IsShield", "IsBoosted", "IsFlying", "GetType", "IsTeleporter", "IsJumper", "IsBurrower", "GetMoveSpeed", "IsCorpse", "IsGrappled",
];
