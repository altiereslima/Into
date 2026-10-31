//! `Point` e `GL_Color` expostos à Lua como userdata, com o comportamento do luabind original.

use itb_core::geometry::{lua_number_to_int, Color, IPoint};
use mlua::{FromLua, Lua, MetaMethod, MultiValue, UserData, UserDataFields, UserDataMethods, Value};

/// Mensagem que o luabind usa quando nenhuma sobrecarga casa.
pub fn no_overload(what: &str) -> mlua::Error {
    mlua::Error::RuntimeError(format!("No matching overload found, candidates: {what}"))
}

/// Converte argumento Lua em `int` como `lua_tointeger` do lua5.1.dll original (arredonda ao par).
pub fn arg_int(v: &Value) -> Option<i32> {
    match v {
        Value::Integer(i) => Some(*i as i32),
        Value::Number(n) => Some(lua_number_to_int(*n)),
        _ => None,
    }
}

pub fn arg_f32(v: &Value) -> Option<f32> {
    match v {
        Value::Integer(i) => Some(*i as f32),
        Value::Number(n) => Some(*n as f32),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LPoint(pub IPoint);

impl FromLua for LPoint {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(*ud.borrow::<LPoint>()?),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(), to: "Point".into(), message: None,
            }),
        }
    }
}

impl UserData for LPoint {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("x", |_, p| Ok(p.0.x));
        f.add_field_method_get("y", |_, p| Ok(p.0.y));
        f.add_field_method_set("x", |_, p, v: Value| { p.0.x = arg_int(&v).ok_or_else(|| no_overload("x=int"))?; Ok(()) });
        f.add_field_method_set("y", |_, p, v: Value| { p.0.y = arg_int(&v).ok_or_else(|| no_overload("y=int"))?; Ok(()) });
    }
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(MetaMethod::Add, |_, p, o: LPoint| Ok(LPoint(p.0 + o.0)));
        m.add_meta_method(MetaMethod::Sub, |_, p, o: LPoint| Ok(LPoint(p.0 - o.0)));
        m.add_meta_method(MetaMethod::Eq, |_, p, o: LPoint| Ok(p.0 == o.0));
        m.add_meta_method(MetaMethod::Mul, |_, p, k: Value| {
            let k = arg_int(&k).ok_or_else(|| no_overload("__mul(Point&, int)"))?;
            Ok(LPoint(p.0 * k))
        });
        m.add_meta_method(MetaMethod::ToString, |_, p, ()| Ok(p.0.lua_string()));
        m.add_method("Manhattan", |_, p, o: LPoint| Ok(p.0.manhattan(o.0)));
        m.add_method("GetString", |_, p, ()| Ok(p.0.lua_string()));
        m.add_method("GetLuaString", |_, p, ()| Ok(p.0.lua_string()));
    }
}

/// `Point()`, `Point(x, y)`, `Point(Point)` (três construtores registrados).
pub fn construct_point(args: &MultiValue) -> mlua::Result<LPoint> {
    let a: Vec<&Value> = args.iter().collect();
    match a.as_slice() {
        [] => Ok(LPoint(IPoint::lua_default())),
        [Value::UserData(ud)] => Ok(*ud.borrow::<LPoint>().map_err(|_| no_overload("Point(Point)"))?),
        [x, y] => match (arg_int(x), arg_int(y)) {
            (Some(x), Some(y)) => Ok(LPoint(IPoint::new(x, y))),
            _ => Err(no_overload("Point(), Point(int,int), Point(Point)")),
        },
        _ => Err(no_overload("Point(), Point(int,int), Point(Point)")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LColor(pub Color);

impl FromLua for LColor {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(*ud.borrow::<LColor>()?),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(), to: "GL_Color".into(), message: None,
            }),
        }
    }
}

impl UserData for LColor {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("r", |_, c| Ok(c.0.r));
        f.add_field_method_get("g", |_, c| Ok(c.0.g));
        f.add_field_method_get("b", |_, c| Ok(c.0.b));
        f.add_field_method_get("a", |_, c| Ok(c.0.a));
        f.add_field_method_set("r", |_, c, v: f32| { c.0.r = v; Ok(()) });
        f.add_field_method_set("g", |_, c, v: f32| { c.0.g = v; Ok(()) });
        f.add_field_method_set("b", |_, c, v: f32| { c.0.b = v; Ok(()) });
        f.add_field_method_set("a", |_, c, v: f32| { c.0.a = v; Ok(()) });
    }
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("Normalize", |_, c, ()| Ok(LColor(c.0.normalize())));
        m.add_method("WithAlpha", |_, c, a: f32| Ok(LColor(c.0.with_alpha(a))));
    }
}

/// `GL_Color()`, `GL_Color(r,g,b)`, `GL_Color(r,g,b,a)`.
pub fn construct_color(args: &MultiValue) -> mlua::Result<LColor> {
    let v: Option<Vec<f32>> = args.iter().map(arg_f32).collect();
    match v.as_deref() {
        Some([]) => Ok(LColor(Color::lua_default())),
        Some([r, g, b]) => Ok(LColor(Color::rgb(*r, *g, *b))),
        Some([r, g, b, a]) => Ok(LColor(Color::rgba(*r, *g, *b, *a))),
        _ => Err(no_overload("GL_Color(), GL_Color(float,float,float), GL_Color(float,float,float,float)")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LRect(pub itb_core::geometry::Rect);

impl FromLua for LRect {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(*ud.borrow::<LRect>()?),
            other => Err(mlua::Error::FromLuaConversionError { from: other.type_name(), to: "Rect2D".into(), message: None }),
        }
    }
}

impl UserData for LRect {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("x", |_, r| Ok(r.0.x));
        f.add_field_method_get("y", |_, r| Ok(r.0.y));
        f.add_field_method_get("w", |_, r| Ok(r.0.w));
        f.add_field_method_get("h", |_, r| Ok(r.0.h));
        f.add_field_method_set("x", |_, r, v: Value| { r.0.x = arg_int(&v).ok_or_else(|| no_overload("x=int"))?; Ok(()) });
        f.add_field_method_set("y", |_, r, v: Value| { r.0.y = arg_int(&v).ok_or_else(|| no_overload("y=int"))?; Ok(()) });
        f.add_field_method_set("w", |_, r, v: Value| { r.0.w = arg_int(&v).ok_or_else(|| no_overload("w=int"))?; Ok(()) });
        f.add_field_method_set("h", |_, r, v: Value| { r.0.h = arg_int(&v).ok_or_else(|| no_overload("h=int"))?; Ok(()) });
    }
}

/// `Rect2D()`, `(int,int,int,int)`, `(Point,Point)`, `(Point,int,int)`, `(Rect2D)`.
pub fn construct_rect(args: &MultiValue) -> mlua::Result<LRect> {
    use itb_core::geometry::Rect;
    let a: Vec<&Value> = args.iter().collect();
    let pt = |v: &Value| match v { Value::UserData(u) => u.borrow::<LPoint>().ok().map(|p| p.0), _ => None };
    let err = || no_overload("Rect2D(), Rect2D(int,int,int,int), Rect2D(Point,Point), Rect2D(Point,int,int), Rect2D(Rect2D)");
    Ok(LRect(match a.as_slice() {
        [] => Rect::lua_default(),
        [Value::UserData(u)] => u.borrow::<LRect>().map_err(|_| err())?.0,
        [p, s] => match (pt(p), pt(s)) { (Some(p), Some(s)) => Rect::new(p.x, p.y, s.x, s.y), _ => return Err(err()) },
        [p, w, h] => match (pt(p), arg_int(w), arg_int(h)) { (Some(p), Some(w), Some(h)) => Rect::new(p.x, p.y, w, h), _ => return Err(err()) },
        [x, y, w, h] => match (arg_int(x), arg_int(y), arg_int(w), arg_int(h)) {
            (Some(x), Some(y), Some(w), Some(h)) => Rect::new(x, y, w, h), _ => return Err(err()) },
        _ => return Err(err()),
    }))
}

fn arg_str(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.to_string_lossy().to_string()),
        Value::Integer(_) | Value::Number(_) => None,
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct LButton(pub itb_core::ui::ButtonDesc);

impl UserData for LButton {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("image", |_, b| Ok(b.0.image.clone()));
        f.add_field_method_get("label", |_, b| Ok(b.0.label.clone()));
        // NOTA: luabind devolve referência para membros de classe; aqui é cópia (ver docs/DIVERGENCIAS.md).
        f.add_field_method_get("pos", |_, b| Ok(LPoint(b.0.pos)));
        f.add_field_method_get("hitstats", |_, b| Ok(LRect(b.0.hitstats)));
        f.add_field_method_set("image", |_, b, v: String| { b.0.image = v; Ok(()) });
        f.add_field_method_set("label", |_, b, v: String| { b.0.label = v; Ok(()) });
        f.add_field_method_set("pos", |_, b, v: LPoint| { b.0.pos = v.0; Ok(()) });
        f.add_field_method_set("hitstats", |_, b, v: LRect| { b.0.hitstats = v.0; Ok(()) });
    }
}

/// `Button()`, `(string, Point)`, `(string, Point, bool)`, `(Rect2D, string)`, `(Rect2D, string, int)`.
pub fn construct_button(args: &MultiValue) -> mlua::Result<LButton> {
    use itb_core::ui::ButtonDesc;
    let a: Vec<&Value> = args.iter().collect();
    let pt = |v: &Value| match v { Value::UserData(u) => u.borrow::<LPoint>().ok().map(|p| p.0), _ => None };
    let rc = |v: &Value| match v { Value::UserData(u) => u.borrow::<LRect>().ok().map(|p| p.0), _ => None };
    let err = || no_overload("Button(), Button(string,Point), Button(string,Point,bool), Button(Rect2D,string), Button(Rect2D,string,int)");
    Ok(LButton(match a.as_slice() {
        [] => ButtonDesc::default(),
        [x, y] => match (arg_str(x), pt(y), rc(x), arg_str(y)) {
            (Some(img), Some(p), _, _) => ButtonDesc::with_image(&img, p, false),
            (_, _, Some(r), Some(l)) => ButtonDesc::with_rect(r, &l, None),
            _ => return Err(err()),
        },
        [x, y, z] => match (arg_str(x), pt(y), z, rc(x), arg_str(y), arg_int(z)) {
            (Some(img), Some(p), Value::Boolean(b), _, _, _) => ButtonDesc::with_image(&img, p, *b),
            (_, _, _, Some(r), Some(l), Some(n)) => ButtonDesc::with_rect(r, &l, Some(n)),
            _ => return Err(err()),
        },
        _ => return Err(err()),
    }))
}
