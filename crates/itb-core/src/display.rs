//! Tamanho virtual da tela e escalas de UI (0x004B2560, 0x004B25F0, 0x004D47F0, 0x004D4640).
//!
//! O EXE lê três globais: `A` = [0x0088F10C], `B` = [0x0088F114] (ambas inteiras) e
//! `F` = [0x0088EB4C] (float, 1.0 no arquivo, alterado em tempo de execução).
//! Pela forma do código, `A`/`B` são a altura da janela e `F` a razão de aspecto
//! (ScreenSizeX = ScreenSizeY * F). **Hipótese ainda não rastreada até a escrita dessas
//! globais**; a aritmética abaixo, porém, é a do EXE, em `f32` e com truncamento `cvttss2si`.

use crate::geometry::f32_to_i32_trunc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Display {
    /// [0x0088F10C]
    pub a: i32,
    /// [0x0088F114]
    pub b: i32,
    /// [0x0088EB4C]
    pub f: f32,
    /// Configuração "max_scale" lida por GetBoardScale (padrão 5 no EXE).
    pub max_scale: i32,
}

impl Display {
    /// Janela `w`×`h` segundo a hipótese acima.
    pub fn from_window(w: i32, h: i32) -> Self {
        Self { a: h, b: h, f: w as f32 / h as f32, max_scale: 5 }
    }

    fn ratio(&self) -> f32 {
        if self.b < 720 { self.b as f32 / 720.0 } else { 1.0 }
    }

    /// 0x004B25F0 `ScreenSizeY()`.
    pub fn screen_size_y(&self) -> i32 { f32_to_i32_trunc(self.a as f32 / self.ratio()) }

    /// 0x004B2560 `ScreenSizeX()`.
    pub fn screen_size_x(&self) -> i32 { f32_to_i32_trunc(self.screen_size_y() as f32 * self.f) }

    /// 0x004D47F0 `GetDefaultScale()`: 2, 3 ou 4 conforme o tamanho virtual.
    pub fn default_scale(&self) -> i32 {
        let y = self.screen_size_y();
        if y < 1070 { return 2; }
        if self.screen_size_x() < 1420 { return 2; }
        if y < 1440 { return 3; }
        if self.screen_size_x() < 2560 { return 3; }
        4
    }

    /// 0x004D4640 `GetBoardScale()`: `min(max_scale, GetDefaultScale())`.
    pub fn board_scale(&self) -> i32 { self.max_scale.min(self.default_scale()) }
}
