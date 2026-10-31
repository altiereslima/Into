//! Consultas de terreno da Etapa 10: `IsTerrain`, `IsCrackable`, `IsPod`, `IsItem`, `IsDangerous`,
//! `IsSpawning`. Provas: `re/evidencias/rotinas_alcance.md`.

use crate::board_occupancy::{consts::*, OccupancyBoard, Point, Tile};

/// Valor especial de `IsTerrain`: chão rachado (lê o byte `IsCracked`).
pub const TERRAIN_QUERY_CRACKED: i32 = 16;
/// Valor especial de `IsTerrain`: gelo cujo objeto de vida vale 1 (gelo já danificado).
pub const TERRAIN_QUERY_DAMAGED_ICE: i32 = 18;

impl OccupancyBoard {
    /// 0x005AE5D0 `Tile::IsTerrain(int)`.
    ///
    /// * 3 (água): falso se o byte +0x23AC estiver ligado com contador +0x23CC < 15; senão terreno == 3.
    /// * 14 (lava): `IsTerrain(3)` e byte +0x2AC8.
    /// * 18: `IsTerrain(5)` (gelo) e vida atual (+0x78) == 1.
    /// * 16: byte `IsCracked` (+0x2B26).
    /// * demais: terreno == valor.
    pub fn tile_is_terrain(tile: &Tile, t: i32) -> bool {
        match t {
            TERRAIN_WATER => {
                if tile.flag_0x23ac && tile.counter_0x23cc < 15 { return false; }
                tile.terrain == TERRAIN_WATER
            }
            TERRAIN_LAVA => Self::tile_is_terrain(tile, TERRAIN_WATER) && tile.lava_0x2ac8,
            TERRAIN_QUERY_DAMAGED_ICE => Self::tile_is_terrain(tile, TERRAIN_ICE) && tile.health == 1,
            TERRAIN_QUERY_CRACKED => tile.cracked != 0,
            _ => tile.terrain == t,
        }
    }

    /// 0x0055F0D0 `Board:IsTerrain(Point, int)` (WeaponTarget slot 0x7C): casa real ou sentinela.
    pub fn is_terrain(&self, p: Point, t: i32) -> bool { Self::tile_is_terrain(self.tile(p), t) }

    /// 0x00568860 → 0x005AE6B0 `Board:IsCrackable(Point)`.
    ///
    /// Recusa, nesta ordem: unidade viva cujo tipo é "SatelliteRocket"; string +0x26F8 ==
    /// "supervolcano"; DWORD +0x2BA0 != 0; "conveyor" na string da casa; água (com a mesma exceção
    /// de `IsTerrain(3)`); lava; buraco; prédio. Caso contrário, verdadeiro.
    pub fn is_crackable(&self, p: Point) -> bool {
        let tile = self.tile(p);
        if self.tile_has_pawn(tile, true) {
            let first = tile.pawns.first().map(|&i| self.pawns[i].type_name.as_str());
            if first == Some("SatelliteRocket") { return false; }
        }
        if tile.str_0x26f8 == "supervolcano" || tile.int_0x2ba0 != 0 { return false; }
        if tile.custom_tile.contains("conveyor") { return false; }
        if Self::tile_is_terrain(tile, TERRAIN_WATER) { return false; }
        if Self::tile_is_terrain(tile, TERRAIN_WATER) && tile.lava_0x2ac8 { return false; }
        !matches!(tile.terrain, TERRAIN_HOLE | TERRAIN_BUILDING)
    }

    /// 0x00571680 `Board:IsPod(Point)`: `p` é a casa da cápsula (Board+0x2D70) e a casa tem +0x2AE4 == 1.
    pub fn is_pod(&self, p: Point) -> bool {
        p == self.pod_space && self.tile(self.pod_space).pod_0x2ae4 == 1
    }

    /// 0x005716F0 `Board:IsItem(Point)`: DWORD +0x2B04 != 0 (sentinela se inválida).
    pub fn is_item(&self, p: Point) -> bool { self.tile(p).item_0x2b04 != 0 }

    /// 0x005717A0 `Board:IsDangerous(Point)`: byte +0x2AF0, ou `p` em uma das listas Board+0x7460/0x7470.
    pub fn is_dangerous(&self, p: Point) -> bool {
        self.tile(p).danger_0x2af0 || self.danger_list_a.contains(&p) || self.danger_list_b.contains(&p)
    }

    /// 0x0056D5B0 `Board:IsSpawning(Point)`: byte +0x27E8, ou `p` na lista Board+0x2D50.
    pub fn is_spawning(&self, p: Point) -> bool {
        self.tile(p).spawning_0x27e8 || self.spawn_list.contains(&p)
    }
}
