//! `Board` e `BoardPawn` (Pawn) expostos à Lua, apoiados em `itb_core::board_occupancy`.
//!
//! O motor original entrega o tabuleiro aos scripts com `SetBoard(board)` e a unidade ativa com
//! `SetPawn(pawn)`. Aqui `LBoard` compartilha o mesmo `OccupancyBoard` com as unidades `LPawn`.
//! Métodos ainda não reconstruídos respondem como stubs (registram uso, devolvem nil).

use crate::values::{arg_f32, arg_int, no_overload, LPoint};
use itb_core::board_occupancy::{consts::TEAM_NONE, OccupancyBoard, OccupancyError, Point as BPoint};
use itb_core::geometry::IPoint;
use itb_core::rng::MsvcRand;
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

/// Casa pedida por `AddPawn`/`SpawnPawn`: `(x)` usa (-1,-1), `(x, Point)` usa o ponto. `None` para as
/// variantes com nome de zona, ainda não reconstruídas.
fn spawn_target(a: &[Value]) -> Option<IPoint> {
    match a {
        [_] => Some(IPoint::new(-1, -1)),
        [_, p] => pt(p),
        _ => None,
    }
}

fn board_stub(lua: &Lua, name: &str) -> mlua::Result<Value> {
    match stub_lookup(lua, "Board", name)? {
        Value::Function(f) => f.call(()),
        _ => Ok(Value::Nil),
    }
}

fn rng_of(lua: &Lua) -> mlua::Result<Rc<RefCell<MsvcRand>>> {
    lua.app_data_ref::<Rc<RefCell<MsvcRand>>>().map(|r| r.clone())
        .ok_or_else(|| mlua::Error::RuntimeError("rand() do host não instalado".into()))
}

/// Copia a unidade recém-criada para entrar num tabuleiro.
fn detach(pawn: &LPawn, board: &LBoard) -> mlua::Result<itb_core::board_occupancy::PawnState> {
    if Rc::ptr_eq(&pawn.board, &board.0) {
        return Err(mlua::Error::RuntimeError("a unidade já está neste tabuleiro".into()));
    }
    let state = pawn.board.borrow().pawns[pawn.idx].clone();
    Ok(state)
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
        // 1.2.93: (Pawn*, Point) 0x56E830, (Pawn*) 0x56ECC0, (string, Point) 0x56EC00 e (string) 0x56EFE0
        // devolvem a casa final. As variantes com nome de zona continuam stub.
        m.add_method("AddPawn", |lua, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let Some(target) = spawn_target(&a) else { return board_stub(lua, "AddPawn") };
            let rng = rng_of(lua)?;
            let fail = |e: OccupancyError| mlua::Error::RuntimeError(format!("AddPawn: {e:?}"));
            let idx = match &a[0] {
                Value::UserData(u) if u.is::<LPawn>() => {
                    let mut pawn = u.borrow_mut::<LPawn>()?;
                    let state = detach(&pawn, s)?;
                    let idx = s.0.borrow_mut().board_add_pawn_at(state, to_b(target), &mut rng.borrow_mut()).map_err(fail)?;
                    pawn.board = s.0.clone();
                    pawn.idx = idx;
                    idx
                }
                name => {
                    let state = crate::pawns::create_pawn(lua, &crate::records::arg_str(name).ok_or_else(|| no_overload("AddPawn"))?, TEAM_NONE)?;
                    s.0.borrow_mut().board_add_pawn_at(state, to_b(target), &mut rng.borrow_mut()).map_err(fail)?
                }
            };
            let space = s.0.borrow().pawns[idx].space;
            LPoint(from_b(space)).into_lua(lua)
        });
        // 1.2.93: (Pawn*, Point) 0x568CA0, (Pawn*) 0x5690C0 e, com o id de volta, (string, Point) 0x568F30 e
        // (string) 0x569030. As variantes com nome de zona continuam stub.
        m.add_method("SpawnPawn", |lua, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let Some(target) = spawn_target(&a) else { return board_stub(lua, "SpawnPawn") };
            let rng = rng_of(lua)?;
            let fail = |e: OccupancyError| mlua::Error::RuntimeError(format!("SpawnPawn: {e:?}"));
            match &a[0] {
                Value::UserData(u) if u.is::<LPawn>() => {
                    let mut pawn = u.borrow_mut::<LPawn>()?;
                    let state = detach(&pawn, s)?;
                    let idx = s.0.borrow_mut().spawn_pawn(state, to_b(target), &mut rng.borrow_mut()).map_err(fail)?;
                    pawn.board = s.0.clone();
                    pawn.idx = idx;
                    Ok(Value::Nil)
                }
                name => {
                    let state = crate::pawns::create_pawn(lua, &crate::records::arg_str(name).ok_or_else(|| no_overload("SpawnPawn"))?, TEAM_NONE)?;
                    let id = state.id;
                    s.0.borrow_mut().spawn_pawn(state, to_b(target), &mut rng.borrow_mut()).map_err(fail)?;
                    id.into_lua(lua)
                }
            }
        });
        m.add_method("SpawnQueued", |_, s, ()| { s.0.borrow_mut().spawn_queued(); Ok(()) });
        // 0x566310: ids, na ordem da lista do tabuleiro.
        m.add_method("GetPawns", |_, s, team: Value| {
            let team = arg_int(&team).ok_or_else(|| no_overload("GetPawns(int)"))?;
            let b = s.0.borrow();
            Ok(crate::skill::LIntList(b.get_pawns(team).into_iter().map(|i| b.pawns[i].id).collect()))
        });
        // 0x565EE0 → `Board::SetTerrain(int, int, int)` 0x565E90 → `BoardSpace::SetTerrain`, depois
        // `Board::UpdateBuildings` refaz a lista de prédios.
        m.add_method("SetTerrain", |_, s, (p, t): (LPoint, Value)| {
            let t = arg_int(&t).ok_or_else(|| no_overload("SetTerrain(Point,int)"))?;
            let mut b = s.0.borrow_mut();
            b.set_terrain(to_b(p.0), t);
            b.update_building_list();
            Ok(())
        });
        m.add_method("GetHealth", |_, s, p: LPoint| Ok(s.0.borrow().get_health(to_b(p.0))));
        // Etapa 13, bloco 1 da fila de uso (`board_mission.rs`)
        q!(m, "IsPowered", "IsPowered(Point)", |b, p| b.is_powered(p));
        q!(m, "IsFrozen", "IsFrozen(Point)", |b, p| b.is_frozen_at(p));
        q!(m, "IsDangerousItem", "IsDangerousItem(Point)", |b, p| b.is_dangerous_item(p));
        q!(m, "IsTargeted", "IsTargeted(Point)", |b, p| b.is_targeted(p));
        m.add_method("GetEnemyCount", |_, s, ()| Ok(s.0.borrow().get_enemy_count()));
        // bloco 2
        m.add_method("GetDistanceToBuilding", |_, s, p: LPoint| Ok(s.0.borrow().get_distance_to_building(to_b(p.0))));
        m.add_method("GetDistanceToPawn", |_, s, (p, team): (LPoint, Value)| {
            let team = arg_int(&team).ok_or_else(|| no_overload("GetDistanceToPawn(Point,int)"))?;
            Ok(s.0.borrow().get_distance_to_pawn(to_b(p.0), team))
        });
        m.add_method("IsDamaged", |_, s, p: LPoint| Ok(s.0.borrow().is_damaged(to_b(p.0))));
        m.add_method("GetMechDamage", |_, s, ()| Ok(s.0.borrow().get_mech_damage()));
        m.add_method("AddUniqueBuilding", |lua, s, name: Value| {
            let name = crate::records::arg_str(&name).ok_or_else(|| no_overload("AddUniqueBuilding(string)"))?;
            let rng = rng_of(lua)?;
            let p = s.0.borrow_mut().add_unique_building(&name, &mut rng.borrow_mut());
            Ok(LPoint(from_b(p)))
        });
        m.add_method("MarkSpaceDesc", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let sig = || no_overload("MarkSpaceDesc(Point,string), MarkSpaceDesc(Point,string,bool)");
            let (p, desc) = match a.as_slice() {
                [p, d] | [p, d, Value::Boolean(_)] => (pt(p).ok_or_else(sig)?, crate::records::arg_str(d).ok_or_else(sig)?),
                _ => return Err(sig()),
            };
            s.0.borrow_mut().mark_space_desc(to_b(p), &desc);
            Ok(())
        });
        m.add_method("IsBusy", |_, s, ()| Ok(s.0.borrow().is_busy()));
        // bloco 3
        q!(m, "IsSafe", "IsSafe(Point)", |b, p| b.is_safe(p));
        m.add_method("StopWeather", |_, s, ()| { s.0.borrow_mut().stop_weather(); Ok(()) });
        // 0x5663F0: cópia da lista de prédios (Board+0x2C44)
        m.add_method("GetBuildings", |_, s, ()| {
            Ok(crate::skill::LPointList(s.0.borrow().buildings.iter().map(|&p| from_b(p)).collect()))
        });
        m.add_method("SetCustomTile", |_, s, (p, name): (LPoint, Value)| {
            let name = crate::records::arg_str(&name).ok_or_else(|| no_overload("SetCustomTile(Point,string)"))?;
            s.0.borrow_mut().set_custom_tile(to_b(p.0), &name);
            Ok(())
        });
        // 0x566150 `SetAcid`, 0x566060 `SetSmoke`, 0x56A7E0 `SetCracked`, 0x56D5F0 `SetFrozen` e 0x569770
        // `RemoveShield` só repassam para a casa (`BoardSpace::SetAcid`, `SetSmoke(bool, bool)`, `SetCracked`,
        // `SetFrozen`, `RemoveForceField`), ou para a sentinela com casa inválida.
        m.add_method("SetAcid", |_, s, (p, v): (LPoint, bool)| { s.0.borrow_mut().tile_set_acid(to_b(p.0), v); Ok(()) });
        m.add_method("SetSmoke", |_, s, (p, v, _extra): (LPoint, bool, bool)| { s.0.borrow_mut().tile_set_smoke(to_b(p.0), v); Ok(()) });
        m.add_method("SetCracked", |_, s, (p, v): (LPoint, bool)| { s.0.borrow_mut().tile_set_cracked(to_b(p.0), v); Ok(()) });
        m.add_method("SetFrozen", |_, s, (p, v): (LPoint, bool)| { s.0.borrow_mut().tile_set_frozen(to_b(p.0), v); Ok(()) });
        m.add_method("RemoveShield", |_, s, p: LPoint| { s.0.borrow_mut().tile_shield_off(to_b(p.0)); Ok(()) });
        // 0x571630 `(int)` e 0x567000 `(string)`
        m.add_method("GetPawnCount", |_, s, v: Value| {
            let b = s.0.borrow();
            match &v {
                Value::String(n) => Ok(b.get_pawn_count_type(&n.to_string_lossy())),
                _ => Ok(b.get_pawn_count(arg_int(&v).ok_or_else(|| no_overload("GetPawnCount(int), GetPawnCount(string)"))?)),
            }
        });
        m.add_method("RandomizeTerrain", |lua, s, (kind, chance): (Value, Value)| {
            let (kind, chance) = arg_int(&kind).zip(arg_int(&chance)).ok_or_else(|| no_overload("RandomizeTerrain(int,int)"))?;
            let rng = rng_of(lua)?;
            s.0.borrow_mut().randomize_terrain(kind, chance, &mut rng.borrow_mut());
            Ok(())
        });
        m.add_method("SetWeather", |_, s, (kind, sub, from, to, time): (Value, Value, LPoint, LPoint, Value)| {
            let sig = || no_overload("SetWeather(int,int,Point,Point,float)");
            let (kind, sub) = arg_int(&kind).zip(arg_int(&sub)).ok_or_else(sig)?;
            let time = arg_f32(&time).ok_or_else(sig)?;
            s.0.borrow_mut().set_weather(kind, sub, to_b(from.0), to_b(to.0), time);
            Ok(())
        });
        m.add_method("BlockSpawn", |_, s, (p, kind): (LPoint, Value)| {
            let kind = arg_int(&kind).ok_or_else(|| no_overload("BlockSpawn(Point,int)"))?;
            s.0.borrow_mut().set_block_spawn(to_b(p.0), kind);
            Ok(())
        });
        m.add_method("ClearBlockSpawns", |_, s, ()| { s.0.borrow_mut().clear_block_spawns(); Ok(()) });
        m.add_method("GetZone", |_, s, name: Value| {
            let n = crate::records::arg_str(&name).ok_or_else(|| no_overload("GetZone(string)"))?;
            let list = s.0.borrow().get_zone(&n);
            Ok(crate::skill::LPointList(list.into_iter().map(from_b).collect()))
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
        // 0x63B750 `Pawn::IsNonGridStructure`: `NonGrid` da tabela do tipo (lido na criação).
        m.add_method("IsNonGridStructure", |_, p, ()| Ok(with(p, |s| s.non_grid_structure)));
        // 0x63CF50: `LuaData::GetInt("Ranged") == 1`; 0x63A960: `LuaData::GetBool("TempUnit")`.
        // 0x637810 `Pawn::SetFrozen(bool)`: grava +0x8D1; congelar apaga fogo e infecção (o resto é animação e som).
        m.add_method("SetFrozen", |_, p, v: bool| { p.board.borrow_mut().pawn_set_frozen(p.idx, v); Ok(()) });
        m.add_method("IsRanged", |lua, p, ()| Ok(crate::pawns::type_int(lua, &with(p, |s| s.type_name.clone()), "Ranged")? == 1));
        m.add_method("IsTempUnit", |lua, p, ()| crate::pawns::type_bool(lua, &with(p, |s| s.type_name.clone()), "TempUnit"));
        // 0x639C00: `CallMethod(tipo, "GetPositionScore", p)` (0x641980, `LuaEnv::CallMethod<int, Point>`). Erro ou
        // retorno que não é número dão 0.
        m.add_method("GetCustomPositionScore", |lua, p, pt: LPoint| {
            let ty = with(p, |s| s.type_name.clone());
            let f: mlua::Function = lua.globals().get("CallMethod")?;
            Ok(match f.call::<Value>((ty, "GetPositionScore", pt)) {
                Ok(v) => arg_int(&v).unwrap_or(0),
                Err(_) => 0,
            })
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
    "GetPawnTeam", "GetSize", "GetSimpleReachable", "GetSimplePath", "GetReachable", "GetPath", "IsDeadly", "IsTerrain", "IsPod", "IsItem",
    "IsDangerous", "IsSpawning", "IsCrackable", "DamageSpace", "AddEffect", "IsPawnSpace", "IsBlocked", "IsPawnTeam", "GetPawn", "IsPawnAlive", "GetPawnSpace",
    "GetZone", "AddPawn", "SpawnPawn", "SpawnQueued", "GetPawns", "SetTerrain", "BlockSpawn", "ClearBlockSpawns",
    "GetHealth", "RandomizeTerrain", "SetWeather", "IsPowered", "IsFrozen", "IsDangerousItem", "GetEnemyCount", "GetPawnCount",
    "GetDistanceToBuilding", "GetDistanceToPawn", "IsDamaged", "GetMechDamage", "AddUniqueBuilding", "MarkSpaceDesc",
    "IsSafe", "StopWeather", "GetBuildings", "SetCustomTile", "SetAcid", "SetSmoke", "SetCracked", "SetFrozen", "RemoveShield",
];
/// Ligados, mas com parte do estado ainda ausente no Rust (ver `docs/DIVERGENCIAS.md`).
pub const BOARD_PARTIAL: &[&str] = &["IsTargeted", "IsBusy"];
/// Respondidos com dados do estado do jogo (rotina original ainda não reconstruída).
pub const PAWN_HOST: &[&str] = &["GetBaseMove", "IsAbility"];
pub const PAWN_DONE: &[&str] = &[
    "GetHealth", "IsDead", "IsDamaged", "GetTeam", "IsPlayer", "IsEnemy", "GetId", "IsMech", "GetSpace", "SetTeam", "SetMech", "GetPathProf", "IsFire", "IsFrozen", "IsAcid", "IsInfected", "IsPowered", "IsGuarding",
    "IsShield", "IsBoosted", "IsFlying", "GetType", "IsTeleporter", "IsJumper", "IsBurrower", "GetMoveSpeed", "IsCorpse", "IsGrappled",
    "IsNonGridStructure", "GetCustomPositionScore", "IsRanged", "IsTempUnit", "SetFrozen",
];
