//! Preparação da missão — Etapa 13, lida na 1.2.93.
//!
//! `BoardPlayer::LoadMission` (0x580D50) carrega o mapa e, antes de `BaseStart`, chama
//! `BoardPlayer::SetupBuildingHealth` (0x591540 → `Board::SetBuildingHP` 0x56D260) e
//! `BoardPlayer::RandomizeEnvironment` (0x58B380), que usa `Board::RandomizeTerrain` (0x568940) e
//! `Board::SetWeather` (0x56D580). As chances vêm de funções Lua; aqui ficam só as regras do tabuleiro.
//! Provas em `re/evidencias/v1.2.93/rotinas_preparo.md`.

use crate::board_occupancy::{consts::*, OccupancyBoard, Point};
use crate::rng::MsvcRand;

/// Valor especial de `SetTerrain`: rachadura.
pub const TERRAIN_CRACK: i32 = 16;
/// Tipo de clima que `RandomizeEnvironment` grava: chuva.
pub const WEATHER_RAIN: i32 = 3;

/// Argumentos de `Board::SetWeather(int, int, Point, Point, float)`, guardados em Board+0x2DA0.
#[derive(Clone, Debug, PartialEq)]
pub struct Weather {
    pub kind: i32,
    pub subtype: i32,
    pub from: Point,
    pub to: Point,
    pub time: f32,
}

/// Quantos prédios recebem a vida `i + 1` (0x56D2C0..0x56D303). A conta é em `float`:
/// `v / 100.0f * n`, com `n` convertido como inteiro sem sinal; depois `ceil` em `double` (0x476150 → 0x799B30,
/// o `ceil` da CRT) e truncamento (`cvttss2si`).
pub fn building_count(percent: i32, n: usize) -> i32 {
    let n = n as u32 as f64 as f32;
    let v = percent as f32 / 100.0f32 * n;
    (v as f64).ceil() as f32 as i32
}

impl OccupancyBoard {
    /// 0x573260: lista de prédios, em x e depois y, das casas com terreno 1 (campo +0x2AE0, sem `IsTerrain`).
    pub fn update_building_list(&mut self) {
        let size = self.get_size();
        self.buildings = (0..size.x).flat_map(|x| (0..size.y).map(move |y| Point::new(x, y)))
            .filter(|&p| self.tile(p).terrain == TERRAIN_BUILDING).collect();
    }

    /// 0x568940 `Board::RandomizeTerrain(int tipo, int chance)`. Chance 0 não faz nada. O terreno-base é
    /// água para gelo e chão para os outros. Em x e depois y, só as casas do terreno-base gastam `rand()`;
    /// com `rand() % 100 < chance` e `custom` vazio, `BoardSpace::SetTerrain(tipo)`. A lista de prédios não
    /// é refeita.
    pub fn randomize_terrain(&mut self, kind: i32, chance: i32, rng: &mut MsvcRand) {
        if chance == 0 { return; }
        let base = if kind == TERRAIN_ICE { TERRAIN_WATER } else { TERRAIN_ROAD };
        let size = self.get_size();
        for x in 0..size.x {
            for y in 0..size.y {
                let p = Point::new(x, y);
                if self.tile(p).terrain != base { continue; }
                if rng.rand() % 100 >= chance { continue; }
                if self.tile(p).custom_tile.is_empty() { self.set_terrain(p, kind); }
            }
        }
    }

    /// 0x56D260 `Board::SetBuildingHP(std::vector<int>)`: `percent[i]` é a fatia de prédios com vida `i + 1`.
    ///
    /// 1. Conta de cada fatia, por `building_count`, sobre o tamanho da lista de prédios.
    /// 2. Para cada prédio da lista, a primeira fatia com conta não nula (o índice só avança, até 4):
    ///    `SetTerrain(1)`, vida `(i + 1)/(i + 1)` por 0x5AF190 e, com vida atual 0, `SetTerrain(2)`.
    ///    Sem fatia restante, para.
    /// 3. Em x e depois y, cada casa de entulho gasta um `rand() % 100` e recebe vida `0/m`, com `m` = 1 + o
    ///    primeiro índice em que o valor fica abaixo da soma acumulada, ou 1.
    pub fn set_building_hp(&mut self, percent: &[i32], rng: &mut MsvcRand) {
        let n = self.buildings.len();
        let mut count: Vec<i32> = percent.iter().map(|&v| building_count(v, n)).collect();
        // o laço lê 4 posições fixas; o jogo sempre passa 4 valores
        count.resize(count.len().max(4), 0);
        let mut i = 0;
        for k in 0..n {
            while i < 4 && count[i] == 0 { i += 1; }
            if i >= 4 { break; }
            let p = self.buildings[k];
            self.set_terrain(p, TERRAIN_BUILDING);
            let hp = i as i32 + 1;
            self.tile_set_health(p, hp, hp);
            if self.tile(p).health == 0 { self.set_terrain(p, TERRAIN_RUBBLE); }
            count[i] -= 1;
        }
        let size = self.get_size();
        for x in 0..size.x {
            for y in 0..size.y {
                let p = Point::new(x, y);
                if self.tile(p).terrain != TERRAIN_RUBBLE { continue; }
                let r = rng.rand() % 100;
                let mut sum = 0;
                let mut max = 1;
                for (j, &v) in percent.iter().enumerate() {
                    sum += v;
                    if r < sum { max = j as i32 + 1; break; }
                }
                self.tile_set_health(p, 0, max);
            }
        }
    }

    /// 0x56D580 `Board::SetWeather`: repassa os argumentos ao objeto de clima (0x504840, não reconstruído).
    pub fn set_weather(&mut self, kind: i32, subtype: i32, from: Point, to: Point, time: f32) {
        self.weather = Some(Weather { kind, subtype, from, to, time });
    }

    /// 0x55FFB0 `Board:GetHealth(Point)`: vida atual da casa (+0x78, copiada por 0x5AF100), ou da sentinela.
    pub fn get_health(&self, p: Point) -> i32 { self.tile(p).health }
}

/// Subtipo da chuva por setor (0x58B5D1..0x58B653): 2 em "snow", 1 em "acid", 0 nos outros.
pub fn rain_subtype(sector: &str) -> i32 {
    match sector {
        "snow" => 2,
        "acid" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(rows: &[&str]) -> OccupancyBoard {
        let mut b = OccupancyBoard::new(rows[0].len() as i32, rows.len() as i32).unwrap();
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let t = match c { 'B' => TERRAIN_BUILDING, 'R' => TERRAIN_RUBBLE, 'W' => TERRAIN_WATER, 'M' => TERRAIN_MOUNTAIN, _ => TERRAIN_ROAD };
                b.tile_mut(Point::new(x as i32, y as i32)).unwrap().terrain = t;
            }
        }
        b.update_building_list();
        b
    }

    #[test]
    fn contagem_em_float_com_ceil() {
        assert_eq!(building_count(50, 7), 4);
        assert_eq!(building_count(40, 7), 3);
        assert_eq!(building_count(10, 7), 1);
        assert_eq!(building_count(0, 7), 0);
        assert_eq!(building_count(50, 0), 0);
        assert_eq!(building_count(10, 10), 1);
        // 0.6f * 25 = 15.000001f: o ceil sobe para 16, onde a conta exata daria 15
        assert_eq!(building_count(60, 25), 16);
    }

    #[test]
    fn lista_de_predios_em_x_depois_y() {
        let b = board(&["B.B", ".B.", "B.."]);
        assert_eq!(b.buildings, vec![Point::new(0, 0), Point::new(0, 2), Point::new(1, 1), Point::new(2, 0)]);
    }

    #[test]
    fn randomize_terrain_gasta_rand_so_no_terreno_base() {
        let mut b = board(&["..", "BW"]);
        let mut rng = MsvcRand::new(7);
        let mut copia = rng.clone();
        b.randomize_terrain(TERRAIN_FOREST, 100, &mut rng);
        // duas casas de chão; (0,1) é prédio e (1,1) é água
        for _ in 0..2 { copia.rand(); }
        assert_eq!(rng, copia);
        assert_eq!(b.get_terrain(Point::new(0, 0)), TERRAIN_FOREST);
        assert_eq!(b.get_terrain(Point::new(1, 0)), TERRAIN_FOREST);
        assert_eq!(b.get_terrain(Point::new(0, 1)), TERRAIN_BUILDING);
        // gelo parte da água
        b.randomize_terrain(TERRAIN_ICE, 100, &mut rng);
        assert_eq!(b.get_terrain(Point::new(1, 1)), TERRAIN_ICE);
        // chance 0 não gasta nada
        let antes = rng.clone();
        b.randomize_terrain(TERRAIN_SAND, 0, &mut rng);
        assert_eq!(rng, antes);
    }

    #[test]
    fn randomize_terrain_respeita_custom() {
        let mut b = board(&["."]);
        b.tile_mut(Point::new(0, 0)).unwrap().custom_tile = "ground_x".into();
        let mut rng = MsvcRand::new(1);
        b.randomize_terrain(TERRAIN_FOREST, 100, &mut rng);
        assert_eq!(b.get_terrain(Point::new(0, 0)), TERRAIN_ROAD);
        // o rand() é gasto antes de olhar `custom`
        let mut copia = MsvcRand::new(1);
        copia.rand();
        assert_eq!(rng, copia);
    }

    #[test]
    fn vida_dos_predios_por_fatia() {
        let mut b = board(&["BBBB", "R..."]);
        let mut rng = MsvcRand::new(3);
        let mut copia = rng.clone();
        b.set_building_hp(&[50, 40, 10, 0], &mut rng);
        // 4 prédios: ceil(2) = 2 com vida 1, ceil(1.6) = 2 com vida 2, ceil(0.4) = 1 com vida 3 (sobra)
        let vidas: Vec<i32> = b.buildings.iter().map(|&p| b.get_health(p)).collect();
        assert_eq!(vidas, vec![1, 1, 2, 2]);
        // uma casa de entulho, um rand()
        let r = copia.rand() % 100;
        assert_eq!(rng, copia);
        let t = b.tile(Point::new(0, 1));
        let esperado = if r < 50 { 1 } else if r < 90 { 2 } else { 3 };
        assert_eq!((t.health, t.max_health), (0, esperado));
    }

    #[test]
    fn sem_fatia_restante_o_predio_fica_como_estava() {
        let mut b = board(&["BBB"]);
        for x in 0..3 {
            let t = b.tile_mut(Point::new(x, 0)).unwrap();
            t.health = 4;
            t.max_health = 4;
        }
        b.set_building_hp(&[50, 0, 0, 0], &mut MsvcRand::new(1));
        // ceil(1.5) = 2 prédios com vida 1; o terceiro mantém 4
        let vidas: Vec<i32> = b.buildings.iter().map(|&p| b.get_health(p)).collect();
        assert_eq!(vidas, vec![1, 1, 4]);
    }

    #[test]
    fn subtipo_da_chuva() {
        assert_eq!(rain_subtype("snow"), 2);
        assert_eq!(rain_subtype("acid"), 1);
        assert_eq!(rain_subtype("grass"), 0);
    }
}
