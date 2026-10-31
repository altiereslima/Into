//! Etapa 09: IsPassable (0x0059F260), IsFree (0x00561300), GetSimpleReachable (0x004CEC60).
use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, Point, Tile};

fn b8() -> OccupancyBoard { OccupancyBoard::new(8, 8).unwrap() }
fn pawn(id: i32, team: i32, x: i32, y: i32) -> PawnState {
    PawnState { id, team, health: 2, max_health: 2, space: Point::new(x, y), ..Default::default() }
}
fn pts(v: &[(i32, i32)]) -> Vec<Point> { v.iter().map(|&(x, y)| Point::new(x, y)).collect() }

#[test]
fn passable_by_profile() {
    let b = b8();
    let t = |terrain| Tile { terrain, ..Default::default() };
    for prof in [PATH_FLYER, PATH_BURROWER, 5, 6] {
        assert!(b.tile_is_passable(&t(TERRAIN_MOUNTAIN), prof));
    }
    assert!(!b.tile_is_passable(&t(TERRAIN_HOLE), PATH_ROADRUNNER));
    assert!(b.tile_is_passable(&t(TERRAIN_WATER), PATH_ROADRUNNER));
    assert!(!b.tile_is_passable(&t(TERRAIN_BUILDING), PATH_PROJECTILE));
    assert!(b.tile_is_passable(&t(TERRAIN_WATER), PATH_PROJECTILE));
    assert!(!b.tile_is_passable(&t(TERRAIN_WATER), PATH_GROUND));
    assert!(b.tile_is_passable(&t(TERRAIN_WATER), PATH_MASSIVE));
    let ice_exception = Tile { terrain: TERRAIN_WATER, flag_0x23ac: true, counter_0x23cc: 3, ..Default::default() };
    assert!(b.tile_is_passable(&ice_exception, PATH_GROUND));
    assert!(!b.tile_is_passable(&t(TERRAIN_HOLE), PATH_GROUND));
    assert!(b.tile_is_passable(&t(TERRAIN_FOREST), PATH_GROUND));
}

#[test]
fn passable_with_pawns_and_team_bits() {
    let mut b = b8();
    b.add_pawn(pawn(1, TEAM_PLAYER, 2, 2)).unwrap();
    let tile = b.tile(Point::new(2, 2)).clone();
    assert!(!b.tile_is_passable(&tile, PATH_PROJECTILE));
    assert!(!b.tile_is_passable(&tile, PATH_GROUND)); // time 0 nos bits altos ≠ 1
    assert!(b.tile_is_passable(&tile, PATH_GROUND | (TEAM_PLAYER << 4))); // aliados atravessam
    b.pawns[0].non_grid_structure = true;
    assert!(!b.tile_is_passable(&tile, PATH_GROUND | (TEAM_PLAYER << 4)));
    assert!(!b.is_free(Point::new(8, 0), PATH_FLYER)); // fora do tabuleiro
}

#[test]
fn simple_reachable_open_board_matches_ray_order() {
    let b = b8();
    let got = b.get_simple_reachable(Point::new(3, 4), 8, false);
    // UP, RIGHT, DOWN, LEFT, cada um até a borda
    assert_eq!(got, pts(&[(3, 3), (3, 2), (3, 1), (3, 0), (4, 4), (5, 4), (6, 4), (7, 4), (3, 5), (3, 6), (3, 7), (2, 4), (1, 4), (0, 4)]));
    // alcance 1: só vizinhas
    assert_eq!(b.get_simple_reachable(Point::new(3, 4), 1, false), pts(&[(3, 3), (4, 4), (3, 5), (2, 4)]));
    // canto do tabuleiro
    assert_eq!(b.get_simple_reachable(Point::new(0, 0), 2, false), pts(&[(1, 0), (2, 0), (0, 1), (0, 2)]));
}

#[test]
fn simple_reachable_stops_at_and_includes_obstacle() {
    let mut b = b8();
    b.add_pawn(pawn(9, TEAM_ENEMY, 3, 1)).unwrap();
    b.tile_mut(Point::new(5, 4)).unwrap().terrain = TERRAIN_MOUNTAIN;
    let got = b.get_simple_reachable(Point::new(3, 4), 8, false);
    assert_eq!(&got[..3], &pts(&[(3, 3), (3, 2), (3, 1)])[..]); // inimigo incluído, para ali
    assert_eq!(&got[3..5], &pts(&[(4, 4), (5, 4)])[..]); // montanha incluída
    // obstáculo além do alcance não entra
    let short = b.get_simple_reachable(Point::new(3, 4), 2, false);
    assert!(!short.contains(&Point::new(3, 1)));
    assert!(short.contains(&Point::new(5, 4)));
    // obstáculo colado entra (passos = 0 < alcance)
    let mut c = b8();
    c.tile_mut(Point::new(3, 3)).unwrap().terrain = TERRAIN_BUILDING;
    assert_eq!(c.get_simple_reachable(Point::new(3, 4), 1, false)[0], Point::new(3, 3));
}

#[test]
fn simple_reachable_corners_quirks() {
    let b = b8();
    // tabuleiro vazio: cada casa do raio acrescenta as duas vizinhas perpendiculares, com repetição
    let got = b.get_simple_reachable(Point::new(3, 4), 1, true);
    assert_eq!(got, pts(&[(3, 3), (4, 3), (2, 3), (4, 4), (4, 5), (4, 3), (3, 5), (4, 5), (2, 5), (2, 4), (2, 5), (2, 3)]));
}

#[test]
fn path_prof_priority_and_team_bits() {
    use itb_core::board_occupancy::MovementTraits;
    let mut p = pawn(1, TEAM_PLAYER, 0, 0);
    assert_eq!(p.get_path_prof(), (1 << 4) | PATH_GROUND);
    p.movement = MovementTraits { massive: true, flying: true, ..Default::default() };
    assert_eq!(p.get_path_prof() & 0xF, PATH_FLYER); // voar vence massivo
    p.movement.burrows = true;
    assert_eq!(p.get_path_prof() & 0xF, PATH_BURROWER);
    p.movement.jumper = true;
    assert_eq!(p.get_path_prof() & 0xF, 6);
    p.movement.teleporter = true;
    assert_eq!(p.get_path_prof() & 0xF, 5);
    p.health = 0; // teleporte só vale se não estiver morto
    assert_eq!(p.get_path_prof() & 0xF, 6);
    p.team = TEAM_ENEMY;
    assert_eq!(p.get_path_prof() >> 4, TEAM_ENEMY);
}

#[test]
fn simple_path_l_shape_and_blocking() {
    let mut b = b8();
    // reto
    assert_eq!(b.get_simple_path(Point::new(3, 4), Point::new(3, 1)), pts(&[(3, 4), (3, 3), (3, 2), (3, 1)]));
    // em L: primeiro UP (aproxima), depois vira para RIGHT
    assert_eq!(b.get_simple_path(Point::new(1, 4), Point::new(3, 2)), pts(&[(1, 4), (1, 3), (1, 2), (2, 2), (3, 2)]));
    // alvo ocupado é aceito; caminho ocupado no meio força outra direção inicial
    b.add_pawn(pawn(1, TEAM_ENEMY, 3, 2)).unwrap();
    assert_eq!(b.get_simple_path(Point::new(3, 4), Point::new(3, 2)), pts(&[(3, 4), (3, 3), (3, 2)]));
    b.tile_mut(Point::new(1, 3)).unwrap().terrain = TERRAIN_MOUNTAIN;
    assert_eq!(b.get_simple_path(Point::new(1, 4), Point::new(3, 2)), pts(&[(1, 4), (2, 4), (3, 4), (3, 3), (3, 2)]));
    // impossível / iguais / inválido
    assert!(b.get_simple_path(Point::new(1, 1), Point::new(1, 1)).is_empty());
    assert!(b.get_simple_path(Point::new(1, 1), Point::new(9, 1)).is_empty());
    let mut c = b8();
    for (x, y) in [(2, 1), (1, 2), (0, 1), (1, 0)] { c.tile_mut(Point::new(x, y)).unwrap().terrain = TERRAIN_MOUNTAIN; }
    assert!(c.get_simple_path(Point::new(1, 1), Point::new(5, 5)).is_empty());
}
