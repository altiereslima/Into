//! Alcance (Dijkstra) e caminho (A*) do tabuleiro — `Board:GetReachable` e `Board:GetPath` (Etapa 10).
//!
//! Provas: `re/evidencias/rotinas_alcance.md` (assembly de 0x004CDA00, 0x004CDE60, 0x004CE630 e
//! das rotinas de heap 0x004CF0B0/0x004D1A30/0x004D1AD0).

use crate::board_occupancy::{consts, OccupancyBoard, Point};
use crate::pathing::DIRS;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Nó da fila: (prioridade float, ponto). Ordem do `std::set` do EXE: prioridade, depois x, depois y.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Node { f: f32, p: Point }

impl Node {
    /// Comparador "menor que" do EXE (0x004CE2B6): `a.f < b.f`; empate → `a.x < b.x`; empate → `a.y < b.y`.
    fn less(a: &Node, b: &Node) -> bool {
        if a.f != b.f { return b.f > a.f; }
        if a.p.x != b.p.x { return a.p.x < b.p.x; }
        a.p.y < b.p.y
    }
}

impl Eq for Node {}
impl PartialOrd for Node { fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) } }
impl Ord for Node {
    fn cmp(&self, o: &Self) -> Ordering {
        if Node::less(self, o) { Ordering::Less } else if Node::less(o, self) { Ordering::Greater } else { Ordering::Equal }
    }
}

fn add(a: Point, b: Point) -> Point { Point::new(a.x.wrapping_add(b.x), a.y.wrapping_add(b.y)) }

/// Resultado do Dijkstra (o EXE o guarda num cache por casa de origem em Board+4).
#[derive(Clone, Debug, Default)]
pub struct ReachMap {
    /// Distância por casa, na ordem do `std::map<Point,int>` (x, depois y). Não alcançadas: `i32::MAX`.
    pub dist: BTreeMap<Point, i32>,
    /// Predecessor por casa ((-1,-1) se nenhum).
    pub prev: BTreeMap<Point, Point>,
}

impl OccupancyBoard {
    /// 0x00573540 (Board vftable slot 0x10) `IsEdgeBlocked(x, y, dir, prof)`: paredes da casa de
    /// SAÍDA (+0x2AC4 + dir; `dir == 4` testa as quatro). Voador, projétil, saltador e teleporte
    /// ignoram paredes.
    pub fn is_edge_blocked(&self, p: Point, dir: i32, path_prof: i32) -> bool {
        let tile = self.tile(p);
        let wall = if dir == 4 { tile.edge_block_0x2ac4.iter().any(|&b| b) }
            else { usize::try_from(dir).ok().and_then(|d| tile.edge_block_0x2ac4.get(d)).copied().unwrap_or(false) };
        if !wall { return false; }
        !matches!(path_prof & 0xF, 1 | 3 | 6 | 5)
    }

    /// 0x005612A0 (slot 0x18) → 0x0059F3A0 `Tile::GetMoveCost(prof)`: 1000 para PRÉDIO com `prof == 3`
    /// exato (sem máscara); 1 em todos os outros casos.
    pub fn move_cost(&self, p: Point, path_prof: i32) -> i32 {
        if path_prof == consts::PATH_PROJECTILE && self.tile(p).terrain == consts::TERRAIN_BUILDING { 1000 } else { 1 }
    }

    /// Custo do passo `cur → n` usado pelo Dijkstra e pelo A*: 1 saindo da origem; senão
    /// slot 0x1C (0x00561280, sempre 0) + `move_cost(n)`.
    fn step_cost(&self, start: Point, cur: Point, n: Point, prof: i32) -> i32 {
        if cur == start { 1 } else { self.move_cost(n, prof) }
    }

    /// 0x004CDE60: Dijkstra a partir de `start` com custo limitado por `range`.
    ///
    /// Inicializa todas as casas do tabuleiro com `i32::MAX`. A fila é um `std::set` ordenado por
    /// (distância como float, x, y); a melhoria de distância apaga a entrada antiga e insere a nova.
    /// Para ao tirar da fila uma casa cuja distância já é `>= range`. Vizinhos em UP, RIGHT, DOWN,
    /// LEFT; descarta saída bloqueada por parede e destino não livre (`IsFree`).
    pub fn reach_map(&self, start: Point, range: i32, prof: i32) -> ReachMap {
        let size = self.get_size();
        let mut rm = ReachMap::default();
        for x in 0..size.x {
            for y in 0..size.y {
                rm.dist.insert(Point::new(x, y), i32::MAX);
                rm.prev.insert(Point::new(x, y), Point::INVALID);
            }
        }
        let mut open: BTreeSet<Node> = BTreeSet::new();
        rm.dist.insert(start, 0);
        open.insert(Node { f: 0.0, p: start });
        rm.prev.insert(start, Point::INVALID);
        while let Some(cur) = open.pop_first() {
            let cur_d = *rm.dist.entry(cur.p).or_insert(0);
            if cur_d >= range { break; }
            for (dir, d) in DIRS.iter().enumerate() {
                let n = add(cur.p, *d);
                if self.is_edge_blocked(cur.p, dir as i32, prof) || !self.is_free(n, prof) { continue; }
                let nd = rm.dist.get(&cur.p).copied().unwrap_or(0).wrapping_add(self.step_cost(start, cur.p, n, prof));
                let old = *rm.dist.entry(n).or_insert(0);
                if nd < old {
                    open.remove(&Node { f: old as f32, p: n });
                    rm.dist.insert(n, nd);
                    rm.prev.insert(n, cur.p);
                    open.insert(Node { f: nd as f32, p: n });
                }
            }
        }
        rm
    }

    /// 0x00573280 → 0x004CDA00 `Board:GetReachable(Point, int range, int pathProf)`.
    ///
    /// Origem inválida → vazio. Senão: casas do Dijkstra com distância `<= range` e não bloqueadas
    /// (`IsBlocked`), em ordem (x, y). Depois, exceto com `prof == 3` exato, remove casas bloqueadas
    /// e água (`terreno == 3`, sem a exceção de `IsTerrain`) para saltadores (`prof & 0xF == 6`).
    pub fn get_reachable(&self, start: Point, range: i32, prof: i32) -> Vec<Point> {
        if !self.is_valid(start) { return Vec::new(); }
        let rm = self.reach_map(start, range, prof);
        let mut out: Vec<Point> = rm.dist.iter()
            .filter(|(p, &d)| d <= range && !self.is_blocked(**p, prof))
            .map(|(p, _)| *p)
            .collect();
        if prof != consts::PATH_PROJECTILE {
            let kind = prof & 0xF;
            out.retain(|&p| !((self.tile(p).terrain == consts::TERRAIN_WATER && kind == consts::PATH_JUMPER) || self.is_blocked(p, prof)));
        }
        out
    }

    /// 0x005733D0 → 0x004CE630 `Board:GetPath(Point from, Point to, int pathProf)`: A*.
    ///
    /// `from == to` ou ponta inválida → vazio. Fila: heap binário do `std::priority_queue` (MSVC)
    /// com mínimo em (f, x, y), `f = |Δ| × 1.01f + g` em float. O destino é aceito mesmo ocupado.
    /// Sem `decrease-key`: entradas antigas ficam no heap. Resultado: de `from` até `to`, inclusive.
    pub fn get_path(&self, from: Point, to: Point, prof: i32) -> Vec<Point> {
        if from == to || !self.is_valid(from) || !self.is_valid(to) { return Vec::new(); }
        let mut heap: Vec<Node> = Vec::new();
        let mut closed: HashSet<Point> = HashSet::new();
        let mut seen: HashSet<Point> = HashSet::new();
        let mut g: HashMap<Point, i32> = HashMap::new();
        let mut came: HashMap<Point, Point> = HashMap::new();
        g.insert(from, 0);
        seen.insert(from);
        heap_push(&mut heap, Node { f: 0.0, p: from });
        while !heap.is_empty() {
            let cur = heap_pop(&mut heap);
            if cur.p == to {
                let mut path = Vec::new();
                let mut c = to;
                loop {
                    if c == from { break; }
                    path.push(c);
                    // `operator[]` cria `Point()` = (0x80000001, 0x80000001) se faltar.
                    c = came.get(&c).copied().unwrap_or(Point::new(i32::MIN + 1, i32::MIN + 1));
                    if c.x < 0 || c.y < 0 { break; }
                }
                path.push(c);
                path.reverse();
                return path;
            }
            closed.insert(cur.p);
            for (dir, d) in DIRS.iter().enumerate() {
                let n = add(cur.p, *d);
                if self.is_edge_blocked(cur.p, dir as i32, prof) { continue; }
                if !self.is_free(n, prof) && n != to { continue; }
                if closed.contains(&n) { continue; }
                let ng = self.step_cost(from, cur.p, n, prof).wrapping_add(*g.entry(cur.p).or_insert(0));
                if seen.contains(&n) && ng >= *g.entry(n).or_insert(0) { continue; }
                came.insert(n, cur.p);
                let h = to.y.wrapping_sub(n.y).wrapping_abs().wrapping_add(to.x.wrapping_sub(n.x).wrapping_abs());
                let f = (h as f32) * 1.01f32 + (ng as f32);
                heap_push(&mut heap, Node { f, p: n });
                seen.insert(n);
                g.insert(n, ng);
            }
        }
        Vec::new()
    }
}

/// Predicado do heap (0x004D1A30/0x004D1AD0): "a vem depois de b" = `a.f > b.f`, empate → `a.x > b.x`,
/// empate → `a.y > b.y`. Com ele o topo do heap é o MENOR (f, x, y).
fn after(a: &Node, b: &Node) -> bool {
    if a.f != b.f { return a.f > b.f; }
    if a.p.x != b.p.x { return a.p.x > b.p.x; }
    a.p.y > b.p.y
}

/// `_Push_heap_by_index` do MSVC (0x004D1A30), com topo 0.
fn push_heap_by_index(h: &mut [Node], mut hole: usize, val: Node) {
    while hole > 0 {
        let parent = (hole - 1) >> 1;
        if !after(&h[parent], &val) { break; }
        h[hole] = h[parent];
        hole = parent;
    }
    h[hole] = val;
}

/// `push_heap` (0x004CF0B0).
fn heap_push(h: &mut Vec<Node>, v: Node) {
    h.push(v);
    let n = h.len();
    if n >= 2 { let val = h[n - 1]; push_heap_by_index(h, n - 1, val); }
}

/// `pop_heap` + `pop_back` (0x004CE771 / `_Pop_heap_hole_by_index` 0x004D1AD0).
fn heap_pop(h: &mut Vec<Node>) -> Node {
    let top = h[0];
    let n = h.len();
    if n >= 2 {
        let val = h[n - 1];
        h[n - 1] = h[0];
        let bottom = n - 1;
        let mut hole = 0usize;
        let mut idx = 0usize;
        let max_non_leaf = (bottom - 1) >> 1;
        while idx < max_non_leaf {
            idx = 2 * idx + 2;
            if after(&h[idx], &h[idx - 1]) { idx -= 1; }
            h[hole] = h[idx];
            hole = idx;
        }
        if idx == max_non_leaf && bottom % 2 == 0 {
            h[hole] = h[bottom - 1];
            hole = bottom - 1;
        }
        push_heap_by_index(&mut h[..bottom], hole, val);
    }
    h.pop();
    top
}
