//! Etapa 10: GetReachable (Dijkstra 0x004CDE60), GetPath (A* 0x004CE630), IsTerrain, IsCrackable,
//! IsPod/IsItem/IsDangerous/IsSpawning e IsDeadly/WouldDie.
use itb_core::board_occupancy::{consts::*, DamageTraits, MovementTraits, OccupancyBoard, PawnState, PawnStatusBytes, Point};
use itb_core::damage::{DAMAGE_DEATH, DAMAGE_ZERO};

fn b8() -> OccupancyBoard { OccupancyBoard::new(8, 8).unwrap() }
fn pt(x: i32, y: i32) -> Point { Point::new(x, y) }
fn pawn(id: i32, team: i32, x: i32, y: i32, hp: i32) -> PawnState {
    PawnState { id, team, health: hp, max_health: hp, space: pt(x, y),
                status: PawnStatusBytes { powered: true, ..Default::default() }, ..Default::default() }
}

#[test]
fn reachable_orders_by_x_then_y_and_respects_range() {
    let mut b = b8();
    b.tile_mut(pt(2, 2)).unwrap().terrain = TERRAIN_MOUNTAIN;
    b.add_pawn(pawn(0, TEAM_PLAYER, 1, 1, 3)).unwrap();
    let r = b.get_reachable(pt(1, 1), 2, (TEAM_PLAYER << 4) | PATH_GROUND);
    assert_eq!(r, vec![pt(0, 0), pt(0, 1), pt(0, 2), pt(1, 0), pt(1, 2), pt(1, 3), pt(2, 0), pt(2, 1), pt(3, 1)]);
    // alcance 0: só a origem entra no mapa, e ela está ocupada (IsBlocked)
    assert!(b.get_reachable(pt(1, 1), 0, PATH_GROUND).is_empty());
    assert!(b.get_reachable(pt(-1, 0), 3, PATH_GROUND).is_empty());
}

#[test]
fn reachable_passes_through_allies_but_not_enemies() {
    let mut b = b8();
    b.add_pawn(pawn(0, TEAM_PLAYER, 0, 0, 3)).unwrap();
    b.add_pawn(pawn(1, TEAM_PLAYER, 1, 0, 3)).unwrap();
    b.add_pawn(pawn(2, TEAM_ENEMY, 0, 1, 3)).unwrap();
    // atravessa o aliado em (1,0), não pode parar nele; o inimigo em (0,1) fecha o outro lado
    let r = b.get_reachable(pt(0, 0), 2, (TEAM_PLAYER << 4) | PATH_GROUND);
    assert_eq!(r, vec![pt(1, 1), pt(2, 0)]);
    // sem os bits de time, o aliado também barra
    assert!(b.get_reachable(pt(0, 0), 2, PATH_GROUND).is_empty());
}

#[test]
fn jumper_filter_and_water() {
    let mut b = b8();
    b.tile_mut(pt(1, 0)).unwrap().terrain = TERRAIN_WATER;
    // o Dijkstra usa IsFree (0x0059F260): água bloqueia chão; a origem vazia entra (distância 0)
    assert_eq!(b.get_reachable(pt(0, 0), 1, PATH_GROUND), vec![pt(0, 0), pt(0, 1)]);
    // saltador passa pela água, mas o pós-filtro 0x00573280 remove a água
    assert_eq!(b.get_reachable(pt(0, 0), 1, PATH_JUMPER), vec![pt(0, 0), pt(0, 1)]);
    // voador pode pousar na água
    assert_eq!(b.get_reachable(pt(0, 0), 1, PATH_FLYER), vec![pt(0, 0), pt(0, 1), pt(1, 0)]);
}

#[test]
fn edge_walls_and_projectile_building_cost() {
    let mut b = b8();
    b.tile_mut(pt(0, 0)).unwrap().edge_block_0x2ac4 = [false, true, false, false]; // parede à direita
    assert!(b.is_edge_blocked(pt(0, 0), 1, PATH_GROUND));
    assert!(b.is_edge_blocked(pt(0, 0), 4, PATH_MASSIVE));
    assert!(!b.is_edge_blocked(pt(0, 0), 1, PATH_FLYER));
    assert!(!b.is_edge_blocked(pt(0, 0), 2, PATH_GROUND));
    assert_eq!(b.get_reachable(pt(0, 0), 1, PATH_GROUND), vec![pt(0, 0), pt(0, 1)]);
    b.tile_mut(pt(3, 3)).unwrap().terrain = TERRAIN_BUILDING;
    assert_eq!(b.move_cost(pt(3, 3), PATH_PROJECTILE), 1000);
    assert_eq!(b.move_cost(pt(3, 3), (1 << 4) | PATH_PROJECTILE), 1); // compara prof inteiro com 3
    assert_eq!(b.move_cost(pt(3, 2), PATH_PROJECTILE), 1);
}

#[test]
fn astar_tie_breaks_like_msvc_heap() {
    let b = b8();
    // empates (f, x, y): o menor x sai primeiro → desce antes de virar
    assert_eq!(b.get_path(pt(0, 0), pt(2, 2), PATH_GROUND), vec![pt(0, 0), pt(0, 1), pt(0, 2), pt(1, 2), pt(2, 2)]);
    assert_eq!(b.get_path(pt(1, 1), pt(4, 1), PATH_GROUND), vec![pt(1, 1), pt(2, 1), pt(3, 1), pt(4, 1)]);
    assert!(b.get_path(pt(1, 1), pt(1, 1), PATH_GROUND).is_empty());
    assert!(b.get_path(pt(1, 1), pt(9, 1), PATH_GROUND).is_empty());
}

#[test]
fn astar_goes_around_and_accepts_occupied_goal() {
    let mut b = b8();
    for y in 0..3 { b.tile_mut(pt(2, y)).unwrap().terrain = TERRAIN_MOUNTAIN; }
    b.add_pawn(pawn(1, TEAM_ENEMY, 4, 0, 2)).unwrap();
    let p = b.get_path(pt(0, 0), pt(4, 0), PATH_GROUND);
    assert_eq!(p.first(), Some(&pt(0, 0)));
    assert_eq!(p.last(), Some(&pt(4, 0))); // destino ocupado é aceito
    assert_eq!(p.len(), 11);
    for w in p.windows(2) {
        assert_eq!((w[0].x - w[1].x).abs() + (w[0].y - w[1].y).abs(), 1);
        assert!(b.is_free(w[1], PATH_GROUND) || w[1] == pt(4, 0));
    }
    // sem saída: lista vazia
    let mut c = b8();
    c.tile_mut(pt(1, 0)).unwrap().terrain = TERRAIN_MOUNTAIN;
    c.tile_mut(pt(0, 1)).unwrap().terrain = TERRAIN_MOUNTAIN;
    assert!(c.get_path(pt(0, 0), pt(5, 5), PATH_GROUND).is_empty());
}

#[test]
fn is_terrain_special_values() {
    let mut b = b8();
    let t = b.tile_mut(pt(1, 1)).unwrap();
    t.terrain = TERRAIN_WATER;
    t.lava_0x2ac8 = true;
    assert!(b.is_terrain(pt(1, 1), TERRAIN_WATER) && b.is_terrain(pt(1, 1), TERRAIN_LAVA));
    let t = b.tile_mut(pt(1, 1)).unwrap();
    t.flag_0x23ac = true;
    t.counter_0x23cc = 3;
    assert!(!b.is_terrain(pt(1, 1), TERRAIN_WATER) && !b.is_terrain(pt(1, 1), TERRAIN_LAVA));
    let t = b.tile_mut(pt(2, 2)).unwrap();
    t.terrain = TERRAIN_ICE;
    t.health = 1;
    t.cracked = 1;
    assert!(b.is_terrain(pt(2, 2), 18) && b.is_terrain(pt(2, 2), 16) && b.is_terrain(pt(2, 2), TERRAIN_ICE));
    assert!(b.is_terrain(pt(-3, 0), TERRAIN_ROAD)); // sentinela
}

#[test]
fn crackable_pod_item_danger_spawn() {
    let mut b = b8();
    assert!(b.is_crackable(pt(0, 0)));
    b.tile_mut(pt(1, 0)).unwrap().terrain = TERRAIN_BUILDING;
    b.tile_mut(pt(2, 0)).unwrap().terrain = TERRAIN_WATER;
    b.tile_mut(pt(3, 0)).unwrap().custom_tile = "conveyor1.png".into();
    b.tile_mut(pt(4, 0)).unwrap().str_0x26f8 = "supervolcano".into();
    let mut rocket = pawn(3, TEAM_PLAYER, 5, 0, 2);
    rocket.type_name = "SatelliteRocket".into();
    b.add_pawn(rocket).unwrap();
    for x in 1..=5 { assert!(!b.is_crackable(pt(x, 0)), "x={x}"); }
    b.pod_space = pt(3, 3);
    assert!(!b.is_pod(pt(3, 3)));
    b.tile_mut(pt(3, 3)).unwrap().pod_0x2ae4 = 1;
    assert!(b.is_pod(pt(3, 3)) && !b.is_pod(pt(3, 4)));
    b.tile_mut(pt(6, 6)).unwrap().item = "Item_Mine".into();
    assert!(b.is_item(pt(6, 6)) && !b.is_item(pt(6, 5)));
    b.danger_list_b.push(pt(7, 7));
    b.tile_mut(pt(0, 7)).unwrap().danger_0x2af0 = true;
    assert!(b.is_dangerous(pt(7, 7)) && b.is_dangerous(pt(0, 7)) && !b.is_dangerous(pt(1, 7)));
    b.spawn_list.push(pt(5, 5));
    assert!(b.is_spawning(pt(5, 5)) && !b.is_spawning(pt(5, 4)));
}

fn deadly(b: &OccupancyBoard, d: i32) -> bool { b.is_deadly(pt(2, 2), d, None) }

#[test]
fn would_die_shield_frozen_armor_acid() {
    let mut b = b8();
    let i = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 2, 3)).unwrap();
    assert!(!deadly(&b, 2) && deadly(&b, 3) && !deadly(&b, DAMAGE_ZERO) && deadly(&b, DAMAGE_DEATH));
    assert!(!b.is_deadly(pt(3, 3), 9, None)); // casa vazia
    b.pawns[i].damage.query_0x63c100 = true; // blindagem: −1
    assert!(!deadly(&b, 3) && deadly(&b, 4));
    b.pawns[i].status.acid = true; // ácido dobra depois da blindagem
    assert!(deadly(&b, 3));
    b.pawns[i].status.acid = false;
    b.pawns[i].damage.query_0x63c100 = false;
    b.pawns[i].damage.shield_0x8d4 = true;
    assert!(!deadly(&b, 99) && deadly(&b, DAMAGE_DEATH));
    assert!(b.pawn_would_die(i, &[5, 3])); // escudo consome o primeiro
    b.pawns[i].damage.shield_0x8d4 = false;
    b.pawns[i].status.frozen = true;
    assert!(!deadly(&b, 99) && b.pawn_would_die(i, &[1, 3]) && !b.pawn_would_die(i, &[DAMAGE_ZERO, 2, 2]));
    b.pawns[i].status.frozen = false;
    assert!(!b.pawn_would_die(i, &[2, -1, 1]) && b.pawn_would_die(i, &[2, -1, 2])); // cura no meio
    b.pawns[i].health = 0;
    assert!(!deadly(&b, 5)); // já morto: falso
}

#[test]
fn would_die_cracked_ground_and_damaged_ice() {
    let mut b = b8();
    let i = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 2, 5)).unwrap();
    b.tile_mut(pt(2, 2)).unwrap().cracked = 1;
    assert!(deadly(&b, 1) && !deadly(&b, 0) && !deadly(&b, DAMAGE_ZERO));
    b.pawns[i].movement = MovementTraits { flying: true, ..Default::default() };
    assert!(!deadly(&b, 1));
    let t = b.tile_mut(pt(2, 2)).unwrap();
    t.cracked = 0;
    t.terrain = TERRAIN_ICE;
    t.health = 1;
    b.pawns[i].movement.flying = false;
    assert!(deadly(&b, 1));
    b.pawns[i].damage.byte_0x99d = 1;
    assert!(!deadly(&b, 1));
}

#[test]
fn boosted_attacker_adds_one() {
    let mut b = b8();
    b.add_pawn(pawn(1, TEAM_ENEMY, 2, 2, 3)).unwrap();
    let mut a = pawn(0, TEAM_PLAYER, 0, 0, 3);
    a.damage = DamageTraits { boosted_0x8d5: true, ..Default::default() };
    let a = b.add_pawn(a).unwrap();
    assert!(b.is_deadly(pt(2, 2), 2, Some(a)) && !b.is_deadly(pt(2, 2), 2, None));
    assert_eq!(PawnState::boost_damage(DAMAGE_ZERO), DAMAGE_ZERO);
    assert_eq!(PawnState::boost_damage(DAMAGE_DEATH), DAMAGE_DEATH);
    assert_eq!((PawnState::boost_damage(-1), PawnState::boost_damage(-9), PawnState::boost_damage(-10)), (-2, -10, -10));
    // "Arrogant_Boost" só com vida cheia
    b.pawns[a].damage = DamageTraits { query_arrogant_boost: true, ..Default::default() };
    assert!(b.pawns[a].is_boosted());
    b.pawns[a].health = 2;
    assert!(!b.pawns[a].is_boosted());
}

#[test]
fn move_speed_and_traits() {
    let mut p = pawn(1, TEAM_PLAYER, 0, 0, 3);
    p.base_move_0x63a8f0 = 3;
    assert_eq!(p.get_move_speed(), 3);
    p.move_speed_override_0xa64 = 5;
    assert_eq!(p.get_move_speed(), 5);
    p.grappled_0xf81 = true;
    assert!(p.is_grappled());
    assert_eq!(p.get_move_speed(), 0);
    p.grappled_0xf81 = false;
    p.status.powered = false;
    assert_eq!(p.get_move_speed(), 0);
    p.movement = MovementTraits { jumper: true, teleporter: true, burrows: true, ..Default::default() };
    assert!(p.is_jumper() && p.is_teleporter() && p.is_burrower());
    p.health = 0;
    assert!(!p.is_jumper() && !p.is_teleporter() && p.is_burrower() && !p.is_grappled());
}
