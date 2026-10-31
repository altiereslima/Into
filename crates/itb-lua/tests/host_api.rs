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

#[test]
fn record_classes_match_manifest_and_exe_defaults() {
    use itb_lua::api_manifest::CLASSES;
    for rc in itb_lua::records::RECORD_CLASSES {
        let c = CLASSES.iter().find(|c| c.name == rc.name).expect("classe no manifesto");
        let mut a: Vec<_> = c.properties.to_vec();
        let mut b: Vec<_> = rc.properties.to_vec();
        a.sort();
        b.sort();
        assert_eq!(a, b, "propriedades de {}", rc.name);
    }
    let h = host();
    assert_eq!(eval::<(String, i32, i32, i32, i32)>(&h, "local t = Text() return t.text, t.pos.x, t.size, t.align, t.width"),
               (String::new(), 0, 5, 2, i32::MAX));
    assert_eq!(eval::<(i32, i32)>(&h, "local t = Text('a', Point(1,2), 9) return t.size, t.align"), (9, 2));
    assert_eq!(eval::<i32>(&h, "local t = Text('a', Point(1,2), 9, 0) t.size = 2.5 return t.size"), 2);
    assert_eq!(eval::<(f32, i32)>(&h, "local j = JoyRumble(0.5, 300) return j.mag, j.length"), (0.5, 300));
    assert_eq!(eval::<(String, String)>(&h, "local p = PilotSkill('N', 'D') return p.name, p.desc"), ("N".into(), "D".into()));
    assert!(h.lua.load("local p = PilotSkill() p.nope = 1").exec().is_err());
    assert!(h.lua.load("StyleInfo('x')").exec().is_err());
}

#[test]
fn boot_clears_engine_globals_like_activelua() {
    let dir = std::env::temp_dir().join("itb_lua_fake_game");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts/scripts.lua"),
        "function GetScripts() return {'scripts/a.lua', 'scripts/falta.lua'} end").unwrap();
    std::fs::write(dir.join("scripts/a.lua"),
        "calls = {} function SetBoard(b) calls[#calls+1] = 'B' .. tostring(b) end \
         function SetPawn(p) calls[#calls+1] = 'P' .. tostring(p) end \
         function SetGame(g) calls[#calls+1] = 'G' .. tostring(g) end").unwrap();
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.boot().unwrap();
    assert_eq!(eval::<String>(&h, "return table.concat(calls, ',')"), "Pnil,Bnil,Gnil");
    let rep = h.report.borrow();
    assert!(rep.console.contains(&"scripts/falta.lua doesn't exist!".to_string()));
    assert_eq!(rep.scripts.len(), 3);
}

#[test]
fn objective_constructors_follow_repobj() {
    let h = host();
    let q = |c: &str| eval::<(String, String, String, i32, i32, i32)>(&h,
        &format!("local o = {c} return o.text, o.param1, o.param2, o.category, o.rep, o.potential"));
    assert_eq!(q("Objective('T', 3)"), ("T".into(), "".into(), "".into(), 0, 3, 3));
    assert_eq!(q("Objective('T', 'X')"), ("T".into(), "".into(), "".into(), 2, 1, 1));
    assert_eq!(q("Objective('T', 2, 5)"), ("T".into(), "".into(), "".into(), 0, 2, 5));
    assert_eq!(q("Objective('T', 'a', 'b', 4)"), ("T".into(), "a".into(), "b".into(), 0, 4, 4));
    assert_eq!(q("Objective('T', 'a', 'b', 'c')"), ("T".into(), "a".into(), "b".into(), 2, 1, 1));
    assert_eq!(q("Objective('T', 'a', 'b', 1, 2)"), ("T".into(), "a".into(), "b".into(), 0, 1, 2));
    assert!(h.lua.load("RegionInfo(Point(1,2))").exec().is_ok());
    assert!(h.lua.load("RegionInfo(1)").exec().is_err());
}

#[test]
fn space_damage_defaults_and_stub_methods() {
    let h = host();
    assert_eq!(eval::<(i32, i32, i32, i32, i32, i32)>(&h,
        "local d = SpaceDamage() return d.loc.x, d.loc.y, d.iDamage, d.iPush, d.iPawnTeam, d.iTerrain"), (-1, -1, 0, 4, 2, 10));
    assert_eq!(eval::<(i32, i32, i32)>(&h, "local d = SpaceDamage(Point(2,3), 1, DIR_LEFT) return d.loc.x, d.iDamage, d.iPush"), (2, 1, 3));
    assert_eq!(eval::<i32>(&h, "local d = SpaceDamage(5) return d.iDamage"), 5);
    assert_eq!(eval::<String>(&h, "local d = SpaceDamage(Point(1,1)) d.sAnimation = 'explo' return d.sAnimation"), "explo");
    h.lua.load("local d = SpaceDamage() d:IsMovement()").exec().unwrap();
    assert_eq!(h.report.borrow().stub_calls.get("SpaceDamage:IsMovement"), Some(&1));
}
