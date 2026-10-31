//! Testes do host Lua sem arquivos do jogo (rodam no CI).
use itb_lua::{Host, HostConfig, Status};

fn host() -> Host {
    Host::new(std::env::temp_dir(), HostConfig::default()).unwrap()
}

fn eval<T: mlua::FromLuaMulti>(h: &Host, code: &str) -> T {
    h.lua.load(code).eval().unwrap_or_else(|e| panic!("{code}\n{e}"))
}

#[test]
fn constants_come_from_the_exe() {
    let h = host();
    assert_eq!(eval::<i32>(&h, "return TERRAIN_HOLE"), 9);
    assert_eq!(eval::<i32>(&h, "return TEAM_ENEMY"), 6);
    assert_eq!(eval::<i32>(&h, "return LEADER_BOSS"), 7);
    assert!(eval::<bool>(&h, "return EFFECT_DEADLY"));
    assert_eq!(eval::<(i32, i32)>(&h, "return VEC_UP.x, VEC_UP.y"), (0, -1));
    assert_eq!(eval::<f32>(&h, "return COLOR_RED.g"), 50.0);
}

#[test]
fn point_behaves_like_luabind_point() {
    let h = host();
    assert_eq!(eval::<(i32, i32)>(&h, "local p = Point(1,2) + Point(3,4) return p.x, p.y"), (4, 6));
    assert!(eval::<bool>(&h, "return Point(1,2) == Point(1,2)"));
    assert!(eval::<bool>(&h, "return Point(1,2) ~= Point(2,1)"));
    assert_eq!(eval::<(i32, i32)>(&h, "local p = Point(2,-3) * 3 return p.x, p.y"), (6, -9));
    assert_eq!(eval::<String>(&h, "return Point(5,6):GetString()"), "Point( 5, 6 )");
    assert_eq!(eval::<i32>(&h, "return Point(0,0):Manhattan(Point(2,-3))"), 5);
    // lua_tointeger do lua5.1.dll arredonda ao par: 640.5 → 640, 3.5 → 4
    assert_eq!(eval::<(i32, i32)>(&h, "local p = Point(1281/2, 3.5) return p.x, p.y"), (640, 4));
    assert_eq!(eval::<i32>(&h, "return Point().x"), -2147483647);
    assert_eq!(eval::<i32>(&h, "local a = Point(7,8) local b = Point(a) b.x = 1 return a.x"), 7);
    assert!(h.lua.load("return Point('a', 1)").exec().is_err());
}

#[test]
fn rect_button_and_color() {
    let h = host();
    assert_eq!(eval::<(i32, i32, i32, i32)>(&h, "local r = Rect2D() return r.x, r.y, r.w, r.h"), (-1, -1, 0, 0));
    assert_eq!(eval::<(i32, i32, i32, i32)>(&h, "local r = Rect2D(Point(1,2), Point(3,4)) return r.x, r.y, r.w, r.h"), (1, 2, 3, 4));
    assert_eq!(eval::<(i32, i32)>(&h, "local r = Rect2D(Point(1,2), 30, 40) return r.w, r.h"), (30, 40));
    assert_eq!(eval::<(i32, i32)>(&h, "local b = Button(Rect2D(5,6,7,8), 'L', 12) return b.pos.x, b.hitstats.w"), (5, 7));
    assert_eq!(eval::<(String, i32)>(&h, "local b = Button('img', Point(3,4), true) return b.image, b.hitstats.x"), ("img".into(), -1));
    assert_eq!(eval::<f32>(&h, "return GL_Color(255, 0, 0):Normalize().r"), 1.0);
    assert_eq!(eval::<f32>(&h, "return GL_Color(1,2,3).a"), 1.0);
}

#[test]
fn display_and_rng_globals() {
    let h = host();
    assert_eq!(eval::<(i32, i32)>(&h, "return ScreenSizeX(), ScreenSizeY()"), (1280, 720));
    assert_eq!(eval::<(i32, i32)>(&h, "return GetDefaultScale(), GetBoardScale()"), (2, 2));
    // srand(1): 41, 18467, ...
    assert_eq!(eval::<(i32, i32)>(&h, "return random_int(100), random_int(10, 20)"), (41, 10 + 18467 % 10));
    assert_eq!(eval::<i32>(&h, "return GetDirection(Point(0, 3))"), 2);
    assert!(h.lua.load("return random_bool(0)").exec().is_err());
}

#[test]
fn stubs_record_calls_and_coverage_is_reported() {
    let h = host();
    h.lua.load("local s = SkillEffect() s:AddDamage(nil) s:AddDamage(nil) PAWN_FACTORY:CreatePawn('x')").exec().unwrap();
    let rep = h.report.borrow();
    assert_eq!(rep.stub_calls.get("SkillEffect:AddDamage"), Some(&2));
    assert_eq!(rep.stub_calls.get("PawnFactory:CreatePawn"), Some(&1));
    assert_eq!(rep.coverage.get("Point:Manhattan"), Some(&Status::Reconstructed));
    assert_eq!(rep.coverage.get("SkillEffect:AddDamage"), Some(&Status::Stub));
    assert_eq!(rep.coverage.len(), 475);
}

/// Integração com a instalação real: `ITB_GAME_DIR=/caminho cargo test -p itb-lua -- --ignored`.
#[test]
#[ignore = "precisa dos arquivos originais do jogo (ITB_GAME_DIR)"]
fn original_scripts_boot_without_errors() {
    let dir = std::env::var("ITB_GAME_DIR").expect("defina ITB_GAME_DIR");
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.boot().unwrap();
    let rep = h.report.borrow();
    let errors: Vec<_> = rep.scripts.iter().filter(|(_, o)| matches!(o, itb_lua::Outcome::Error(_))).collect();
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(rep.scripts.len() > 100);
}
