//! Etapa 06: Point, GL_Color, conversão número→int e rand() da CRT MSVC.
use itb_core::geometry::*;
use itb_core::rng::*;

#[test]
fn msvc_rand_sequence_matches_crt() {
    // Sequência conhecida do rand() do MSVC com srand(1) (padrão da CRT).
    let mut r = MsvcRand::default();
    let got: Vec<i32> = (0..5).map(|_| r.rand()).collect();
    assert_eq!(got, vec![41, 18467, 6334, 26500, 19169]);
    r.srand(1);
    assert_eq!(r.rand(), 41);
}

#[test]
fn random_wrappers_follow_0x4dfd50_family() {
    let mut r = MsvcRand::default();
    assert_eq!(r.random_int(0), 0);
    assert_eq!(r.seed(), 1, "n==0 não consome rand()");
    assert_eq!(r.random_int(10), 41 % 10);
    assert_eq!(r.random_int_range(5, 5), 5); // a == b não consome rand()
    assert_eq!(r.random_int_range(3, 8), 3 + 18467 % 5);
    assert_eq!(r.random_bool(2), Ok(6334 % 2 == 0));
    assert_eq!(r.random_bool_chance(1, 3), Ok(26500 % 3 < 1));
    assert_eq!(r.random_bool(0), Err(DivideByZero));
    // remainder truncado como idiv: divisor negativo não muda o sinal do resto
    let mut r = MsvcRand::default();
    assert_eq!(r.random_int(-10), 41 % -10);
    assert_eq!(41 % -10, 1);
}

#[test]
fn point_operations() {
    let a = IPoint::new(2, -3);
    let b = IPoint::new(-1, 7);
    assert_eq!(a + b, IPoint::new(1, 4));
    assert_eq!(a - b, IPoint::new(3, -10));
    assert_eq!(a * 3, IPoint::new(6, -9));
    assert_eq!(a.manhattan(b), 3 + 10);
    assert_eq!((IPoint::new(i32::MAX, 0) + IPoint::new(1, 0)).x, i32::MIN);
    assert_eq!(a.lua_string(), "Point( 2, -3 )");
    assert_eq!(IPoint::lua_default(), IPoint::new(-2147483647, -2147483647));
}

#[test]
fn get_direction_matches_0x4b2340() {
    assert_eq!(get_direction(IPoint::new(0, 0)), 4);
    assert_eq!(get_direction(IPoint::new(0, -5)), 0);
    assert_eq!(get_direction(IPoint::new(3, 0)), 1);
    assert_eq!(get_direction(IPoint::new(0, 1)), 2);
    assert_eq!(get_direction(IPoint::new(-9, 0)), 3);
    assert_eq!(get_direction(IPoint::new(4, -2)), 1); // eixo dominante X
    assert_eq!(get_direction(IPoint::new(2, 2)), 2); // empate → eixo Y
    assert_eq!(get_direction(IPoint::new(-2, -2)), 0);
}

#[test]
fn lua_number_to_int_rounds_like_fistp() {
    assert_eq!(lua_number_to_int(2.5), 2);
    assert_eq!(lua_number_to_int(3.5), 4);
    assert_eq!(lua_number_to_int(-2.5), -2);
    assert_eq!(lua_number_to_int(1.4999), 1);
    assert_eq!(lua_number_to_int(640.0), 640);
    assert_eq!(lua_number_to_int(1e12), i32::MIN);
    assert_eq!(lua_number_to_int(f64::NAN), i32::MIN);
}

#[test]
fn colors() {
    assert_eq!(Color::lua_default(), Color::rgba(1.0, 1.0, 1.0, 1.0));
    assert_eq!(Color::rgb(255.0, 0.0, 51.0).a, 1.0);
    let n = Color::rgba(255.0, 0.0, 51.0, 0.5).normalize();
    assert_eq!((n.r, n.g, n.b, n.a), (1.0, 0.0, 0.2, 0.5));
    assert_eq!(Color::rgb(1.0, 2.0, 3.0).with_alpha(0.25).a, 0.25);
}
