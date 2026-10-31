//! Tipos de valor `Point` e `GL_Color` com o comportamento exato do motor original.
//! Provas: `re/evidencias/rotinas_valores.md`.

/// `Point` (classe `Point` na Lua): dois `int` de 32 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IPoint {
    pub x: i32,
    pub y: i32,
}

/// Valor de `Point()` sem argumentos: 0x80000001 nos dois campos (construtor em 0x00465B00).
pub const POINT_DEFAULT: i32 = -0x7FFF_FFFF;

/// Resultado de `cvttss2si`/`fistp` fora do intervalo de `i32` ("integer indefinite").
const INT_INDEFINITE: i32 = i32::MIN;

impl IPoint {
    pub const fn new(x: i32, y: i32) -> Self { Self { x, y } }
    /// `Point()`.
    pub const fn lua_default() -> Self { Self { x: POINT_DEFAULT, y: POINT_DEFAULT } }

    /// `Manhattan(p)` 0x004BC350: `|x-px| + |y-py|` com aritmética de 32 bits.
    pub fn manhattan(self, o: Self) -> i32 {
        self.x.wrapping_sub(o.x).wrapping_abs().wrapping_add(self.y.wrapping_sub(o.y).wrapping_abs())
    }
    /// `GetString()` 0x004BC380 e `GetLuaString()` 0x004BC510: `"Point( x, y )"`.
    pub fn lua_string(self) -> String { format!("Point( {}, {} )", self.x, self.y) }
}

/// `__add` 0x004520A0 (soma com estouro de 32 bits).
impl std::ops::Add for IPoint {
    type Output = Self;
    fn add(self, o: Self) -> Self { Self::new(self.x.wrapping_add(o.x), self.y.wrapping_add(o.y)) }
}

/// `__sub` 0x00452020.
impl std::ops::Sub for IPoint {
    type Output = Self;
    fn sub(self, o: Self) -> Self { Self::new(self.x.wrapping_sub(o.x), self.y.wrapping_sub(o.y)) }
}

/// `__mul(int)` 0x00451F60: `(int) roundf((float)v * (float)k)` por componente.
impl std::ops::Mul<i32> for IPoint {
    type Output = Self;
    fn mul(self, k: i32) -> Self {
        let f = |v: i32| f32_to_i32_trunc(((v as f32) * (k as f32)).round());
        Self::new(f(self.x), f(self.y))
    }
}

/// `GetDirection(Point)` 0x004B2340: (0,0) → 4 (`DIR_NONE`); senão reduz ao eixo dominante
/// (empate vai para o eixo Y), normaliza para ±1 e procura na tabela de vetores
/// UP(0,-1)=0, RIGHT(1,0)=1, DOWN(0,1)=2, LEFT(-1,0)=3.
pub fn get_direction(p: IPoint) -> i32 {
    if p.x == 0 && p.y == 0 { return 4; }
    let (ax, ay) = (p.x.wrapping_abs(), p.y.wrapping_abs());
    let (dx, dy) = if ax > ay { (p.x / ax, 0) } else { (0, p.y / ay) };
    match (dx, dy) {
        (0, -1) => 0,
        (1, 0) => 1,
        (0, 1) => 2,
        (-1, 0) => 3,
        _ => 4,
    }
}

/// `cvttss2si`: trunca; fora de faixa/NaN → `i32::MIN`.
pub fn f32_to_i32_trunc(v: f32) -> i32 {
    if v.is_nan() || !(-2147483648.0..2147483648.0).contains(&v) { INT_INDEFINITE } else { v as i32 }
}

/// Conversão número Lua → `int` feita por `lua_tointeger` do `lua5.1.dll` original
/// (`fld`/`fistp` com o modo padrão da FPU: arredonda para o par mais próximo).
/// Ex.: 2.5 → 2, 3.5 → 4, -2.5 → -2.
pub fn lua_number_to_int(v: f64) -> i32 {
    let r = v.round_ties_even();
    if r.is_nan() || !(-2147483648.0..2147483648.0).contains(&r) { INT_INDEFINITE } else { r as i32 }
}

/// `GL_Color`: quatro `float` (r, g, b em 0..255; a em 0..1 por convenção).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// `GL_Color()` (0x00466490): tudo 1.0.
    pub const fn lua_default() -> Self { Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }
    /// `GL_Color(r,g,b)` (0x00466610): alfa 1.0.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self { Self { r, g, b, a: 1.0 } }
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    /// `Normalize()` 0x0046C510: r,g,b / 255 (alfa intacto).
    pub fn normalize(self) -> Self { Self { r: self.r / 255.0, g: self.g / 255.0, b: self.b / 255.0, a: self.a } }
    /// `WithAlpha(a)` 0x0046C4E0.
    pub fn with_alpha(self, a: f32) -> Self { Self { a, ..self } }
}

/// `Rect2D`: quatro `int` (x, y, w, h).
/// Construtores (corpos em 0x00465D00, 0x00465DB0, 0x00465E64): `Rect2D()` = (-1,-1,0,0);
/// `(x,y,w,h)`, `(Point,int,int)`, `(Point,Point)` e `(Rect2D)` copiam 4 inteiros na ordem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self { Self { x, y, w, h } }
    pub const fn lua_default() -> Self { Self { x: -1, y: -1, w: 0, h: 0 } }
}
