//! Etapa 11: aplicação de dano e empurrão (Tile::ApplyDamage 0x005ABA90, Pawn::ApplyDamage 0x006295B0,
//! Pawn::Push/ResolvePush 0x0062CC00/0x0062CDD0, Health::Add 0x004DFFC0 e modificadores de casa).
use itb_core::apply::{health_add, BoardEvent, Hit, DIR_FLIP};
use itb_core::board_occupancy::{consts::*, DamageTraits, MovementTraits, OccupancyBoard, PawnState, PawnStatusBytes, Point};
use itb_core::damage::{DAMAGE_DEATH, DAMAGE_ZERO};

fn pt(x: i32, y: i32) -> Point { Point::new(x, y) }
fn b8() -> OccupancyBoard { OccupancyBoard::new(8, 8).unwrap() }
/// Unidade empurrável (byte +0x91E = 1; com 0 o EXE a trata como "estável").
fn pawn(id: i32, team: i32, x: i32, y: i32, hp: i32) -> PawnState {
    PawnState { id, team, health: hp, max_health: hp, space: pt(x, y),
                status: PawnStatusBytes { powered: true, byte_0x91e: 1, ..Default::default() }, ..Default::default() }
}
fn hit(x: i32, y: i32, dmg: i32) -> Hit { Hit::new(pt(x, y), dmg) }
fn push(x: i32, y: i32, dmg: i32, dir: i32) -> Hit { let mut h = hit(x, y, dmg); h.push = dir; h }
fn building(b: &mut OccupancyBoard, x: i32, y: i32, hp: i32) {
    let t = b.tile_mut(pt(x, y)).unwrap();
    t.terrain = TERRAIN_BUILDING;
    t.health = hp;
    t.max_health = hp;
}

#[test]
fn flip_depends_on_ignore_flip_not_on_pushable() {
    // +0x91E (`Pushable`) e +0x91F (não `IgnoreFlip`) são independentes em `Pawn::Push`.
    let flips = |pushable: u8, ignore_flip: bool| {
        let mut b = b8();
        let mut p = pawn(1, TEAM_ENEMY, 2, 2, 3);
        p.status.byte_0x91e = pushable;
        p.status.ignore_flip = ignore_flip;
        let i = b.add_pawn(p).unwrap();
        b.pawn_push(i, DIR_FLIP);
        b.events.iter().any(|e| matches!(e, BoardEvent::Flip(_)))
    };
    assert!(flips(1, false));
    assert!(flips(0, false));
    assert!(!flips(1, true));
    assert!(!flips(0, true));
}

#[test]
fn health_add_clamps_like_the_exe() {
    let (mut c, m) = (2, 3);
    assert_eq!(health_add(&mut c, m, -5), -2);
    assert_eq!(c, 0);
    assert_eq!(health_add(&mut c, m, 5), 3);
    assert_eq!(c, 3);
    // acima do máximo: delta 0 vira min(0, máx − atual) = −2 (devolvido), e a vida cai ao máximo
    let mut over = 5;
    assert_eq!(health_add(&mut over, 3, 0), -2);
    assert_eq!(over, 3);
}

#[test]
fn push_moves_and_bumps() {
    let mut b = b8();
    let e = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 3, 3)).unwrap();
    b.apply_effects(&[push(2, 3, 2, 0)]);
    assert_eq!((b.pawns[e].space, b.pawns[e].health), (pt(2, 2), 1));
    assert_eq!(b.tile(pt(2, 3)).pawns, Vec::<usize>::new());
    assert_eq!(b.tile(pt(2, 2)).pawns, vec![e]);

    // contra prédio: a unidade toma 1 a mais e o prédio toma 1 (vira entulho com vida 0)
    let mut b = b8();
    building(&mut b, 2, 2, 1);
    let e = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 3, 3)).unwrap();
    b.apply_effects(&[push(2, 3, 1, 0)]);
    assert_eq!((b.pawns[e].space, b.pawns[e].health), (pt(2, 3), 1));
    assert_eq!((b.tile(pt(2, 2)).terrain, b.tile(pt(2, 2)).health), (TERRAIN_RUBBLE, 0));

    // contra outra unidade: as duas tomam 1
    let mut b = b8();
    let e = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 3, 3)).unwrap();
    let f = b.add_pawn(pawn(2, TEAM_ENEMY, 2, 2, 3)).unwrap();
    b.apply_effects(&[push(2, 3, 0, 0)]);
    assert_eq!((b.pawns[e].health, b.pawns[f].health, b.pawns[e].space), (2, 2, pt(2, 3)));

    // montanha bloqueia e toma 1
    let mut b = b8();
    b.set_terrain(pt(2, 2), TERRAIN_MOUNTAIN);
    let e = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 3, 3)).unwrap();
    b.apply_effects(&[push(2, 3, 0, 0)]);
    assert_eq!((b.pawns[e].health, b.tile(pt(2, 2)).health), (2, 1));
}

#[test]
fn push_edge_cases() {
    // para fora do tabuleiro: nada é agendado
    let mut b = b8();
    let e = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 0, 3)).unwrap();
    b.apply_effects(&[push(2, 0, 0, 0)]);
    assert_eq!((b.pawns[e].space, b.pawns[e].health), (pt(2, 0), 3));
    // unidade estável (byte +0x91E == 0 → IsGuarding) não se move
    let mut g = pawn(2, TEAM_ENEMY, 4, 4, 3);
    g.status.byte_0x91e = 0;
    let g = b.add_pawn(g).unwrap();
    b.apply_effects(&[push(4, 4, 0, 1)]);
    assert_eq!(b.pawns[g].space, pt(4, 4));
    // DIR_FLIP não move
    b.apply_effects(&[push(2, 0, 0, DIR_FLIP)]);
    assert_eq!(b.pawns[e].space, pt(2, 0));
    // a casa empurra a ÚLTIMA unidade da lista
    let mut c = b8();
    let dead = c.add_pawn(pawn(3, TEAM_ENEMY, 1, 1, 0)).unwrap();
    let live = c.add_pawn(pawn(4, TEAM_ENEMY, 1, 1, 2)).unwrap();
    c.apply_effects(&[push(1, 1, 0, 2)]);
    assert_eq!((c.pawns[live].space, c.pawns[dead].space), (pt(1, 2), pt(1, 1)));
}

#[test]
fn water_and_hole_on_enter() {
    let mut b = b8();
    b.set_terrain(pt(2, 2), TERRAIN_WATER);
    b.set_terrain(pt(5, 2), TERRAIN_HOLE);
    let g = b.add_pawn(pawn(1, TEAM_ENEMY, 2, 3, 3)).unwrap();
    let mut f = pawn(2, TEAM_ENEMY, 3, 2, 3);
    f.movement = MovementTraits { flying: true, ..Default::default() };
    let f = b.add_pawn(f).unwrap();
    let mut m = pawn(3, TEAM_PLAYER, 5, 3, 3);
    m.damage.byte_0x99d = 1;
    m.status.acid = true;
    let m = b.add_pawn(m).unwrap();
    b.apply_effects(&[push(2, 3, 0, 0), push(3, 2, 0, 3), push(5, 3, 0, 0)]);
    assert!(b.pawns[g].is_dead() && b.pawns[g].status.died_0x921); // afogou
    assert_eq!((b.pawns[f].space, b.pawns[f].is_dead()), (pt(2, 2), false)); // voa sobre a água
    assert!(b.pawns[m].is_dead()); // buraco: cai mesmo com +0x99D
    // +0x99D sobrevive à água e deixa o ácido nela
    let mut c = b8();
    c.set_terrain(pt(2, 2), TERRAIN_WATER);
    let mut m = pawn(3, TEAM_PLAYER, 2, 3, 3);
    m.damage.byte_0x99d = 1;
    m.status.acid = true;
    let m = c.add_pawn(m).unwrap();
    c.apply_effects(&[push(2, 3, 0, 0)]);
    assert!(!c.pawns[m].is_dead() && c.tile(pt(2, 2)).acid == 1);
}

#[test]
fn shield_frozen_armor_acid_on_pawns() {
    let mut b = b8();
    let mut s = pawn(1, TEAM_ENEMY, 1, 1, 3);
    s.damage = DamageTraits { shield_0x8d4: true, ..Default::default() };
    let s = b.add_pawn(s).unwrap();
    let mut h = hit(1, 1, 2);
    h.fire = 1;
    b.apply_effects(&[h.clone()]);
    // o escudo absorve o dano; o fogo é aplicado depois, já sem escudo
    assert_eq!((b.pawns[s].health, b.pawns[s].damage.shield_0x8d4, b.pawns[s].status.fire), (3, false, true));
    b.pawns[s].damage.shield_0x8d4 = true;
    b.apply_effects(&[hit(1, 1, DAMAGE_DEATH)]);
    assert!(b.pawns[s].is_dead());

    let mut b = b8();
    let mut f = pawn(1, TEAM_ENEMY, 1, 1, 3);
    f.status.frozen = true;
    let f = b.add_pawn(f).unwrap();
    b.apply_effects(&[hit(1, 1, 2)]);
    assert_eq!((b.pawns[f].health, b.pawns[f].status.frozen), (3, false));
    b.apply_effects(&[hit(1, 1, DAMAGE_ZERO)]);
    assert_eq!(b.pawns[f].health, 3);

    let mut b = b8();
    let mut a = pawn(1, TEAM_ENEMY, 1, 1, 5);
    a.damage.query_0x63c100 = true; // blindado
    a.status.acid = true;           // ácido dobra depois da blindagem
    let a = b.add_pawn(a).unwrap();
    b.apply_effects(&[hit(1, 1, 2)]);
    assert_eq!(b.pawns[a].health, 3);
    // golpe de empurrão (modo 1) ignora blindagem e ácido
    b.add_pawn(pawn(2, TEAM_ENEMY, 1, 2, 3)).unwrap();
    b.apply_effects(&[push(1, 2, 0, 0)]);
    assert_eq!(b.pawns[a].health, 2);
}

#[test]
fn heal_clears_statuses() {
    let mut b = b8();
    let mut p = pawn(1, TEAM_PLAYER, 1, 1, 3);
    p.health = 1;
    p.status.fire = true;
    p.status.acid = true;
    let p = b.add_pawn(p).unwrap();
    b.apply_effects(&[hit(1, 1, -1)]);
    let s = &b.pawns[p];
    assert_eq!((s.health, s.status.fire, s.status.acid), (2, false, false));
}

#[test]
fn tile_fire_smoke_acid_and_terrain() {
    let mut b = b8();
    b.set_terrain(pt(0, 0), TERRAIN_FOREST);
    b.set_terrain(pt(1, 0), TERRAIN_ICE);
    b.set_terrain(pt(2, 0), TERRAIN_WATER);
    for x in 0..4 { let mut h = hit(x, 0, 0); h.fire = 1; b.apply_effects(&[h]); }
    assert_eq!((b.tile(pt(0, 0)).terrain, b.tile(pt(0, 0)).fire), (TERRAIN_ROAD, 2)); // floresta queima
    assert_eq!((b.tile(pt(1, 0)).terrain, b.tile(pt(1, 0)).fire), (TERRAIN_WATER, 0)); // gelo derrete
    assert_eq!(b.tile(pt(2, 0)).fire, 0); // água não pega fogo
    assert_eq!(b.tile(pt(3, 0)).fire, 1);
    let mut s = hit(3, 0, 0);
    s.smoke = 1;
    b.apply_effects(&[s]);
    assert_eq!((b.tile(pt(3, 0)).fire, b.tile(pt(3, 0)).smoke), (0, 1));
    let mut f = hit(4, 0, 0);
    f.fire = 1;
    let mut a = hit(4, 0, 0);
    a.acid = 1;
    b.apply_effects(&[f, a]);
    assert_eq!((b.tile(pt(4, 0)).fire, b.tile(pt(4, 0)).acid), (0, 1));
    // dano em floresta (sem iTerrain) acende; em areia vira fumaça e chão
    b.set_terrain(pt(5, 0), TERRAIN_FOREST);
    b.set_terrain(pt(6, 0), TERRAIN_SAND);
    b.apply_effects(&[hit(5, 0, 1), hit(6, 0, 1)]);
    assert_eq!(b.tile(pt(5, 0)).fire, 2);
    assert_eq!((b.tile(pt(6, 0)).terrain, b.tile(pt(6, 0)).smoke), (TERRAIN_ROAD, 1));
}

#[test]
fn mountain_ice_crack_and_hole() {
    let mut b = b8();
    b.set_terrain(pt(0, 0), TERRAIN_MOUNTAIN);
    b.set_terrain(pt(1, 0), TERRAIN_ICE);
    b.apply_effects(&[hit(0, 0, 1), hit(1, 0, 1)]);
    assert_eq!((b.tile(pt(0, 0)).terrain, b.tile(pt(0, 0)).health), (TERRAIN_MOUNTAIN, 1));
    assert!(b.is_terrain(pt(1, 0), 18)); // gelo danificado
    b.apply_effects(&[hit(0, 0, 3), hit(1, 0, 3)]);
    assert_eq!(b.tile(pt(0, 0)).terrain, TERRAIN_RUBBLE);
    assert_eq!(b.tile(pt(1, 0)).terrain, TERRAIN_WATER);
    // DAMAGE_DEATH derruba a montanha de uma vez
    b.set_terrain(pt(2, 0), TERRAIN_MOUNTAIN);
    b.apply_effects(&[hit(2, 0, DAMAGE_DEATH)]);
    assert_eq!(b.tile(pt(2, 0)).terrain, TERRAIN_RUBBLE);
    // rachar e depois atingir: buraco
    let mut c = hit(3, 3, 0);
    c.crack = 1;
    b.apply_effects(&[c]);
    assert_eq!(b.tile(pt(3, 3)).cracked, 1);
    b.apply_effects(&[hit(3, 3, 1)]);
    assert_eq!((b.tile(pt(3, 3)).terrain, b.tile(pt(3, 3)).cracked), (TERRAIN_HOLE, 0));
    // golpe de empurrão (modo 1) não abre o buraco
    let mut d = b8();
    d.tile_mut(pt(1, 1)).unwrap().cracked = 1;
    d.board_bump(pt(1, 1));
    assert_eq!(d.tile(pt(1, 1)).terrain, TERRAIN_ROAD);
}

#[test]
fn building_shield_frozen_and_add_building() {
    let mut b = b8();
    building(&mut b, 1, 1, 2);
    let mut s = hit(1, 1, 0);
    s.shield = 1;
    b.apply_effects(&[s]);
    assert_eq!(b.tile(pt(1, 1)).shield_0x2aa4, 2);
    b.apply_effects(&[hit(1, 1, 1)]);
    assert_eq!((b.tile(pt(1, 1)).health, b.tile(pt(1, 1)).shield_0x2aa4), (2, 0));
    let mut f = hit(1, 1, 0);
    f.frozen = 1;
    b.apply_effects(&[f]);
    assert!(b.tile(pt(1, 1)).frozen_0x2aa1);
    b.apply_effects(&[hit(1, 1, 1)]);
    assert_eq!((b.tile(pt(1, 1)).health, b.tile(pt(1, 1)).frozen_0x2aa1), (2, false));
    b.apply_effects(&[hit(1, 1, 1)]);
    assert_eq!(b.tile(pt(1, 1)).health, 1);
    // iTerrain = 1: mata quem está na casa e ergue prédio 1/1
    let mut c = b8();
    let e = c.add_pawn(pawn(1, TEAM_ENEMY, 4, 4, 3)).unwrap();
    let mut t = hit(4, 4, 0);
    t.terrain = 1;
    c.apply_effects(&[t]);
    assert!(c.pawns[e].is_dead());
    assert_eq!((c.tile(pt(4, 4)).terrain, c.tile(pt(4, 4)).health, c.tile(pt(4, 4)).max_health), (TERRAIN_BUILDING, 1, 1));
}

#[test]
fn movement_entries_move_the_first_pawn() {
    let mut b = b8();
    let m = b.add_pawn(pawn(0, TEAM_PLAYER, 1, 6, 3)).unwrap();
    let mut mv = hit(1, 6, 0);
    mv.path = vec![pt(1, 6), pt(1, 5), pt(1, 4)];
    b.apply_effects(&[mv]);
    assert_eq!(b.pawns[m].space, pt(1, 4));
    // tipo 3 (corpo a corpo) não é movimento
    let mut melee = hit(1, 4, 0);
    melee.path = vec![pt(1, 4), pt(1, 3)];
    melee.move_type = 3;
    b.apply_effects(&[melee]);
    assert_eq!(b.pawns[m].space, pt(1, 4));
}
