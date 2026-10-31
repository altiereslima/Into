//! Passagem e alcance em linha reta do tabuleiro (Etapa 09).
//!
//! Provas: `re/evidencias/rotinas_pathing.md` (decompilado com angr + conferido no assembly).

use crate::board_occupancy::{consts, OccupancyBoard, Point, Tile};

/// Tabela de vetores 0x008CF9AC (inicializador 0x00403030): UP, RIGHT, DOWN, LEFT.
pub const DIRS: [Point; 4] = [Point { x: 0, y: -1 }, Point { x: 1, y: 0 }, Point { x: 0, y: 1 }, Point { x: -1, y: 0 }];

/// 0x004B2C70: direções perpendiculares usadas pelo alcance "com cantos".
/// Verticais (0, 2) → [1, 3]; horizontais (1, 3) → [2, 0]; outras → nenhuma.
pub fn perpendicular(dir: usize) -> &'static [usize] {
    match dir {
        0 | 2 => &[1, 3],
        1 | 3 => &[2, 0],
        _ => &[],
    }
}

fn add(a: Point, b: Point) -> Point { Point::new(a.x.wrapping_add(b.x), a.y.wrapping_add(b.y)) }

impl OccupancyBoard {
    /// 0x0059F020 `Tile::GetPawnTeam`: time do primeiro item se `HasPawn(true)`, senão 2.
    pub fn tile_pawn_team(&self, tile: &Tile) -> i32 {
        if self.tile_has_pawn(tile, true) {
            if let Some(&i) = tile.pawns.first() { return self.pawns[i].team; }
        }
        consts::TEAM_NONE
    }

    /// 0x0059F260 `Tile::IsPassable(int pathProf)`. Os 4 bits baixos são o perfil; os demais, um
    /// time que pode atravessar unidades dele (perfil 0/2/…).
    pub fn tile_is_passable(&self, tile: &Tile, path_prof: i32) -> bool {
        let prof = path_prof & 0xF;
        let team_bits = path_prof >> 4;
        let terrain = tile.terrain;
        match prof {
            7 | 1 | 6 | 5 => true,
            4 => !matches!(terrain, 1 | 4 | 9),
            3 => !(terrain == 1 || terrain == 4 || self.tile_has_pawn(tile, true)),
            _ => {
                let massive = prof == 2;
                let occupied = self.tile_has_pawn(tile, true);
                let structure = occupied && tile.pawns.first().is_some_and(|&i| self.pawns[i].non_grid_structure);
                if occupied && self.tile_pawn_team(tile) != team_bits { return false; }
                if terrain == 1 || structure { return false; }
                if (!tile.flag_0x23ac || tile.counter_0x23cc >= 15) && terrain == 3 && !massive { return false; }
                !(terrain == 4 || terrain == 9)
            }
        }
    }

    /// 0x00561300 (vftable de Board, slot 0x14) `IsFree(x, y, prof)`: fora dos limites → false.
    pub fn is_free(&self, p: Point, path_prof: i32) -> bool {
        if !self.is_valid(p) { return false; }
        self.tile_is_passable(self.tile(p), path_prof)
    }

    /// 0x00573160 → 0x004CEC60 `Board:GetSimpleReachable(Point, int range, bool corners)`.
    ///
    /// Para cada direção (UP, RIGHT, DOWN, LEFT): anda por casas livres para projétil (perfil 3)
    /// até `range` passos, juntando cada casa (sem repetir); ao parar, acrescenta a casa seguinte
    /// (o obstáculo) se for válida, inédita e ainda dentro do alcance. Com `corners`, a partir de
    /// cada casa do raio olha as duas perpendiculares: se a vizinha estiver livre, ela entra direto
    /// (sem checar repetição nem alcance); se estiver ocupada, anda sobre as ocupadas (com repetição
    /// checada e alcance contado) e acrescenta a primeira livre depois delas.
    pub fn get_simple_reachable(&self, p: Point, range: i32, corners: bool) -> Vec<Point> {
        const PROJ: i32 = consts::PATH_PROJECTILE;
        let mut out: Vec<Point> = Vec::new();
        let push_unique = |out: &mut Vec<Point>, q: Point| if !out.contains(&q) { out.push(q) };
        for (d_idx, d) in DIRS.iter().enumerate() {
            let mut cur = p;
            let mut steps = 0;
            if self.is_free(add(p, *d), PROJ) {
                loop {
                    let next = add(cur, *d);
                    if !(self.is_valid(next) && steps < range) { break; }
                    cur = next;
                    push_unique(&mut out, next);
                    steps += 1;
                    if corners {
                        for &side_dir in perpendicular(d_idx) {
                            let dv = DIRS[side_dir];
                            let mut side = cur;
                            let mut s2 = steps;
                            if !self.is_free(add(cur, dv), PROJ) {
                                loop {
                                    let n = add(side, dv);
                                    if !self.is_valid(n) || s2 >= range { break; }
                                    side = n;
                                    push_unique(&mut out, n);
                                    s2 += 1;
                                    if self.is_free(add(side, dv), PROJ) { break; }
                                }
                            }
                            let last = add(side, dv);
                            if self.is_valid(last) { out.push(last); } // sem checagem de repetição/alcance
                        }
                    }
                    if !self.is_free(add(cur, *d), PROJ) { break; }
                }
            }
            let last = add(cur, *d);
            if self.is_valid(last) && !out.contains(&last) && steps < range {
                out.push(last);
            }
        }
        out
    }
}

fn manhattan(a: Point, b: Point) -> i32 {
    a.x.wrapping_sub(b.x).wrapping_abs().wrapping_add(a.y.wrapping_sub(b.y).wrapping_abs())
}

impl OccupancyBoard {
    /// 0x005731F0 → 0x004CD680 `Board:GetSimplePath(Point from, Point to)`.
    ///
    /// Tenta cada direção inicial (UP, RIGHT, DOWN, LEFT): anda em linha reta enquanto se aproxima do
    /// alvo, depois vira na direção de `GetDirection(alvo − atual)` e segue enquanto se aproxima. As
    /// casas precisam estar livres para PROJÉTIL (perfil 3), exceto a própria casa do alvo. A primeira
    /// tentativa que chega ao alvo é devolvida, começando por `from`. Sem sucesso (ou `from == to`,
    /// ou alvo inválido): lista vazia.
    pub fn get_simple_path(&self, from: Point, to: Point) -> Vec<Point> {
        const PROJ: i32 = consts::PATH_PROJECTILE;
        if from == to || !self.is_valid(to) { return Vec::new(); }
        let ok = |n: Point| self.is_free(n, PROJ) || n == to;
        for d in DIRS {
            let mut path = vec![from];
            let mut cur = from;
            let mut n = add(cur, d);
            if manhattan(to, n) >= manhattan(to, cur) { continue; }
            loop {
                if !ok(n) { break; }
                path.push(n);
                cur = n;
                n = add(cur, d);
                if manhattan(to, n) >= manhattan(to, cur) { break; }
            }
            let turn = crate::geometry::get_direction(crate::geometry::IPoint::new(to.x - cur.x, to.y - cur.y));
            if turn != 4 {
                let d2 = DIRS[turn as usize];
                let mut n = add(cur, d2);
                while manhattan(to, n) < manhattan(to, cur) {
                    if !ok(n) { break; }
                    path.push(n);
                    cur = n;
                    n = add(cur, d2);
                }
            }
            if cur == to { return path; }
        }
        Vec::new()
    }
}
