//! Consultas do tabuleiro reconstruídas por análise estática do Breach.exe v1.2.88.
//!
//! Implementação semântica portátil, NÃO representa o layout binário C++ original.
//! Referências precisas em `evidencias/rotinas_board.md`.
//! Não reconstruímos criação, movimentação nem dano de Pawn nesta etapa.

use crate::board::BoardBounds;

/// ID da equipe retornado pelo motor original quando não há pawn válido
/// (retorno imediato em 0x005689A5). Etapa 05: confirmado como `TEAM_NONE` (= 2),
/// constante registrada pelo próprio EXE.
pub const ORIGINAL_ABSENT_PAWN_TEAM: i32 = 2;
/// `IsBuilding`: compare DWORD [tile + 0x2AE0], 1 (VA 0x00567D00).
pub const ORIGINAL_BUILDING_TERRAIN: i32 = 1;
/// Distância (em bytes) entre o início das colunas x e o registro y no EXE.
pub const ORIGINAL_CELL_Y_STRIDE: usize = 0x2BBC;
pub const ORIGINAL_TERRAIN_OFFSET: usize = 0x2AE0;
pub const ORIGINAL_FIRE_OFFSET: usize = 0x2AEC;
pub const ORIGINAL_VINES_OFFSET: usize = 0x2AA0;
pub const ORIGINAL_SMOKE_OFFSET: usize = 0x2B25;
pub const ORIGINAL_ACID_OFFSET: usize = 0x2AF1;
pub const ORIGINAL_CRACKED_OFFSET: usize = 0x2B26;

/// Representação dos campos de uma casa *relevantes para consultas*.
/// O EXE tem muitos outros campos, e esta estrutura não replica seus offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CellState {
    pub terrain: i32,
    pub fire_state: i32,
    pub smoke_state: u8,
    pub vines_state: u8,
    pub acid_state: u8,
    pub cracked_state: u8,
    /// Primeira unidade que satisfaz o filtro de `0x0059F050` no modelo semântico.
    /// Sem reconstruir o objeto Pawn, só guardamos o ID da equipe filtrada.
    pub active_pawn_team: Option<i32>,
}

/// Fotografia do tabuleiro, independente de `Breach.exe`, SDL ou Lua.
/// Usamos vetor em linhas; o executável possui uma estrutura diferente.
#[derive(Clone, Debug)]
pub struct BoardSnapshot {
    bounds: BoardBounds,
    cells: Vec<CellState>,
    /// Casa sentinela retornada pelo acesso nativo 0x0055DC30 em posição inválida.
    /// Seus valores reais dependem do objeto Board carregado no jogo original.
    fallback: CellState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoardError { InvalidDimensions, ExcessiveArea, OutOfBounds }

impl BoardSnapshot {
    pub fn new(width: i32, height: i32, fallback: CellState) -> Result<Self, BoardError> {
        if width <= 0 || height <= 0 { return Err(BoardError::InvalidDimensions); }
        let w = usize::try_from(width).map_err(|_| BoardError::InvalidDimensions)?;
        let h = usize::try_from(height).map_err(|_| BoardError::InvalidDimensions)?;
        let len = w.checked_mul(h).ok_or(BoardError::ExcessiveArea)?;
        // Proteção do modelo Rust contra entradas incorretas; não é limite observado no EXE.
        if len > 1_048_576 { return Err(BoardError::ExcessiveArea); }
        Ok(Self { bounds: BoardBounds { width, height }, cells: vec![CellState::default(); len], fallback })
    }

    pub fn bounds(&self) -> BoardBounds { self.bounds }
    pub fn is_valid(&self, x: i32, y: i32) -> bool { self.bounds.is_valid(x, y) }
    pub fn set_cell(&mut self, x: i32, y: i32, cell: CellState) -> Result<(), BoardError> {
        let i = self.index(x, y).ok_or(BoardError::OutOfBounds)?;
        self.cells[i] = cell;
        Ok(())
    }
    pub fn get_cell(&self, x: i32, y: i32) -> &CellState {
        self.index(x, y).and_then(|i| self.cells.get(i)).unwrap_or(&self.fallback)
    }
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if !self.is_valid(x, y) { return None; }
        let x = usize::try_from(x).ok()?;
        let y = usize::try_from(y).ok()?;
        Some(y * usize::try_from(self.bounds.width).ok()? + x)
    }

    /// 0x00503B60 -> 0x0055DC30 -> [tile + 0x2AE0].
    pub fn get_terrain(&self, x: i32, y: i32) -> i32 { self.get_cell(x, y).terrain }

    /// 0x00567D00: `GetTerrain(x,y) == 1`.
    pub fn is_building(&self, x: i32, y: i32) -> bool {
        self.get_terrain(x, y) == ORIGINAL_BUILDING_TERRAIN
    }

    /// 0x00567C50: DWORD fire_state != 0, inclusive para casa inválida (sentinela).
    pub fn is_fire(&self, x: i32, y: i32) -> bool { self.get_cell(x, y).fire_state != 0 }

    /// 0x005653E0: BYTE smoke_state, convertido em booleano para esta API semântica.
    pub fn is_smoke(&self, x: i32, y: i32) -> bool { self.get_cell(x, y).smoke_state != 0 }

    /// 0x005653C0: BYTE vines_state, convertido em booleano para esta API semântica.
    pub fn is_vines(&self, x: i32, y: i32) -> bool { self.get_cell(x, y).vines_state != 0 }

    /// 0x00567CB0: BYTE [tile+0x2AF1], ou [this+0x2B4D] na casa sentinela.
    pub fn is_acid(&self, x: i32, y: i32) -> bool { self.get_cell(x, y).acid_state != 0 }

    /// 0x00569920: BYTE [tile+0x2B26], ou [this+0x2B82] na casa sentinela.
    pub fn is_cracked(&self, x: i32, y: i32) -> bool { self.get_cell(x, y).cracked_state != 0 }

    /// 0x0056FB20: compara com x==0, y==0, x==width-1, y==height-1.
    /// Não faz `is_valid`; coordenadas negativas/out-of-bounds não são rejeitadas
    /// por uma guarda genérica. Usamos wrapping_sub para o DEC x86 de 32 bits.
    pub fn is_edge(&self, x: i32, y: i32) -> bool {
        x == 0 || y == 0 || x == self.bounds.width.wrapping_sub(1) ||
            y == self.bounds.height.wrapping_sub(1)
    }

    /// SUPERADO na Etapa 05 por `board_occupancy::OccupancyBoard::get_pawn_team`,
    /// que reconstrói o filtro 0x0059F050 por completo. Mantido por compatibilidade.
    /// 0x00568950: seleciona a primeira unidade aceita por `0x0059F050(1)`;
    /// quando não há tal unidade, retorna 2. `active_pawn_team` é dado já filtrado;
    /// a seleção completa de unidades do EXE ainda não foi recuperada.
    pub fn get_pawn_team(&self, x: i32, y: i32) -> i32 {
        self.get_cell(x, y).active_pawn_team.unwrap_or(ORIGINAL_ABSENT_PAWN_TEAM)
    }
}
