//! Etapa 13: casa disponível, sorteio e fila de surgimento, como `Board::SpawnPawn` (1.2.93: 0x568CA0).
use itb_core::apply::BoardEvent;
use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, Point};
use itb_core::game::GameState;
use itb_core::pawn_factory::PawnType;
use itb_core::rng::MsvcRand;

fn pt(x: i32, y: i32) -> Point { Point::new(x, y) }
fn b8() -> OccupancyBoard { OccupancyBoard::new(8, 8).unwrap() }

fn pawn(name: &str, id: i32, team: i32) -> PawnState {
    PawnState::from_type(&PawnType { name: name.into(), health: 2, move_speed: 3, pushable: true, ..Default::default() }, id, team)
}

fn set(b: &mut OccupancyBoard, p: Point, terrain: i32) { b.set_terrain(p, terrain); }

#[test]
fn enemy_rules_of_is_available() {
    let mut b = b8();
    let p = pt(5, 3);
    assert!(b.is_available(TEAM_ENEMY, PATH_GROUND, p));
    for t in [TERRAIN_ICE, TERRAIN_WATER, TERRAIN_FOREST, TERRAIN_MOUNTAIN, TERRAIN_BUILDING] {
        let mut c = b.clone();
        set(&mut c, p, t);
        assert!(!c.is_available(TEAM_ENEMY, PATH_GROUND, p), "terreno {t}");
    }
    // time 9: a floresta passa a servir, o resto não
    let mut f = b.clone();
    set(&mut f, p, TERRAIN_FOREST);
    assert!(f.is_available(9, PATH_GROUND, p));
    // ácido, fogo: fogo não impede inimigo, ácido impede
    b.tile_mut(p).unwrap().acid = 1;
    assert!(!b.is_available(TEAM_ENEMY, PATH_GROUND, p));
    b.tile_mut(p).unwrap().acid = 0;
    b.tile_mut(p).unwrap().fire = 1;
    assert!(b.is_available(TEAM_ENEMY, PATH_GROUND, p));
    assert!(!b.is_available(TEAM_PLAYER, PATH_GROUND, p));
}

#[test]
fn blocks_items_pods_and_queued_spaces_are_not_available() {
    let mut b = b8();
    let p = pt(5, 3);
    b.set_block_spawn(p, 1);
    assert!(!b.is_available(TEAM_ENEMY, PATH_GROUND, p));
    b.clear_block_spawns();
    assert!(b.is_available(TEAM_ENEMY, PATH_GROUND, p));
    b.set_block_spawn(p, 2);
    b.clear_block_spawns();
    assert!(!b.is_available(TEAM_ENEMY, PATH_GROUND, p), "o tipo 2 sobrevive a ClearBlockSpawns");
    let q = pt(5, 4);
    b.tile_mut(q).unwrap().item = "Item_Mine".into();
    assert!(!b.is_available(TEAM_ENEMY, PATH_GROUND, q));
    let r = pt(5, 5);
    b.tile_mut(r).unwrap().pod_0x2ae4 = 1;
    assert!(!b.is_available(TEAM_ENEMY, PATH_GROUND, r));
    let s = pt(6, 3);
    b.spawn_list.push(s);
    assert!(!b.is_available(TEAM_ENEMY, PATH_GROUND, s));
}

#[test]
fn enemy_space_is_drawn_from_the_zone_with_rand() {
    let mut b = b8();
    let mut rng = MsvcRand::new(1);
    // srand(1): o primeiro rand() é 41; 41 % 12 = 5, a sexta casa da zona "enemy"
    assert_eq!(b.get_available_space(&pawn("X", 1, TEAM_ENEMY), &mut rng), Some(pt(6, 3)));
}

#[test]
fn blocked_zone_falls_back_to_rightmost_column_of_the_board() {
    let mut b = b8();
    for x in 0..8 {
        for y in 0..8 {
            if x > 3 { set(&mut b, pt(x, y), TERRAIN_MOUNTAIN); }
        }
    }
    let mut rng = MsvcRand::new(1);
    let p = b.get_available_space(&pawn("X", 1, TEAM_ENEMY), &mut rng).unwrap();
    assert_eq!(p.x, 3);
}

#[test]
fn drawn_forest_is_cleared_only_through_the_relaxed_zone() {
    let mut b = b8();
    for p in b.get_zone("enemy") { set(&mut b, p, TERRAIN_FOREST); }
    let mut rng = MsvcRand::new(1);
    let p = b.get_available_space(&pawn("X", 1, TEAM_ENEMY), &mut rng).unwrap();
    assert_eq!(p, pt(6, 3));
    assert_eq!(b.tile(p).terrain, TERRAIN_ROAD);
}

#[test]
fn no_space_at_all_gives_none() {
    let mut b = b8();
    for x in 0..8 {
        for y in 0..8 { set(&mut b, pt(x, y), TERRAIN_MOUNTAIN); }
    }
    assert_eq!(b.get_available_space(&pawn("X", 1, TEAM_ENEMY), &mut MsvcRand::new(1)), None);
}

#[test]
fn player_uses_the_drop_zone() {
    let mut b = b8();
    let z = b.get_drop_zone();
    assert_eq!(z.len(), 18);
    assert_eq!((z[0], z[17]), (pt(1, 1), pt(3, 6)));
    let mut m = b.clone();
    m.mission.zones.insert("deployment".into(), vec![pt(2, 2), pt(2, 3)]);
    assert_eq!(m.get_drop_zone(), vec![pt(2, 2), pt(2, 3)]);
    let mut rng = MsvcRand::new(1);
    let p = b.get_available_space(&pawn("M", 1, TEAM_PLAYER), &mut rng).unwrap();
    assert!((1..=3).contains(&p.x) && (1..=6).contains(&p.y));
}

#[test]
fn enemy_goes_to_the_queue_and_emerges_later() {
    let mut b = b8();
    let mut rng = MsvcRand::new(1);
    let a = b.spawn_pawn(pawn("A", 5, TEAM_ENEMY), Point::INVALID, &mut rng).unwrap();
    let c = b.spawn_pawn(pawn("B", 6, TEAM_ENEMY), pt(5, 2), &mut rng).unwrap();
    assert_eq!(b.spawn_list.len(), 2);
    assert!(b.is_spawning(pt(5, 2)));
    assert!(b.pawn_list.is_empty());
    assert_eq!(b.get_pawn_by_id(5), None, "na fila, a unidade ainda não está na lista do tabuleiro");
    assert_eq!(b.get_pawn_at(pt(5, 2)), None);
    b.spawn_queued();
    assert!(b.spawn_queue.is_empty() && b.spawn_list.is_empty());
    assert_eq!(b.pawn_list, vec![a, c]);
    assert_eq!(b.get_pawn_at(pt(5, 2)), Some(c));
    assert!(b.pawns[a].active_0x91c);
}

#[test]
fn queued_spaces_are_not_drawn_twice() {
    let mut b = b8();
    let mut rng = MsvcRand::new(7);
    for i in 0..12 { b.spawn_pawn(pawn("A", i, TEAM_ENEMY), Point::INVALID, &mut rng).unwrap(); }
    let mut seen = b.spawn_list.clone();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 12);
    assert!(b.spawn_list.iter().all(|p| (5..=7).contains(&p.x) && (2..=5).contains(&p.y)));
}

#[test]
fn blocked_emergence_stays_queued_and_hurts_the_blocker() {
    let mut b = b8();
    let mut rng = MsvcRand::new(1);
    let q = b.spawn_pawn(pawn("A", 5, TEAM_ENEMY), pt(5, 2), &mut rng).unwrap();
    let blocker = b.board_add_pawn(pawn("Mech", 1, TEAM_PLAYER), pt(5, 2)).unwrap();
    b.spawn_queued();
    assert_eq!(b.spawn_queue, vec![q]);
    assert_eq!(b.pawns[blocker].health, 1);
    assert!(b.events.contains(&BoardEvent::EmergeBlocked(pt(5, 2))));
    // Passive_Burrows protege mech
    let mut c = b8();
    let mut m = pawn("Mech", 1, TEAM_PLAYER);
    m.mech = true;
    c.spawn_pawn(pawn("A", 5, TEAM_ENEMY), pt(5, 2), &mut rng).unwrap();
    let i = c.board_add_pawn(m, pt(5, 2)).unwrap();
    c.passive_skills.push("Passive_Burrows".into());
    c.spawn_queued();
    assert_eq!(c.pawns[i].health, 2);
}

#[test]
fn player_enters_at_once_and_hole_or_water_enter_with_state_1() {
    let mut b = b8();
    let mut rng = MsvcRand::new(1);
    let m = b.spawn_pawn(pawn("Mech", 1, TEAM_PLAYER), pt(2, 2), &mut rng).unwrap();
    assert_eq!(b.get_pawn_at(pt(2, 2)), Some(m));
    set(&mut b, pt(3, 3), TERRAIN_HOLE);
    let h = b.spawn_pawn(pawn("E", 2, TEAM_ENEMY), pt(3, 3), &mut rng).unwrap();
    assert_eq!(b.pawns[h].state_0x9e0, 1);
    assert!(b.spawn_queue.is_empty());
}

#[test]
fn get_pawns_filters_by_team_and_skips_the_dead() {
    let mut b = b8();
    let a = b.board_add_pawn(pawn("E1", 1, TEAM_ENEMY), pt(1, 1)).unwrap();
    let c = b.board_add_pawn(pawn("M", 2, TEAM_PLAYER), pt(2, 1)).unwrap();
    let d = b.board_add_pawn(pawn("E2", 3, TEAM_ENEMY), pt(3, 1)).unwrap();
    assert_eq!(b.get_pawns(TEAM_ENEMY), vec![a, d]);
    assert_eq!(b.get_pawns(TEAM_PLAYER), vec![c]);
    b.pawns[a].health = 0;
    assert_eq!(b.get_pawns(TEAM_ENEMY), vec![d]);
    b.pawns[a].mech = true;
    assert_eq!(b.get_pawns(TEAM_ENEMY), vec![a, d]);
}

#[test]
fn set_active_keeps_idle_pawns_inactive() {
    let mut p = PawnState::from_type(&PawnType { name: "Parede".into(), health: 1, ..Default::default() }, 1, TEAM_NONE);
    p.set_active(true);
    assert!(!p.active_0x91c);
    p.mech = true;
    p.set_active(true);
    assert!(p.active_0x91c);
    p.status.powered = false;
    p.set_active(false);
    assert!(p.active_0x91c, "sem energia não muda");
}

#[test]
fn game_accessors_follow_the_exe() {
    let mut g = GameState::default();
    assert_eq!(g.get_sector(), 1);
    g.sector_0x5238 = 2;
    assert_eq!(g.get_sector(), 3);
    g.sector_0x5238 = 9;
    assert_eq!(g.get_sector(), 4);
    assert_eq!((g.get_team_turn(), g.get_turn_count()), (TEAM_NONE, 0));
    g.turn = Some((3, TEAM_ENEMY));
    assert_eq!((g.get_team_turn(), g.get_turn_count()), (TEAM_ENEMY, 3));
    g.profile_islands_0x30 = 2;
    assert!(g.is_island_unlocked(2) && !g.is_island_unlocked(3));
    {
        let mut gr = g.grid.borrow_mut();
        gr.value = 5;
        gr.max = 7;
    }
    g.set_power_max(4);
    assert_eq!(g.power(), (4, 4));
    g.set_power_max(9);
    assert_eq!(g.power(), (4, 9));
}
