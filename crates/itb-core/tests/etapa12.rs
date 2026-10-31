//! Etapa 12: executor de SkillEffect com relógio (0x0055F7B0, 0x00568CF0, 0x005601C0).
use itb_core::apply::Hit;
use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, PawnStatusBytes, Point};
use itb_core::executor::{Executor, Timing};

fn pt(x: i32, y: i32) -> Point { Point::new(x, y) }
fn pawn(id: i32, x: i32, y: i32, hp: i32) -> PawnState {
    PawnState { id, team: TEAM_ENEMY, health: hp, max_health: hp, space: pt(x, y),
                status: PawnStatusBytes { powered: true, byte_0x91e: 1, ..Default::default() }, ..Default::default() }
}
fn hit(x: i32, y: i32, dmg: i32) -> Hit { Hit::new(pt(x, y), dmg) }

fn setup() -> (OccupancyBoard, usize) {
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    let e = b.add_pawn(pawn(1, 2, 3, 5)).unwrap();
    (b, e)
}

/// Empurra (2,3) para cima com atraso `d`, depois atinge (2,2).
fn push_then_hit(d: f32) -> Vec<Hit> {
    let mut a = hit(2, 3, 0);
    a.push = 0;
    a.delay = d;
    vec![a, hit(2, 2, 1)]
}

#[test]
fn short_delay_runs_before_the_push_lands() {
    let (mut b, e) = setup();
    let mut ex = Executor::default();
    ex.add_effect(push_then_hit(0.1));
    ex.run_until_idle(&mut b);
    // o golpe em (2,2) chega aos 0,1 s; a unidade só entra lá aos 0,4 s
    assert_eq!((b.pawns[e].space, b.pawns[e].health), (pt(2, 2), 5));
}

#[test]
fn long_and_full_delay_wait_for_the_push() {
    for d in [0.5, -1.0] {
        let (mut b, e) = setup();
        let mut ex = Executor::default();
        ex.add_effect(push_then_hit(d));
        ex.run_until_idle(&mut b);
        assert_eq!((b.pawns[e].space, b.pawns[e].health), (pt(2, 2), 4), "atraso {d}");
    }
}

#[test]
fn proj_delay_waits_only_for_projectiles() {
    // empurrão em andamento não segura PROJ_DELAY
    let (mut b, e) = setup();
    let mut ex = Executor::default();
    ex.add_effect(push_then_hit(-2.0));
    ex.run_until_idle(&mut b);
    assert_eq!(b.pawns[e].health, 5);
    // projétil em voo segura: o dano chega antes da entrada seguinte
    let (mut b, e) = setup();
    let mut p = hit(2, 3, 1);
    p.kind = 2;
    p.push = 0;
    p.delay = -2.0;
    let mut ex = Executor::with_timing(Timing { projectile: 0.3, ..Default::default() });
    ex.add_effect(vec![p, hit(2, 3, 1)]);
    ex.run_until_idle(&mut b);
    // 0,3 s: projétil (1 de dano, empurrão armado até 0,7 s); a entrada seguinte roda já aos 0,3 s,
    // com a unidade ainda em (2,3)
    assert_eq!((b.pawns[e].health, b.pawns[e].space), (3, pt(2, 2)));
    assert!(ex.log.iter().any(|l| l.starts_with("0.30s projétil chegou")));
}

#[test]
fn melee_lands_after_015s() {
    let (mut b, e) = setup();
    b.add_pawn(PawnState { id: 0, team: TEAM_PLAYER, health: 3, max_health: 3, space: pt(2, 4), mech: true, ..Default::default() }).unwrap();
    let mut m = hit(2, 3, 2);
    m.move_type = 3;
    m.path = vec![pt(2, 4)];
    m.push = 0;
    // golpe corpo a corpo e, sem pausa, um golpe direto na mesma casa
    let mut ex = Executor::default();
    ex.add_effect(vec![m, hit(2, 3, 1)]);
    ex.run_until_idle(&mut b);
    // o golpe direto (t=0) acerta (2,3); o corpo a corpo (t=0,15) também; o empurrão sai aos 0,55 s
    assert_eq!((b.pawns[e].health, b.pawns[e].space), (2, pt(2, 2)));
    assert!(ex.log.iter().any(|l| l.starts_with("0.15s golpe corpo a corpo")));
    assert!(ex.log.iter().any(|l| l.starts_with("0.55s empurrão")));
    // sem atacante na casa de origem: vira DamageSpace imediato
    let (mut b, e) = setup();
    let mut m = hit(2, 3, 2);
    m.move_type = 3;
    m.path = vec![pt(6, 6)];
    let mut ex = Executor::default();
    ex.add_effect(vec![m]);
    ex.run_until_idle(&mut b);
    assert_eq!(b.pawns[e].health, 3);
}

#[test]
fn deferred_rest_goes_to_the_end_of_the_queue() {
    let (mut b, _) = setup();
    let mut ex = Executor::default();
    let mut a1 = hit(0, 0, 0);
    a1.delay = 0.2;
    let mut fire = hit(0, 0, 0);
    fire.fire = 1;
    let mut smoke = hit(0, 0, 0);
    smoke.smoke = 1;
    // lista A: pausa 0,2 e depois fogo; lista B: fumaça (cronômetro −1, roda assim que ocioso)
    ex.add_effect(vec![a1, fire]);
    ex.add_effect(vec![smoke]);
    ex.run_until_idle(&mut b);
    // B roda antes do resto de A: a fumaça já está lá quando o fogo chega (fogo apaga fumaça)
    let t = b.tile(pt(0, 0));
    assert_eq!((t.fire, t.smoke), (1, 0));
}
