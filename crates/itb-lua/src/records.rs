//! Classes "de dados" do luabind (só propriedades + construtores), num tipo genérico.
//!
//! Cada classe declara seus campos visíveis na Lua (os mesmos do manifesto) com tipo e valor
//! padrão, e seus construtores. Valores e mapeamentos vêm da desmontagem dos construtores C++
//! (provas em `re/evidencias/rotinas_valores.md`).

use crate::values::{arg_f32, arg_int, no_overload, LPoint};
use itb_core::geometry::IPoint;
use mlua::{IntoLua, Lua, MetaMethod, MultiValue, UserData, UserDataMethods, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Field {
    Int(i32),
    Float(f32),
    Bool(bool),
    Str(String),
    Point(IPoint),
    /// Vetor de pontos interno (ex.: caminho de movimento de SpaceDamage, +0xCC).
    Points(Vec<IPoint>),
}

impl Field {
    fn to_lua(&self, lua: &Lua) -> mlua::Result<Value> {
        match self {
            Field::Int(i) => i.into_lua(lua),
            Field::Float(f) => f.into_lua(lua),
            Field::Bool(b) => b.into_lua(lua),
            Field::Str(s) => s.as_str().into_lua(lua),
            Field::Point(p) => LPoint(*p).into_lua(lua),
            Field::Points(v) => Ok(Value::Integer(v.len() as _)),
        }
    }
    /// Conversão com as regras do luabind para o tipo atual do campo.
    fn assign(&self, v: &Value) -> Option<Field> {
        Some(match self {
            Field::Int(_) => Field::Int(arg_int(v)?),
            Field::Float(_) => Field::Float(arg_f32(v)?),
            Field::Bool(_) => match v { Value::Boolean(b) => Field::Bool(*b), _ => return None },
            Field::Str(_) => Field::Str(arg_str(v)?),
            Field::Point(_) => Field::Point(arg_point(v)?),
            Field::Points(_) => return None,
        })
    }
}

pub fn arg_str(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.to_string_lossy().to_string()),
        _ => None,
    }
}

pub fn arg_point(v: &Value) -> Option<IPoint> {
    match v {
        Value::UserData(u) => u.borrow::<LPoint>().ok().map(|p| p.0),
        _ => None,
    }
}

/// Instância de uma classe de dados. `hidden` guarda campos internos (não registrados na Lua).
#[derive(Clone, Debug)]
pub struct Record {
    pub class: &'static str,
    pub fields: Vec<(&'static str, Field)>,
    pub hidden: Vec<(&'static str, Field)>,
}

impl Record {
    pub fn get(&self, k: &str) -> Option<&Field> { self.fields.iter().find(|(n, _)| *n == k).map(|(_, f)| f) }
    pub fn hidden(&self, k: &str) -> Option<&Field> { self.hidden.iter().find(|(n, _)| *n == k).map(|(_, f)| f) }
    pub fn set(&mut self, k: &str, f: Field) {
        if let Some(slot) = self.fields.iter_mut().find(|(n, _)| *n == k) { slot.1 = f; return; }
        if let Some(slot) = self.hidden.iter_mut().find(|(n, _)| *n == k) { slot.1 = f; return; }
        panic!("campo inexistente {}.{k}", self.class);
    }
    pub fn int(&self, k: &str) -> i32 {
        match self.get(k).or_else(|| self.hidden(k)) { Some(Field::Int(i)) => *i, _ => panic!("{k} não é int") }
    }
    pub fn points(&self, k: &str) -> &[IPoint] {
        match self.hidden(k) { Some(Field::Points(v)) => v, _ => &[] }
    }
}

/// Leitura de propriedade com fallback para métodos (reais ou stubs) da classe.
pub fn rec_index(lua: &Lua, r: &Record, k: &str) -> mlua::Result<Value> {
    if let Some(f) = r.get(k) {
        return f.to_lua(lua);
    }
    if let Some(t) = lua.named_registry_value::<Option<mlua::Table>>(&format!("itb_real_methods_{}", r.class))? {
        let v: Value = t.get(k)?;
        if !v.is_nil() { return Ok(v); }
    }
    match lua.named_registry_value::<Option<mlua::Table>>(&format!("itb_methods_{}", r.class))? {
        Some(t) => t.get::<Value>(k),
        None => Ok(Value::Nil),
    }
}

/// Escrita de propriedade com as conversões do luabind.
pub fn rec_newindex(r: &mut Record, k: &str, v: &Value) -> mlua::Result<()> {
    let class = r.class;
    match r.fields.iter_mut().find(|(n, _)| *n == k) {
        Some((_, f)) => {
            *f = f.assign(v).ok_or_else(|| no_overload(&format!("{class}.{k} = <tipo do campo>")))?;
            Ok(())
        }
        None => Err(mlua::Error::RuntimeError(format!("propriedade inexistente: {class}.{k}"))),
    }
}

impl UserData for Record {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(MetaMethod::Index, |lua, r, k: String| rec_index(lua, r, &k));
        m.add_meta_method_mut(MetaMethod::NewIndex, |_, r, (k, v): (String, Value)| rec_newindex(r, &k, &v));
    }
}

type Ctor = fn(&[Value]) -> Option<Record>;

/// (classe Lua, campos visíveis por ordem do manifesto, construtor)
pub struct RecordClass {
    pub name: &'static str,
    pub properties: &'static [&'static str],
    pub ctor: Ctor,
    pub signature: &'static str,
}

fn rec(class: &'static str, fields: Vec<(&'static str, Field)>, hidden: Vec<(&'static str, Field)>) -> Record {
    Record { class, fields, hidden }
}

/// `Text` = C++ `TextData` (0x2C bytes). `Text()` (0x0046670E): "", (0,0), 5, 2, 0x7FFFFFFF.
/// `Text(char*, Point, int)` 0x004C4130: align 2, width 0x7FFFFFFF. `(char*, Point, int, int)` 0x004C41D0: align dado.
/// `Text(TextData&)`: cópia.
fn text(a: &[Value]) -> Option<Record> {
    let mk = |t: String, p: IPoint, size: i32, align: i32, width: i32| rec("Text", vec![
        ("text", Field::Str(t)), ("pos", Field::Point(p)), ("size", Field::Int(size)),
        ("align", Field::Int(align)), ("width", Field::Int(width))], vec![]);
    match a {
        [] => Some(mk(String::new(), IPoint::new(0, 0), 5, 2, i32::MAX)),
        [Value::UserData(u)] => u.borrow::<Record>().ok().filter(|r| r.class == "Text").map(|r| r.clone()),
        [t, p, s] => Some(mk(arg_str(t)?, arg_point(p)?, arg_int(s)?, 2, i32::MAX)),
        [t, p, s, al] => Some(mk(arg_str(t)?, arg_point(p)?, arg_int(s)?, arg_int(al)?, i32::MAX)),
        _ => None,
    }
}

/// `StyleInfo` (0x1C bytes, sem propriedades). `()`: ("classic", 12) (0x006D6E6E); `(string, int)` copia.
fn style_info(a: &[Value]) -> Option<Record> {
    let (n, s) = match a {
        [] => ("classic".to_string(), 12),
        [n, s] => (arg_str(n)?, arg_int(s)?),
        _ => return None,
    };
    Some(rec("StyleInfo", vec![], vec![("font", Field::Str(n)), ("size", Field::Int(s))]))
}

/// `JoyRumble` (8 bytes): `()` = (0.0, 0) (0x006D8A40); `(float mag, int length)` (0x006D8AE6).
fn joy_rumble(a: &[Value]) -> Option<Record> {
    let (m, l) = match a {
        [] => (0.0, 0),
        [m, l] => (arg_f32(m)?, arg_int(l)?),
        _ => return None,
    };
    Some(rec("JoyRumble", vec![("mag", Field::Float(m)), ("length", Field::Int(l))], vec![]))
}

/// `PilotSkill` (0x74 bytes): strings +0x00, +0x18 (internas), name +0x30, desc +0x48; ints +0x60..+0x70 = 0.
/// `()` 0x00619D30: tudo vazio. `(string, string)` 0x00675A00: name, desc.
fn pilot_skill(a: &[Value]) -> Option<Record> {
    let (n, d) = match a {
        [] => (String::new(), String::new()),
        [n, d] => (arg_str(n)?, arg_str(d)?),
        _ => return None,
    };
    Some(rec("PilotSkill", vec![("desc", Field::Str(d)), ("name", Field::Str(n))],
             vec![("str_0x00", Field::Str(String::new())), ("str_0x18", Field::Str(String::new()))]))
}

/// `TipData` (0x30 bytes): type +0x00, value +0x18. `()` 0x006757A0 vazio; `(string, string)` 0x00675840.
fn tip_data(a: &[Value]) -> Option<Record> {
    let (t, v) = match a {
        [] => (String::new(), String::new()),
        [t, v] => (arg_str(t)?, arg_str(v)?),
        _ => return None,
    };
    Some(rec("TipData", vec![("value", Field::Str(v)), ("type", Field::Str(t))], vec![]))
}

/// `Objective` = C++ `RepObj`: text +0x00, param1 +0x18, param2 +0x30, category +0x48 (enum
/// RewardCategory), rep +0x4C, potential +0x50, string interna +0x58.
/// Construtores C++: `(s,int)` 0x005B83A0, `(s,int,int)` 0x005B86C0, `(s,s,s,int)` 0x005B84F0,
/// `(s,s,s,int,int)` 0x005B8810 → categoria 0; `(s,s)` 0x005B89E0 e `(s,s,s,s)` 0x005B8B60 →
/// categoria 2, rep = potential = 1 e o último texto vai para +0x58.
fn objective(a: &[Value]) -> Option<Record> {
    let is_s = |v: &Value| matches!(v, Value::String(_));
    let (text, p1, p2, cat, rep, pot, extra) = match a {
        [t, r] if !is_s(r) => (arg_str(t)?, String::new(), String::new(), 0, arg_int(r)?, arg_int(r)?, String::new()),
        [t, e] => (arg_str(t)?, String::new(), String::new(), 2, 1, 1, arg_str(e)?),
        [t, r, p] => (arg_str(t)?, String::new(), String::new(), 0, arg_int(r)?, arg_int(p)?, String::new()),
        [t, a1, a2, r] if !is_s(r) => (arg_str(t)?, arg_str(a1)?, arg_str(a2)?, 0, arg_int(r)?, arg_int(r)?, String::new()),
        [t, a1, a2, e] => (arg_str(t)?, arg_str(a1)?, arg_str(a2)?, 2, 1, 1, arg_str(e)?),
        [t, a1, a2, r, p] => (arg_str(t)?, arg_str(a1)?, arg_str(a2)?, 0, arg_int(r)?, arg_int(p)?, String::new()),
        _ => return None,
    };
    Some(rec("Objective", vec![
        ("rep", Field::Int(rep)), ("potential", Field::Int(pot)), ("text", Field::Str(text)),
        ("param1", Field::Str(p1)), ("param2", Field::Str(p2)), ("category", Field::Int(cat))],
        vec![("str_0x58", Field::Str(extra)), ("failed_0x55", Field::Bool(false))]))
}

/// `RepObj::Failed` (0x5BA850): cópia por `(texto, p1, p2, 0, potential)` (rep 0), com a categoria e +0x58 do
/// original e o byte +0x55 ligado.
pub fn objective_failed(o: &Record) -> Record {
    let mut f = o.clone();
    f.set("rep", Field::Int(0));
    f.set("failed_0x55", Field::Bool(true));
    f
}

/// Métodos reais de `Objective`.
pub fn install_objective_methods(lua: &Lua) -> mlua::Result<()> {
    let t = lua.create_table()?;
    t.set("Failed", lua.create_function(|_, s: Value| {
        let Value::UserData(u) = &s else { return Err(no_overload("Objective:Failed()")) };
        let o = u.borrow::<Record>().ok().filter(|r| r.class == "Objective").ok_or_else(|| no_overload("Objective:Failed()"))?;
        Ok(objective_failed(&o))
    })?)?;
    lua.set_named_registry_value("itb_real_methods_Objective", t)
}

/// `VoiceEvent(string, int, int)` (registro 0x67AB90): `id` +0x00, `pawn1` +0x18, `pawn2` +0x1C. O corpo do
/// construtor não foi localizado: a cópia dos argumentos na ordem é SUPOSIÇÃO, conferida pelo teste diferencial.
fn voice_event(a: &[Value]) -> Option<Record> {
    match a {
        [id, p1, p2] => Some(rec("VoiceEvent", vec![("id", Field::Str(arg_str(id)?)), ("pawn1", Field::Int(arg_int(p1)?)),
                                                   ("pawn2", Field::Int(arg_int(p2)?))], vec![])),
        _ => None,
    }
}

/// `RegionInfo` (0x14 bytes, sem propriedades): `(Point a)` = (a.x, a.y, 0, 0, 1000) (0x006D79A0);
/// `(Point a, Point b, int n)` = (a.x, a.y, b.x, b.y, n) (0x006D78E4).
fn region_info(a: &[Value]) -> Option<Record> {
    let (p, q, n) = match a {
        [p] => (arg_point(p)?, IPoint::new(0, 0), 1000),
        [p, q, n] => (arg_point(p)?, arg_point(q)?, arg_int(n)?),
        _ => return None,
    };
    Some(rec("RegionInfo", vec![], vec![("a", Field::Point(p)), ("b", Field::Point(q)), ("n", Field::Int(n))]))
}

/// `SpaceDamage` (construtores C++ 0x0055A170 `()`, 0x00598AB0 `(int)`, 0x00579D70 `(Point)`,
/// 0x0057E820 `(Point,int)`, 0x005E5EF0 `(Point,int,int)`; os 25 campos a partir de +0x10 são
/// inicializados de forma idêntica nos cinco — verificado por comparação das instruções).
/// Padrões: loc (-1,-1), iDamage 0, iPush 4 (DIR_NONE), iPawnTeam 2 (TEAM_NONE), iTerrain 10,
/// demais 0/false/"". Campos internos relevantes guardados em `hidden`.
pub fn space_damage(a: &[Value]) -> Option<Record> {
    let (loc, dmg, push) = match a {
        [] => (IPoint::new(-1, -1), 0, 4),
        [Value::UserData(_)] => (arg_point(&a[0])?, 0, 4),
        [d] => (IPoint::new(-1, -1), arg_int(d)?, 4),
        [p, d] => (arg_point(p)?, arg_int(d)?, 4),
        [p, d, push] => (arg_point(p)?, arg_int(d)?, arg_int(push)?),
        _ => return None,
    };
    let s = || Field::Str(String::new());
    Some(rec("SpaceDamage", vec![
        ("loc", Field::Point(loc)), ("iDamage", Field::Int(dmg)), ("iPush", Field::Int(push)),
        ("iShield", Field::Int(0)), ("bSimpleMark", Field::Bool(false)), ("iFire", Field::Int(0)),
        ("iFrozen", Field::Int(0)), ("iInjure", Field::Int(0)), ("iSmoke", Field::Int(0)),
        ("iAcid", Field::Int(0)), ("iCrack", Field::Int(0)), ("bKO_Effect", Field::Bool(false)),
        ("sAnimation", s()), ("sSound", s()), ("sImageMark", s()), ("sPawn", s()),
        ("iPawnTeam", Field::Int(2)), ("bHide", Field::Bool(false)), ("bHidePath", Field::Bool(false)),
        ("bHideIcon", Field::Bool(false)), ("bEvacuate", Field::Bool(false)), ("fDelay", Field::Float(0.0)),
        ("iTerrain", Field::Int(10)), ("sScript", s()), ("sItem", s())],
        vec![("anim_param_0x34", Field::Int(2)), ("image_0x68", s()), ("kind_0x98", Field::Int(0)),
             ("source_0x9c", Field::Point(IPoint::new(-1, -1))), ("i_0xc0", Field::Int(2)),
             ("path_0xcc", Field::Points(vec![])), ("move_type_0xd8", Field::Int(0)), ("str_0xf8", s()),
             ("p_0x110", Field::Point(IPoint::new(-1, -1))), ("i_0x130", Field::Int(3))]))
}

pub static RECORD_CLASSES: &[RecordClass] = &[
    RecordClass { name: "SpaceDamage", properties: &["loc", "iDamage", "iPush", "iShield", "bSimpleMark", "iFire", "iFrozen",
                  "iInjure", "iSmoke", "iAcid", "iCrack", "bKO_Effect", "sAnimation", "sSound", "sImageMark", "sPawn", "iPawnTeam",
                  "bHide", "bHidePath", "bHideIcon", "bEvacuate", "fDelay", "iTerrain", "sScript", "sItem"], ctor: space_damage,
                  signature: "SpaceDamage(), SpaceDamage(int), SpaceDamage(Point), SpaceDamage(Point,int), SpaceDamage(Point,int,int)" },
    RecordClass { name: "Objective", properties: &["rep", "potential", "text", "param1", "param2", "category"], ctor: objective,
                  signature: "Objective(string,int), Objective(string,string), Objective(string,int,int), Objective(string,string,string,int), Objective(string,string,string,string), Objective(string,string,string,int,int)" },
    RecordClass { name: "RegionInfo", properties: &[], ctor: region_info, signature: "RegionInfo(Point), RegionInfo(Point,Point,int)" },
    RecordClass { name: "VoiceEvent", properties: &["id", "pawn1", "pawn2"], ctor: voice_event, signature: "VoiceEvent(string,int,int)" },
    RecordClass { name: "Text", properties: &["text", "pos", "size", "align", "width"], ctor: text,
                  signature: "Text(), Text(string,Point,int), Text(string,Point,int,int), Text(Text)" },
    RecordClass { name: "StyleInfo", properties: &[], ctor: style_info, signature: "StyleInfo(), StyleInfo(string,int)" },
    RecordClass { name: "JoyRumble", properties: &["mag", "length"], ctor: joy_rumble, signature: "JoyRumble(), JoyRumble(float,int)" },
    RecordClass { name: "PilotSkill", properties: &["desc", "name"], ctor: pilot_skill, signature: "PilotSkill(), PilotSkill(string,string)" },
    RecordClass { name: "TipData", properties: &["value", "type"], ctor: tip_data, signature: "TipData(), TipData(string,string)" },
];

pub fn construct(class: &RecordClass, args: &MultiValue) -> mlua::Result<Record> {
    let a: Vec<Value> = args.iter().cloned().collect();
    (class.ctor)(&a).ok_or_else(|| no_overload(class.signature))
}
