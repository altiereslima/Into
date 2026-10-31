//! Primeira rotina nativa recuperada por análise estática de Breach.exe.
//!
//! Função original: VA 0x00561370 (PE32 x86), chamada via luabind.
//! O `this` original chega em ECX; x/y são argumentos na pilha.
//! Lê limites inteiros com sinal em [ECX + 0x48] e [ECX + 0x4C].
//! Rejeita x<0, x>=largura, y<0 ou y>=altura.
//! Esta implementação Rust NÃO representa a ABI/estrutura binária original.

/// Offsets observados no objeto C++ original; não são offsets deste tipo Rust.
pub const ORIGINAL_WIDTH_OFFSET: usize = 0x48;
pub const ORIGINAL_HEIGHT_OFFSET: usize = 0x4C;

/// Modelo semântico portátil; NÃO é a reconstrução completa de `Board`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardBounds {
    pub width: i32,
    pub height: i32,
}

impl BoardBounds {
    #[inline]
    pub const fn is_valid(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }
}

/// Verificação opcional dos offsets vistos no código de máquina.
/// Esta representação é apenas DOCUMENTAL; não deve ser usada para acessar
/// instâncias do processo original por FFI sem validar layout e lifetime.
#[repr(C)]
pub struct ObservedBoardPrefix {
    pub unknown_0x00_0x47: [u8; 0x48],
    pub width_at_0x48: i32,
    pub height_at_0x4c: i32,
}
