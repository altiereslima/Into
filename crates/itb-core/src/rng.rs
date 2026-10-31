//! Gerador aleatório do jogo original.
//!
//! `Breach.exe` usa o `rand()` da CRT da Microsoft (0x0078478E): LCG por thread,
//! `seed = seed * 214013 + 2531011`, resultado `(seed >> 16) & 0x7FFF`; `srand` em 0x007847AF.
//! As funções Lua `random_int`/`random_bool` (0x004DFD50..0x004DFE00) são finas camadas sobre ele.
//! Reproduzir a sequência exata é o que permite, no futuro, comparar partidas com a mesma semente.

/// `rand()`/`srand()` da UCRT (MSVC). Semente inicial da CRT = 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsvcRand {
    seed: u32,
}

impl Default for MsvcRand {
    fn default() -> Self { Self { seed: 1 } }
}

/// Erro equivalente a uma divisão inteira por zero no código original (o EXE travaria).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DivideByZero;

impl MsvcRand {
    pub fn new(seed: u32) -> Self { Self { seed } }
    /// 0x007847AF `srand`.
    pub fn srand(&mut self, seed: u32) { self.seed = seed; }
    pub fn seed(&self) -> u32 { self.seed }

    /// 0x0078478E `rand`: 0..=32767.
    pub fn rand(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(0x343FD).wrapping_add(0x269EC3);
        ((self.seed >> 16) & 0x7FFF) as i32
    }

    /// 0x004DFD50 `random_int(n)`: `n == 0 → 0`, senão `rand() % n` (resto truncado, como `idiv`).
    /// Não consome número aleatório quando `n == 0`.
    pub fn random_int(&mut self, n: i32) -> i32 {
        if n == 0 { return 0; }
        self.rand().wrapping_rem(n)
    }

    /// 0x004DFD70 `random_int(a, b)`: `a == b → a`, senão `a + rand() % (b - a)` (b exclusivo).
    pub fn random_int_range(&mut self, a: i32, b: i32) -> i32 {
        if a == b { return a; }
        let r = self.rand();
        a.wrapping_add(r.wrapping_rem(b.wrapping_sub(a)))
    }

    /// 0x004DFDE0 `random_bool(n)`: `rand() % n == 0`. Com `n == 0` o original divide por zero.
    pub fn random_bool(&mut self, n: i32) -> Result<bool, DivideByZero> {
        let r = self.rand();
        if n == 0 { return Err(DivideByZero); }
        Ok(r.wrapping_rem(n) == 0)
    }

    /// 0x004DFE00 `random_bool(a, b)`: `rand() % b < a`.
    pub fn random_bool_chance(&mut self, a: i32, b: i32) -> Result<bool, DivideByZero> {
        let r = self.rand();
        if b == 0 { return Err(DivideByZero); }
        Ok(r.wrapping_rem(b) < a)
    }
}
