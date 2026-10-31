//! `SkillEffect`, `DamageList`, `PointList`, `IntList`, `UIntList` e referências a `SpaceDamage`.
//!
//! O luabind devolve `SkillEffect.effect`, `DamageList:index()` e `:back()` POR REFERÊNCIA
//! (scripts fazem `ret.effect:back().bHide = true`). Por isso as listas de dano são
//! `Rc<RefCell<Vec<Record>>>` e `LDamageRef` aponta para um item dentro delas.
//! Provas: `re/evidencias/rotinas_skilleffect.md`.

use crate::records::{arg_point, rec_index, rec_newindex, space_damage, Field, Record};
use crate::values::{arg_f32, arg_int, no_overload, LPoint};
use itb_core::geometry::IPoint;
use mlua::{FromLua, Lua, MetaMethod, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub type DamageVec = Rc<RefCell<Vec<Record>>>;

fn bad_index() -> mlua::Error {
    mlua::Error::RuntimeError("índice fora da lista (no original: leitura de memória inválida)".into())
}

/// Referência a um `SpaceDamage` dentro de uma lista.
#[derive(Clone)]
pub struct LDamageRef {
    pub vec: DamageVec,
    pub idx: usize,
}

impl UserData for LDamageRef {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(MetaMethod::Index, |lua, r, k: String| {
            let v = r.vec.borrow();
            let rec = v.get(r.idx).ok_or_else(bad_index)?;
            rec_index(lua, rec, &k)
        });
        m.add_meta_method(MetaMethod::NewIndex, |_, r, (k, val): (String, Value)| {
            let mut v = r.vec.borrow_mut();
            let rec = v.get_mut(r.idx).ok_or_else(bad_index)?;
            rec_newindex(rec, &k, &val)
        });
    }
}

/// Aceita `SpaceDamage` por valor ou referência e devolve uma cópia (passagem por valor do C++).
pub fn damage_arg(v: &Value) -> Option<Record> {
    let Value::UserData(u) = v else { return None };
    if let Ok(r) = u.borrow::<Record>() {
        return (r.class == "SpaceDamage").then(|| r.clone());
    }
    if let Ok(r) = u.borrow::<LDamageRef>() {
        return r.vec.borrow().get(r.idx).cloned();
    }
    None
}

/// Métodos reais de `SpaceDamage` (0x00655F10, 0x00655F40, 0x00655F80).
pub fn install_space_damage_methods(lua: &Lua) -> mlua::Result<()> {
    let t = lua.create_table()?;
    t.set("IsMovement", lua.create_function(|_, s: Value| {
        let d = damage_arg(&s).ok_or_else(|| no_overload("SpaceDamage:IsMovement()"))?;
        Ok(!d.points("path_0xcc").is_empty() && d.int("move_type_0xd8") != 3 && d.int("kind_0x98") == 0)
    })?)?;
    t.set("MoveStart", lua.create_function(|_, s: Value| {
        let d = damage_arg(&s).ok_or_else(|| no_overload("SpaceDamage:MoveStart()"))?;
        // caminho vazio → (-1, 1) no original (0x00655F64)
        Ok(LPoint(d.points("path_0xcc").first().copied().unwrap_or(IPoint::new(-1, 1))))
    })?)?;
    t.set("MoveEnd", lua.create_function(|_, s: Value| {
        let d = damage_arg(&s).ok_or_else(|| no_overload("SpaceDamage:MoveEnd()"))?;
        // caminho vazio: o original lê memória antes do início do vetor (indefinido)
        d.points("path_0xcc").last().copied().map(LPoint).ok_or_else(bad_index)
    })?)?;
    lua.set_named_registry_value("itb_real_methods_SpaceDamage", t)
}

// ------------------------------------------------------------------ DamageList

#[derive(Clone)]
pub struct LDamageList(pub DamageVec);

impl LDamageList {
    pub fn new() -> Self { Self(Rc::new(RefCell::new(Vec::new()))) }
}

impl Default for LDamageList {
    fn default() -> Self { Self::new() }
}

impl FromLua for LDamageList {
    fn from_lua(v: Value, _: &Lua) -> mlua::Result<Self> {
        match v {
            Value::UserData(u) => Ok(u.borrow::<LDamageList>()?.clone()),
            o => Err(mlua::Error::FromLuaConversionError { from: o.type_name(), to: "DamageList".into(), message: None }),
        }
    }
}

impl UserData for LDamageList {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        // 0x006528E0 size, 0x00652900 index (1-based, sem checagem), 0x0066E4E0 push_back,
        // 0x00675790 back, 0x00652920 empty
        m.add_method("size", |_, l, ()| Ok(l.0.borrow().len() as i32));
        m.add_method("index", |_, l, i: Value| {
            let i = arg_int(&i).ok_or_else(|| no_overload("DamageList:index(int)"))?;
            if i < 1 || i as usize > l.0.borrow().len() { return Err(bad_index()); }
            Ok(LDamageRef { vec: l.0.clone(), idx: i as usize - 1 })
        });
        m.add_method("push_back", |_, l, d: Value| {
            let d = damage_arg(&d).ok_or_else(|| no_overload("DamageList:push_back(SpaceDamage)"))?;
            l.0.borrow_mut().push(d);
            Ok(())
        });
        m.add_method("back", |_, l, ()| {
            let n = l.0.borrow().len();
            if n == 0 { return Err(bad_index()); }
            Ok(LDamageRef { vec: l.0.clone(), idx: n - 1 })
        });
        m.add_method("empty", |_, l, ()| Ok(l.0.borrow().is_empty()));
    }
}

// ------------------------------------------------------------------ listas simples

macro_rules! simple_list {
    ($name:ident, $elem:ty, $lua:literal, $conv:expr, $out:expr, erase = $erase:literal, empty = $empty:literal) => {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct $name(pub Vec<$elem>);

        impl FromLua for $name {
            fn from_lua(v: Value, _: &Lua) -> mlua::Result<Self> {
                match v {
                    Value::UserData(u) => Ok(u.borrow::<$name>()?.clone()),
                    o => Err(mlua::Error::FromLuaConversionError { from: o.type_name(), to: $lua.into(), message: None }),
                }
            }
        }

        impl UserData for $name {
            fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
                m.add_method("size", |_, l, ()| Ok(l.0.len() as i32));
                m.add_method("index", |_, l, i: Value| {
                    let i = arg_int(&i).ok_or_else(|| no_overload(concat!($lua, ":index(int)")))?;
                    if i < 1 || i as usize > l.0.len() { return Err(bad_index()); }
                    Ok($out(l.0[i as usize - 1]))
                });
                m.add_method_mut("push_back", |_, l, v: Value| {
                    let conv: fn(&Value) -> Option<$elem> = $conv;
                    l.0.push(conv(&v).ok_or_else(|| no_overload(concat!($lua, ":push_back")))?);
                    Ok(())
                });
                m.add_method("back", |_, l, ()| l.0.last().copied().map($out).ok_or_else(bad_index));
                if $erase {
                    m.add_method_mut("erase", |_, l, i: Value| {
                        let i = arg_int(&i).ok_or_else(|| no_overload(concat!($lua, ":erase(int)")))?;
                        if i < 1 || i as usize > l.0.len() { return Err(bad_index()); }
                        l.0.remove(i as usize - 1);
                        Ok(())
                    });
                }
                if $empty {
                    m.add_method("empty", |_, l, ()| Ok(l.0.is_empty()));
                }
            }
        }
    };
}

simple_list!(LPointList, IPoint, "PointList", arg_point, LPoint, erase = true, empty = true);
simple_list!(LIntList, i32, "IntList", arg_int, |v: i32| v, erase = true, empty = true);
simple_list!(LUIntList, u32, "UIntList", |v: &Value| arg_int(v).map(|i| i as u32), |v: u32| v, erase = false, empty = false);

// ------------------------------------------------------------------ SkillEffect

/// `SkillEffect` (construtor C++ 0x0055D8F0): listas vazias, impact_sound "",
/// piOrigin = Point() (0x80000001, 0x80000001), iOwner = -1.
pub struct LSkillEffect {
    pub effect: DamageVec,
    pub q_effect: DamageVec,
    pub impact_sound: String,
    pub pi_origin: IPoint,
    pub i_owner: i32,
}

impl Default for LSkillEffect {
    fn default() -> Self {
        Self { effect: LDamageList::new().0, q_effect: LDamageList::new().0, impact_sound: String::new(),
               pi_origin: IPoint::lua_default(), i_owner: -1 }
    }
}

fn sd_zero() -> Record { space_damage(&[Value::Integer(0)]).expect("SpaceDamage(0)") }

fn sd_at(p: IPoint) -> Record {
    let mut d = sd_zero();
    d.set("loc", Field::Point(p));
    d
}

/// Projétil (0x00653370 / fila 0x00653740): imagem em +0x68; tipo 5 se a imagem contém "laser",
/// senão 2; atraso 0 para laser (só na versão imediata); origem em +0x9C (só na versão imediata).
fn projectile(mut d: Record, image: String, delay: f32, origin: Option<IPoint>) -> Record {
    let kind = if image.contains("laser") { 5 } else { 2 };
    d.set("kind_0x98", Field::Int(kind));
    let delay = if origin.is_some() && kind == 5 { 0.0 } else { delay };
    d.set("fDelay", Field::Float(delay));
    d.set("image_0x68", Field::Str(image));
    if let Some(o) = origin { d.set("source_0x9c", Field::Point(o)); }
    d
}

/// Artilharia (0x00653640 / fila 0x00653920): imagem, tipo 1, atraso, origem (imediata).
fn artillery(mut d: Record, image: String, delay: f32, origin: Option<IPoint>) -> Record {
    d.set("kind_0x98", Field::Int(1));
    d.set("fDelay", Field::Float(delay));
    d.set("image_0x68", Field::Str(image));
    if let Some(o) = origin { d.set("source_0x9c", Field::Point(o)); }
    d
}

/// Corpo a corpo (0x00654B50 / fila 0x00654C70): tipo de movimento 3, origem anexada ao caminho, atraso.
fn melee(mut d: Record, origin: IPoint, delay: f32) -> Record {
    d.set("move_type_0xd8", Field::Int(3));
    let mut path = d.points("path_0xcc").to_vec();
    path.push(origin);
    d.set("path_0xcc", Field::Points(path));
    d.set("fDelay", Field::Float(delay));
    d
}

fn str_arg(v: &Value) -> Option<String> { crate::records::arg_str(v) }

impl UserData for LSkillEffect {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        use mlua::MultiValue;
        m.add_meta_method(MetaMethod::Index, |lua, s, k: String| {
            Ok(match k.as_str() {
                "effect" => mlua::IntoLua::into_lua(LDamageList(s.effect.clone()), lua)?,
                "q_effect" => mlua::IntoLua::into_lua(LDamageList(s.q_effect.clone()), lua)?,
                "impact_sound" => mlua::IntoLua::into_lua(s.impact_sound.as_str(), lua)?,
                "piOrigin" => mlua::IntoLua::into_lua(LPoint(s.pi_origin), lua)?,
                "iOwner" => Value::Integer(s.i_owner as _),
                _ => match lua.named_registry_value::<Option<mlua::Table>>("itb_methods_SkillEffect")? {
                    Some(t) => t.get(k)?,
                    None => Value::Nil,
                },
            })
        });
        m.add_meta_method_mut(MetaMethod::NewIndex, |lua, s, (k, v): (String, Value)| {
            match k.as_str() {
                // def_readwrite com tipo de classe: a atribuição copia o conteúdo
                "effect" => { let l = LDamageList::from_lua(v, lua)?; let c = l.0.borrow().clone(); *s.effect.borrow_mut() = c; }
                "q_effect" => { let l = LDamageList::from_lua(v, lua)?; let c = l.0.borrow().clone(); *s.q_effect.borrow_mut() = c; }
                "impact_sound" => s.impact_sound = str_arg(&v).ok_or_else(|| no_overload("impact_sound=string"))?,
                "piOrigin" => s.pi_origin = arg_point(&v).ok_or_else(|| no_overload("piOrigin=Point"))?,
                "iOwner" => s.i_owner = arg_int(&v).ok_or_else(|| no_overload("iOwner=int"))?,
                _ => return Err(mlua::Error::RuntimeError(format!("propriedade inexistente: SkillEffect.{k}"))),
            }
            Ok(())
        });

        let dmg = |v: &Value, sig: &str| damage_arg(v).ok_or_else(|| no_overload(sig));
        // 0x00652A50 / 0x00652CB0
        m.add_method("AddDamage", move |_, s, d: Value| { s.effect.borrow_mut().push(dmg(&d, "AddDamage(SpaceDamage)")?); Ok(()) });
        m.add_method("AddQueuedDamage", move |_, s, d: Value| { s.q_effect.borrow_mut().push(dmg(&d, "AddQueuedDamage(SpaceDamage)")?); Ok(()) });
        // 0x00653120: SpaceDamage(0) com fDelay e bHide
        m.add_method("AddDelay", |_, s, f: Value| {
            let mut d = sd_zero();
            d.set("fDelay", Field::Float(arg_f32(&f).ok_or_else(|| no_overload("AddDelay(float)"))?));
            d.set("bHide", Field::Bool(true));
            s.effect.borrow_mut().push(d);
            Ok(())
        });
        // 0x00654220 / 0x00654580 / 0x00654670
        m.add_method("AddScript", |_, s, t: Value| {
            let mut d = sd_zero(); d.set("sScript", Field::Str(str_arg(&t).ok_or_else(|| no_overload("AddScript(string)"))?));
            s.effect.borrow_mut().push(d); Ok(())
        });
        m.add_method("AddQueuedScript", |_, s, t: Value| {
            let mut d = sd_zero(); d.set("sScript", Field::Str(str_arg(&t).ok_or_else(|| no_overload("AddQueuedScript(string)"))?));
            s.q_effect.borrow_mut().push(d); Ok(())
        });
        m.add_method("AddSound", |_, s, t: Value| {
            let mut d = sd_zero(); d.set("sSound", Field::Str(str_arg(&t).ok_or_else(|| no_overload("AddSound(string)"))?));
            s.effect.borrow_mut().push(d); Ok(())
        });
        // 0x00654310 / 0x00654400: SpaceDamage(p, 0) com sAnimation (+ parâmetro em +0x34)
        m.add_method("AddAnimation", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let err = || no_overload("AddAnimation(Point,string), AddAnimation(Point,string,int)");
            let mut d = sd_at(arg_point(a.first().ok_or_else(err)?).ok_or_else(err)?);
            d.set("sAnimation", Field::Str(str_arg(a.get(1).ok_or_else(err)?).ok_or_else(err)?));
            match a.len() {
                2 => {}
                3 => d.set("anim_param_0x34", Field::Int(arg_int(&a[2]).ok_or_else(err)?)),
                _ => return Err(err()),
            }
            s.effect.borrow_mut().push(d);
            Ok(())
        });
        // Projéteis: (sd, img) atraso -2 (PROJ_DELAY); (sd, img, f); (origem, sd, img, f). Fila: sem origem.
        m.add_method("AddProjectile", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let err = || no_overload("AddProjectile(SpaceDamage,string[,float]), AddProjectile(Point,SpaceDamage,string,float)");
            let d = match a.as_slice() {
                [d, i] => projectile(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, -2.0, Some(IPoint::new(-1, -1))),
                [o, d, i, f] if arg_point(o).is_some() =>
                    projectile(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, arg_f32(f).ok_or_else(err)?, arg_point(o)),
                [d, i, f] => projectile(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, arg_f32(f).ok_or_else(err)?, Some(IPoint::new(-1, -1))),
                _ => return Err(err()),
            };
            s.effect.borrow_mut().push(d);
            Ok(())
        });
        m.add_method("AddQueuedProjectile", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let err = || no_overload("AddQueuedProjectile(SpaceDamage,string[,float])");
            let (d, i, f) = match a.as_slice() { [d, i] => (d, i, -2.0), [d, i, f] => (d, i, arg_f32(f).ok_or_else(err)?), _ => return Err(err()) };
            let d = projectile(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, f, None);
            s.q_effect.borrow_mut().push(d);
            Ok(())
        });
        m.add_method("AddArtillery", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let err = || no_overload("AddArtillery(SpaceDamage,string[,float]), AddArtillery(Point,SpaceDamage,string,float)");
            let d = match a.as_slice() {
                [d, i] => artillery(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, -2.0, Some(IPoint::new(-1, -1))),
                [o, d, i, f] if arg_point(o).is_some() =>
                    artillery(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, arg_f32(f).ok_or_else(err)?, arg_point(o)),
                [d, i, f] => artillery(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, arg_f32(f).ok_or_else(err)?, Some(IPoint::new(-1, -1))),
                _ => return Err(err()),
            };
            s.effect.borrow_mut().push(d);
            Ok(())
        });
        m.add_method("AddQueuedArtillery", |_, s, a: MultiValue| {
            let a: Vec<Value> = a.into_iter().collect();
            let err = || no_overload("AddQueuedArtillery(SpaceDamage,string[,float])");
            let (d, i, f) = match a.as_slice() { [d, i] => (d, i, -2.0), [d, i, f] => (d, i, arg_f32(f).ok_or_else(err)?), _ => return Err(err()) };
            let d = artillery(damage_arg(d).ok_or_else(err)?, str_arg(i).ok_or_else(err)?, f, None);
            s.q_effect.borrow_mut().push(d);
            Ok(())
        });
        // Corpo a corpo: atraso padrão -1 (FULL_DELAY)
        for (name, queued) in [("AddMelee", false), ("AddQueuedMelee", true)] {
            m.add_method(name, move |_, s, a: MultiValue| {
                let a: Vec<Value> = a.into_iter().collect();
                let err = || no_overload("AddMelee(Point,SpaceDamage[,float])");
                let (o, d, f) = match a.as_slice() {
                    [o, d] => (o, d, -1.0),
                    [o, d, f] => (o, d, arg_f32(f).ok_or_else(err)?),
                    _ => return Err(err()),
                };
                let d = melee(damage_arg(d).ok_or_else(err)?, arg_point(o).ok_or_else(err)?, f);
                if queued { s.q_effect.borrow_mut().push(d) } else { s.effect.borrow_mut().push(d) }
                Ok(())
            });
        }
        // 0x00653AE0 / 0x006549F0: caminho com <= 1 ponto → false sem efeito
        for (name, queued) in [("AddMove", false), ("AddQueuedMove", true)] {
            m.add_method(name, move |_, s, (path, f): (LPointList, Value)| {
                if path.0.len() <= 1 { return Ok(false); }
                let mut d = sd_zero();
                d.set("path_0xcc", Field::Points(path.0));
                d.set("fDelay", Field::Float(arg_f32(&f).ok_or_else(|| no_overload("AddMove(PointList,float)"))?));
                if queued { s.q_effect.borrow_mut().push(d) } else { s.effect.borrow_mut().push(d) }
                Ok(true)
            });
        }
        m.add_method("GetDamageCount", |_, s, ()| Ok(s.effect.borrow().len() as i32));
        m.add_method("GetQueuedCount", |_, s, ()| Ok(s.q_effect.borrow().len() as i32));
        m.add_method("GetDamage", |_, s, i: Value| {
            let i = arg_int(&i).ok_or_else(|| no_overload("GetDamage(int)"))?;
            if i < 1 || i as usize > s.effect.borrow().len() { return Err(bad_index()); }
            Ok(LDamageRef { vec: s.effect.clone(), idx: i as usize - 1 })
        });
        m.add_method("GetQueuedDamage", |_, s, i: Value| {
            let i = arg_int(&i).ok_or_else(|| no_overload("GetQueuedDamage(int)"))?;
            if i < 1 || i as usize > s.q_effect.borrow().len() { return Err(bad_index()); }
            Ok(LDamageRef { vec: s.q_effect.clone(), idx: i as usize - 1 })
        });
    }
}

/// Nomes de `SkillEffect` reconstruídos nesta etapa (para o relatório de cobertura).
pub const SKILL_EFFECT_DONE: &[&str] = &[
    "AddDamage", "AddQueuedDamage", "AddDelay", "AddScript", "AddQueuedScript", "AddSound", "AddAnimation",
    "AddProjectile", "AddQueuedProjectile", "AddArtillery", "AddQueuedArtillery", "AddMelee", "AddQueuedMelee",
    "AddMove", "AddQueuedMove", "GetDamageCount", "GetQueuedCount", "GetDamage", "GetQueuedDamage",
];
