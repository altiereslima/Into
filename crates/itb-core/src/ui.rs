//! Estruturas de dados de UI expostas à Lua (`Button` = C++ `ButtonDesc`).

use crate::geometry::{IPoint, Rect};

/// `ButtonDesc` (0x54 bytes). Construtores C++: 0x004692D0 `()`, 0x004691D0 `(image, Point)`,
/// 0x004690D0 `(image, Point, bool)`, 0x00468FD0 `(Rect2D, label)`, 0x00468EE0 `(Rect2D, label, int)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonDesc {
    /// +0x00 `image`
    pub image: String,
    /// +0x18 `pos`
    pub pos: IPoint,
    /// +0x20 `label`
    pub label: String,
    /// +0x38 `hitstats`
    pub hitstats: Rect,
    /// +0x48: 1 no construtor `(Rect2D, label)`; NÃO inicializado nos demais (lixo de memória
    /// no original). `None` representa "não inicializado".
    pub field_0x48: Option<i32>,
    /// +0x4C: 9 por padrão; tamanho informado em `(Rect2D, label, int)` (provável tamanho de fonte).
    pub field_0x4c: i32,
    /// +0x50: bool do construtor `(image, Point, bool)`; falso nos demais.
    pub field_0x50: bool,
}

impl Default for ButtonDesc {
    fn default() -> Self {
        Self { image: String::new(), pos: IPoint::new(0, 0), label: String::new(), hitstats: Rect::lua_default(),
               field_0x48: None, field_0x4c: 9, field_0x50: false }
    }
}

impl ButtonDesc {
    pub fn with_image(image: &str, pos: IPoint, flag: bool) -> Self {
        Self { image: image.into(), pos, field_0x50: flag, ..Default::default() }
    }
    pub fn with_rect(r: Rect, label: &str, size: Option<i32>) -> Self {
        Self {
            pos: IPoint::new(r.x, r.y), label: label.into(), hitstats: r,
            field_0x48: if size.is_none() { Some(1) } else { None },
            field_0x4c: size.unwrap_or(9),
            ..Default::default()
        }
    }
}
