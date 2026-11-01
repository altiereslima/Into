//! Etapa 13: mapa aplicado ao tabuleiro, como `Board::LoadScript` (1.2.93: 0x562220).
use itb_core::board_occupancy::{consts::*, OccupancyBoard, Point};
use itb_core::board_occupancy::PawnState;
use itb_core::map::{MapData, MapTile};
use itb_core::pawn_factory::PawnType;

fn pt(x: i32, y: i32) -> Point { Point::new(x, y) }

fn map(tiles: Vec<MapTile>) -> OccupancyBoard {
    OccupancyBoard::from_map(&MapData { name: "teste".into(), size: pt(8, 8), tiles, ..Default::default() }).unwrap()
}

fn tile(x: i32, y: i32, terrain: i32) -> MapTile { MapTile { terrain, ..MapTile::new(pt(x, y)) } }

#[test]
fn unlisted_tiles_keep_the_boardspace_constructor_values() {
    let b = map(vec![]);
    let t = b.tile(pt(3, 3));
    assert_eq!((t.terrain, t.health, t.max_health, t.byte_0x2710), (TERRAIN_ROAD, 4, 4, true));
}

#[test]
fn listed_tile_uses_loadscript_defaults() {
    let b = map(vec![tile(1, 1, TERRAIN_ROAD)]);
    let t = b.tile(pt(1, 1));
    // health_max tem padrão 2; health_min, o próprio máximo; populated, 0
    assert_eq!((t.health, t.max_health, t.byte_0x2710), (2, 2, false));
}

#[test]
fn building_health_and_population() {
    let b = map(vec![MapTile { populated: 1, health_max: 4, health_min: Some(3), ..tile(2, 2, TERRAIN_BUILDING) }]);
    let t = b.tile(pt(2, 2));
    assert_eq!((t.terrain, t.health, t.max_health, t.byte_0x2710), (TERRAIN_BUILDING, 3, 4, true));
    assert!(b.is_building(pt(2, 2)));
}

#[test]
fn current_health_never_exceeds_the_maximum() {
    let b = map(vec![MapTile { health_max: 2, health_min: Some(9), ..tile(2, 2, TERRAIN_BUILDING) }]);
    assert_eq!(b.tile(pt(2, 2)).health, 2);
}

#[test]
fn terrain_adjusts_health_like_the_exe() {
    let b = map(vec![
        MapTile { health_max: 4, ..tile(0, 0, TERRAIN_RUBBLE) },
        MapTile { health_max: 4, ..tile(0, 1, TERRAIN_MOUNTAIN) },
        MapTile { health_max: 4, health_min: Some(1), ..tile(0, 2, TERRAIN_MOUNTAIN) },
    ]);
    assert_eq!((b.tile(pt(0, 0)).health, b.tile(pt(0, 0)).max_health), (0, 4));
    assert_eq!((b.tile(pt(0, 1)).health, b.tile(pt(0, 1)).max_health), (2, 2));
    assert_eq!((b.tile(pt(0, 2)).health, b.tile(pt(0, 2)).max_health), (1, 2));
}

#[test]
fn lava_is_water_marked_as_lava_and_ignores_acid() {
    let b = map(vec![MapTile { lava: true, poison: 1, fire: 1, ..tile(4, 4, TERRAIN_ROAD) }]);
    let t = b.tile(pt(4, 4));
    assert_eq!((t.terrain, t.lava_0x2ac8, t.fire), (TERRAIN_WATER, true, 0));
    // o ácido é aplicado antes de `lava`, então ele fica; SetAcid só recusa se a casa já for lava
    assert_eq!(t.acid, 1);
    assert!(b.is_terrain(pt(4, 4), TERRAIN_LAVA));
}

#[test]
fn acid_clears_fire_from_the_map() {
    let b = map(vec![MapTile { fire: 1, poison: 1, ..tile(5, 5, TERRAIN_ROAD) }]);
    assert_eq!((b.tile(pt(5, 5)).fire, b.tile(pt(5, 5)).acid), (0, 1));
}

#[test]
fn pawn_of_the_map_waits_for_the_factory() {
    let b = map(vec![MapTile { pawn: "Wall".into(), ..tile(6, 6, TERRAIN_ROAD) }]);
    assert_eq!(b.mission.pending_pawns, vec![(pt(6, 6), "Wall".to_string())]);
    assert!(b.pawns.is_empty());
}

#[test]
fn mission_data_is_kept() {
    let mut m = MapData { name: "x".into(), size: pt(8, 8), enemy_kills: 3, tags: vec!["generic".into()], ..Default::default() };
    m.zones.insert("dam".into(), vec![pt(1, 1), pt(1, 2)]);
    let b = OccupancyBoard::from_map(&m).unwrap();
    assert_eq!((b.mission.name.as_str(), b.mission.enemy_kills), ("x", 3));
    assert_eq!(b.mission.zones["dam"], vec![pt(1, 1), pt(1, 2)]);
    assert_eq!(b.mission.tags, vec!["generic".to_string()]);
}

#[test]
fn missing_enemy_zone_is_computed_from_the_board_size() {
    let mut m = MapData { size: pt(8, 8), ..Default::default() };
    let b = OccupancyBoard::from_map(&m).unwrap();
    assert!(b.get_zone("dam").is_empty());
    let z = b.get_zone("enemy");
    assert_eq!(z.len(), 12);
    assert_eq!((z[0], z[3], z[4], z[11]), (pt(5, 2), pt(5, 5), pt(6, 2), pt(7, 5)));
    // zona declarada no mapa vence o cálculo
    m.zones.insert("enemy".into(), vec![pt(1, 1)]);
    assert_eq!(OccupancyBoard::from_map(&m).unwrap().get_zone("enemy"), vec![pt(1, 1)]);
}

fn typed(name: &str, id: i32, team: i32) -> PawnState {
    PawnState::from_type(&PawnType { name: name.into(), health: 2, move_speed: 3, pushable: true, ..Default::default() }, id, team)
}

#[test]
fn pawn_from_type_matches_oninit_and_the_constructor() {
    let t = PawnType { name: "X".into(), health: 5, move_speed: 2, corpse: true, flying: true, teleporter: true, jumper: true, burrows: true,
                       massive: true, pushable: false, ignore_flip: true, neutral: true, default_faction: 1, non_grid: true, armor: true,
                       ignore_fire: true, skills: vec!["A".into()] };
    let p = PawnState::from_type(&t, 9, TEAM_ENEMY);
    assert_eq!((p.id, p.team, p.health, p.max_health, p.type_name.as_str()), (9, TEAM_ENEMY, 5, 5, "X"));
    assert_eq!(p.skills, vec!["Move".to_string(), "A".to_string()]);
    assert_eq!((p.state_0x9e0, p.status.powered, p.status.byte_0x91e, p.status.ignore_flip), (5, true, 0, true));
    assert!(p.flag_0xf80 && p.movement.flying && p.movement.teleporter && p.movement.jumper && p.movement.burrows && p.movement.massive);
    assert_eq!((p.damage.byte_0x99d, p.field_0x10bc, p.base_move_0x63a8f0), (1, 1, 2));
    assert!(p.neutral && p.non_grid_structure && p.damage.query_0x63c100 && p.damage.query_0x638bd0);
}

#[test]
fn add_pawn_rebuilds_the_list_in_three_passes() {
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    let mut neutral = typed("Civil", 1, TEAM_PLAYER);
    neutral.neutral = true;
    let enemy = typed("Inseto", 2, TEAM_ENEMY);
    let mech = typed("Mech", 3, TEAM_PLAYER);
    let mut filler = typed("Filler_Pawn", 4, TEAM_PLAYER);
    filler.neutral = true;
    let mut bot = typed("Bot", 5, TEAM_PLAYER);
    bot.neutral = true;
    bot.field_0x10bc = 1;
    let enemy2 = typed("Inseto2", 6, TEAM_ENEMY);
    for (i, p) in [neutral, enemy, mech, filler, bot, enemy2].into_iter().enumerate() {
        b.board_add_pawn(p, pt(i as i32, 0)).unwrap();
    }
    let order: Vec<i32> = b.pawn_list.iter().map(|&i| b.pawns[i].id).collect();
    // 1ª passada: mech, Filler_Pawn e bot; 2ª: inimigos; 3ª: neutro do jogador
    assert_eq!(order, vec![3, 4, 5, 2, 6, 1]);
    // índices da arena continuam estáveis
    assert_eq!(b.pawns[0].id, 1);
}

#[test]
fn pawn_without_weapons_or_movement_is_not_active() {
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    let still = PawnState::from_type(&PawnType { name: "Parede".into(), health: 1, ..Default::default() }, 1, TEAM_NONE);
    let i = b.board_add_pawn(still, pt(0, 0)).unwrap();
    assert!(!b.pawns[i].active_0x91c);
    let i = b.board_add_pawn(typed("Andador", 2, TEAM_ENEMY), pt(1, 0)).unwrap();
    assert!(b.pawns[i].active_0x91c);
    let armed = PawnState::from_type(&PawnType { name: "Torre".into(), health: 1, skills: vec!["Tiro".into()], ..Default::default() }, 3, TEAM_ENEMY);
    let i = b.board_add_pawn(armed, pt(2, 0)).unwrap();
    assert!(b.pawns[i].active_0x91c);
}

#[test]
fn add_pawn_outside_the_board_is_an_error() {
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    assert!(b.board_add_pawn(typed("X", 1, TEAM_ENEMY), pt(-1, -1)).is_err());
}

#[test]
fn invalid_dimensions_are_rejected() {
    assert!(OccupancyBoard::from_map(&MapData { size: pt(0, 8), ..Default::default() }).is_err());
}
