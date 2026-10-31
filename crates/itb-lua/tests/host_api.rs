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
    h.lua.load("PAWN_FACTORY:CreatePawn('x') PAWN_FACTORY:CreatePawn('y')").exec().unwrap();
    let rep = h.report.borrow();
    assert_eq!(rep.stub_calls.get("PawnFactory:CreatePawn"), Some(&2));
    assert_eq!(rep.coverage.get("Point:Manhattan"), Some(&Status::Reconstructed));
    assert_eq!(rep.coverage.get("PawnFactory:CreatePawn"), Some(&Status::Stub));
    assert_eq!(rep.coverage.get("SkillEffect:AddDamage"), Some(&Status::Reconstructed));
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
    assert!(!eval::<bool>(&h, "return SpaceDamage():IsMovement()"));
}

#[test]
fn skill_effect_lists_have_reference_semantics() {
    let h = host();
    // padrão de ae_weapons.lua: modificar o último dano pela referência
    assert!(eval::<bool>(&h, "local ret = SkillEffect() ret:AddDamage(SpaceDamage(Point(1,1), 2)) \
        ret.effect:back().bHide = true return ret.effect:index(1).bHide"));
    // AddDamage copia: alterar o original depois não muda a lista
    assert_eq!(eval::<i32>(&h, "local ret = SkillEffect() local d = SpaceDamage(Point(1,1), 2) ret:AddDamage(d) \
        d.iDamage = 9 return ret:GetDamage(1).iDamage"), 2);
    assert_eq!(eval::<(i32, i32)>(&h, "local r = SkillEffect() r:AddQueuedDamage(SpaceDamage(3)) return r:GetDamageCount(), r:GetQueuedCount()"), (0, 1));
    assert_eq!(eval::<(i32, i32, i32)>(&h, "local r = SkillEffect() return r.iOwner, r.piOrigin.x, r.effect:size()"), (-1, -2147483647, 0));
}

#[test]
fn skill_effect_builders_match_exe() {
    let h = host();
    // AddDelay: SpaceDamage(0) com fDelay e bHide
    assert_eq!(eval::<(f32, bool, i32, i32)>(&h, "local r = SkillEffect() r:AddDelay(0.25) local d = r:GetDamage(1) \
        return d.fDelay, d.bHide, d.loc.x, d.iDamage"), (0.25, true, -1, 0));
    // projétil: atraso padrão -2 (PROJ_DELAY); laser zera o atraso
    assert_eq!(eval::<f32>(&h, "local r = SkillEffect() r:AddProjectile(SpaceDamage(Point(2,2), 1), 'effects/shot_mechtank') return r:GetDamage(1).fDelay"), -2.0);
    assert_eq!(eval::<f32>(&h, "local r = SkillEffect() r:AddProjectile(Point(0,0), SpaceDamage(Point(2,2), 1), 'effects/laser1', 0.5) return r:GetDamage(1).fDelay"), 0.0);
    assert_eq!(eval::<f32>(&h, "local r = SkillEffect() r:AddArtillery(SpaceDamage(Point(2,2), 1), 'effects/shotup', 0.3) return r:GetDamage(1).fDelay"), 0.3);
    // movimento: caminho com <= 1 ponto é recusado
    assert!(!eval::<bool>(&h, "local r = SkillEffect() local p = PointList() p:push_back(Point(1,1)) return r:AddMove(p, FULL_DELAY)"));
    assert_eq!(eval::<(bool, i32, i32, i32)>(&h, "local r = SkillEffect() local p = PointList() p:push_back(Point(1,1)) p:push_back(Point(1,3)) \
        local ok = r:AddMove(p, FULL_DELAY) local d = r:GetDamage(1) return ok, d:MoveEnd().y, d:MoveStart().y, r:GetDamageCount()"), (true, 3, 1, 1));
    assert!(eval::<bool>(&h, "local r = SkillEffect() local p = PointList() p:push_back(Point(1,1)) p:push_back(Point(2,1)) r:AddMove(p, 0) return r:GetDamage(1):IsMovement()"));
    // melee: tipo 3 → não é "movimento"
    assert!(!eval::<bool>(&h, "local r = SkillEffect() r:AddMelee(Point(1,1), SpaceDamage(Point(1,2), 1)) return r:GetDamage(1):IsMovement()"));
    // MoveStart com caminho vazio devolve (-1, 1) como o original
    assert_eq!(eval::<(i32, i32)>(&h, "local s = SpaceDamage():MoveStart() return s.x, s.y"), (-1, 1));
    // listas
    assert_eq!(eval::<(i32, i32, bool)>(&h, "local l = PointList() l:push_back(Point(1,2)) l:push_back(Point(3,4)) l:erase(1) \
        return l:size(), l:index(1).x, l:empty()"), (1, 3, false));
    assert!(h.lua.load("local l = PointList() return l:index(1)").exec().is_err());
}

#[test]
fn etapa10_skill_effects_airstrike_voice_dropper_emitter() {
    let h = host();
    // AddAirstrike: casa (0, y), depois atraso x*0.125+0.375
    assert_eq!(eval::<(i32, i32, f32, i32, f32, bool)>(&h, "local r = SkillEffect() r:AddAirstrike(Point(4,3), 'units/player/mech_jet_1.png') \
        local a, b = r:GetDamage(1), r:GetDamage(2) return a.loc.x, a.loc.y, a.fDelay, r:GetDamageCount(), b.fDelay, b.bHide"), (0, 3, -2.0, 2, 0.875, true));
    assert_eq!(eval::<(i32, i32, f32)>(&h, "local r = SkillEffect() r:AddReverseAirstrike(Point(4,3), 'x.png') \
        return r:GetDamage(1).loc.x, r:GetDamage(1).loc.y, r:GetDamage(2).fDelay"), (4, 0, 0.75));
    assert_eq!(eval::<(String, i32, i32)>(&h, "local r = SkillEffect() r:AddVoice('Mech_Repaired', 3) r:AddQueuedVoice('X', 1) \
        return r:GetDamage(1).sScript, r:GetDamageCount(), r:GetQueuedCount()"), ("PrepareVoiceEvent(\"Mech_Repaired\",3)".into(), 1, 1));
    assert_eq!(eval::<String>(&h, "local r = SkillEffect() r:AddEmitter(Point(1,1), 'Emitter_Burst') return r:GetDamage(1).sAnimation"), "add_emitter_Emitter_Burst");
    assert_eq!(eval::<(i32, f32)>(&h, "local r = SkillEffect() local d = SpaceDamage(Point(2,2), 1) d.fDelay = 0.5 r:AddDropper(d, 'pod.png') \
        return r:GetDamage(1).iDamage, r:GetDamage(1).fDelay"), (1, 0.0));
}

#[test]
fn etapa10_board_queries_from_lua() {
    use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, Point};
    let h = host();
    h.lua.load("function SetBoard(b) Board = b end function SetPawn(p) Pawn = p end").exec().unwrap();
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    b.tile_mut(Point::new(3, 3)).unwrap().cracked = 1;
    let m = b.add_pawn(PawnState { id: 0, team: TEAM_PLAYER, health: 3, max_health: 3, space: Point::new(0, 0), mech: true, ..Default::default() }).unwrap();
    b.add_pawn(PawnState { id: 1, team: TEAM_ENEMY, health: 2, max_health: 2, space: Point::new(2, 2), ..Default::default() }).unwrap();
    b.add_pawn(PawnState { id: 2, team: TEAM_ENEMY, health: 5, max_health: 5, space: Point::new(3, 3), ..Default::default() }).unwrap();
    let shared = std::rc::Rc::new(std::cell::RefCell::new(b));
    h.set_board(shared.clone()).unwrap();
    h.set_pawn(&shared, m).unwrap();
    assert!(eval::<bool>(&h, "return Board:IsDeadly(SpaceDamage(Point(2,2), 2), Pawn) and not Board:IsDeadly(SpaceDamage(Point(2,2), 1), nil)"));
    assert!(eval::<bool>(&h, "return Board:IsDeadly(SpaceDamage(Point(3,3), 1), Pawn)")); // chão rachado
    assert!(eval::<bool>(&h, "return Board:IsTerrain(Point(3,3), 16) and Board:IsTerrain(Point(1,1), TERRAIN_ROAD)"));
    assert_eq!(eval::<i32>(&h, "return Board:GetPath(Point(0,0), Point(2,2), Pawn:GetPathProf()):size()"), 5);
    assert!(eval::<bool>(&h, "return not Board:IsPod(Point(0,0)) and Board:IsCrackable(Point(1,1)) and Pawn:GetMoveSpeed() == 0"));
}

#[test]
fn etapa11_damage_space_applies_to_the_board() {
    use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, PawnStatusBytes, Point};
    let h = host();
    h.lua.load("function SetBoard(b) Board = b end").exec().unwrap();
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    let st = PawnStatusBytes { byte_0x91e: 1, ..Default::default() };
    let e = b.add_pawn(PawnState { id: 7, team: TEAM_ENEMY, health: 3, max_health: 3, space: Point::new(4, 4), status: st, ..Default::default() }).unwrap();
    let shared = std::rc::Rc::new(std::cell::RefCell::new(b));
    h.set_board(shared.clone()).unwrap();
    h.lua.load("local d = SpaceDamage(Point(4,4), 1, DIR_RIGHT) d.iFire = EFFECT_CREATE Board:DamageSpace(d) Board:DamageSpace(Point(5,4), 1)").exec().unwrap();
    let b = shared.borrow();
    assert_eq!((b.pawns[e].space, b.pawns[e].health, b.pawns[e].status.fire), (Point::new(5, 4), 1, true));
    assert_eq!(b.tile(Point::new(4, 4)).fire, 1);
}

#[test]
fn board_and_pawn_bindings_reach_the_core() {
    use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, Point};
    let h = host();
    h.lua.load("function SetBoard(b) Board = b end function SetPawn(p) Pawn = p end").exec().unwrap();
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    b.tile_mut(Point::new(2, 2)).unwrap().terrain = TERRAIN_MOUNTAIN;
    let m = b.add_pawn(PawnState { id: 0, team: TEAM_PLAYER, health: 3, max_health: 3, space: Point::new(1, 1), mech: true, ..Default::default() }).unwrap();
    b.add_pawn(PawnState { id: 7, team: TEAM_ENEMY, health: 2, max_health: 2, space: Point::new(4, 4), ..Default::default() }).unwrap();
    let shared = std::rc::Rc::new(std::cell::RefCell::new(b));
    h.set_board(shared.clone()).unwrap();
    h.set_pawn(&shared, m).unwrap();
    assert!(eval::<bool>(&h, "return Board:IsValid(Point(7,7)) and not Board:IsValid(8, 0)"));
    assert!(eval::<bool>(&h, "return Board:IsBlocked(Point(2,2), PATH_FLYER) and Board:IsBlocked(Point(4,4), PATH_PHASING)"));
    assert_eq!(eval::<(i32, i32)>(&h, "local s = Board:GetSize() return s.x, s.y"), (8, 8));
    assert_eq!(eval::<(i32, bool)>(&h, "local p = Board:GetPawn(Point(4,4)) return p:GetId(), p:IsEnemy()"), (7, true));
    assert!(eval::<bool>(&h, "return Board:GetPawn(Point(5,5)) == nil"));
    assert!(eval::<bool>(&h, "return Board:GetPawn(7) == Board:GetPawn(Point(4,4))"));
    assert!(eval::<bool>(&h, "return Pawn:IsMech() and Board:IsPawnTeam(Point(1,1), TEAM_PLAYER)"));
    assert_eq!(eval::<(i32, i32)>(&h, "local s = Board:GetPawnSpace(7) return s.x, s.y"), (4, 4));
    // escrita pelo script muda o núcleo compartilhado
    h.lua.load("Board:GetPawn(7):SetTeam(TEAM_PLAYER)").exec().unwrap();
    assert_eq!(shared.borrow().pawns[1].team, TEAM_PLAYER);
    assert_eq!(eval::<i32>(&h, "return Board:GetSimpleReachable(Point(1,1), 1, false):size()"), 4);
    // método ainda não reconstruído continua stub
    h.lua.load("Board:GetDistance(Point(1,1), Point(3,3))").exec().unwrap();
    assert_eq!(h.report.borrow().stub_calls.get("Board:GetDistance"), Some(&1));
    // Dijkstra do EXE: montanha em (2,2), mech em (1,1), inimigo em (4,4); ordem (x, y)
    let reach: Vec<(i32, i32)> = h.lua.load("local l = Board:GetReachable(Point(1,1), 2, PATH_GROUND) local r = {} \
        for i = 1, l:size() do r[#r+1] = {l:index(i).x, l:index(i).y} end return r").eval::<Vec<Vec<i32>>>().unwrap()
        .into_iter().map(|v| (v[0], v[1])).collect();
    assert_eq!(reach, vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 2), (1, 3), (2, 0), (2, 1), (3, 1)]);
}

/// Integração: armas originais executam no motor Rust (`ITB_GAME_DIR=... cargo test -- --ignored`).
#[test]
#[ignore = "precisa dos arquivos originais do jogo (ITB_GAME_DIR)"]
fn original_laser_weapon_produces_falloff_damage() {
    use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, Point};
    let dir = std::env::var("ITB_GAME_DIR").expect("defina ITB_GAME_DIR");
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.boot().unwrap();
    let mut b = OccupancyBoard::new(8, 8).unwrap();
    let m = b.add_pawn(PawnState { id: 0, team: TEAM_PLAYER, health: 3, space: Point::new(3, 4), mech: true, ..Default::default() }).unwrap();
    let shared = std::rc::Rc::new(std::cell::RefCell::new(b));
    h.set_board(shared.clone()).unwrap();
    h.set_pawn(&shared, m).unwrap();
    let dmg: Vec<i32> = h.lua.load("local r = Prime_Lasermech:GetSkillEffect(Point(3,4), Point(3,3)) local t = {} \
        for i = 1, r.effect:size() do t[i] = r.effect:index(i).iDamage end return t").eval().unwrap();
    assert_eq!(dmg, vec![3, 2, 1, 1]);
}
