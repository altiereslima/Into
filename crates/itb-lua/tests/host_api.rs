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
    h.lua.load("local g = GameMap() g:GetPower() g:GetPower()").exec().unwrap();
    let rep = h.report.borrow();
    assert_eq!(rep.stub_calls.get("GameMap:GetPower"), Some(&2));
    assert_eq!(rep.coverage.get("Point:Manhattan"), Some(&Status::Reconstructed));
    assert_eq!(rep.coverage.get("GameMap:AddNote"), Some(&Status::Stub));
    assert_eq!(rep.coverage.get("PawnFactory:CreatePawn"), Some(&Status::Reconstructed));
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
    drop(b);
    // AddEffect(SkillEffect): fila do tabuleiro; o empurrão termina antes do golpe com FULL_DELAY
    h.lua.load("local fx = SkillEffect() local d = SpaceDamage(Point(5,4), 0, DIR_DOWN) fx:AddDamage(d) fx:AddDelay(FULL_DELAY) \
        fx:AddDamage(SpaceDamage(Point(5,5), 1)) Board:AddEffect(fx)").exec().unwrap();
    let b = shared.borrow();
    assert_eq!((b.pawns[e].space, b.pawns[e].health), (Point::new(5, 5), 0));
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

/// Mapa sintético, no formato dos `.map` (nenhum arquivo do jogo).
#[test]
fn map_table_becomes_a_board() {
    use itb_core::board_occupancy::{consts::*, Point};
    let dir = std::env::temp_dir().join(format!("itb_map_teste_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("maps")).unwrap();
    std::fs::write(dir.join("maps/teste.map"), r#"teste = {["version"] = 7, ["dimensions"] = Point( 8, 8 ), ["name"] = "teste", ["enemy_kills"] = 5,
["map"] = {{["loc"] = Point( 0, 0 ), ["terrain"] = 4, },
{["loc"] = Point( 1, 0 ), ["terrain"] = 1, ["populated"] = 1, ["health_max"] = 4, },
{["loc"] = Point( 2, 0 ), ["terrain"] = 3, ["poison"] = 1, },
{["loc"] = Point( 3, 0 ), ["terrain"] = 0, ["lava"] = true, },
{["loc"] = Point( 4, 0 ), ["terrain"] = 0, ["pawn"] = "Wall", ["team"] = 6, ["custom"] = "ground_rail.png", },
},
["spawns"] = {},
["spawn_ids"] = {},
["spawn_points"] = {},
["zones"] = {["dam"] = {Point( 5, 5 ), Point( 5, 6 ), },
},
["tags"] = {"generic", "acid", },
["blocked_points"] = {Point( 7, 7 ), },
["blocked_type"] = {2, },
}
"#).unwrap();
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.lua.load("function SetBoard(b) Board = b end").exec().unwrap();
    let (board, loaded) = h.load_map("teste").unwrap();
    assert!(loaded.unsupported.is_empty(), "{:?}", loaded.unsupported);
    {
        let b = board.borrow();
        assert_eq!((b.get_size().x, b.mission.enemy_kills, b.mission.name.as_str()), (8, 5, "teste"));
        assert_eq!(b.tile(Point::new(0, 0)).terrain, TERRAIN_MOUNTAIN);
        assert_eq!((b.tile(Point::new(1, 0)).health, b.tile(Point::new(1, 0)).byte_0x2710), (4, true));
        assert_eq!(b.tile(Point::new(2, 0)).acid, 1);
        assert!(b.tile(Point::new(3, 0)).lava_0x2ac8);
        assert_eq!(b.tile(Point::new(4, 0)).custom_tile, "ground_rail.png");
        // tipo inexistente vira "PunchMech", como em CreateEmptyPawn; `team` do arquivo não é lida
        assert!(b.mission.pending_pawns.is_empty());
        assert_eq!((b.pawns[0].type_name.as_str(), b.pawns[0].space, b.pawns[0].id), ("PunchMech", Point::new(4, 0), 5));
        assert_eq!(b.mission.tags, vec!["generic".to_string(), "acid".to_string()]);
        assert_eq!(b.mission.block_spawn.get(&Point::new(7, 7)), Some(&2));
    }
    // os scripts recebem o tabuleiro e leem as zonas
    assert_eq!(eval::<(i32, i32)>(&h, r#"local z = Board:GetZone("dam") return z:size(), z:index(2).y"#), (2, 6));
    assert_eq!(eval::<i32>(&h, r#"return Board:GetZone("nada"):size()"#), 0);
    assert_eq!(eval::<i32>(&h, r#"return Board:GetZone("enemy"):size()"#), 12);
    assert!(eval::<bool>(&h, "return Board:IsBuilding(Point(1,0))"));
    // chave desconhecida é relatada, não ignorada em silêncio
    h.lua.load(r#"teste.rain = 3 teste.map[1].vines = 1"#).exec().unwrap();
    let t: mlua::Table = h.lua.globals().get("teste").unwrap();
    let mut u = itb_lua::map::read_map(&t).unwrap().unsupported;
    u.sort();
    assert_eq!(u, vec!["map.vines".to_string(), "rain".to_string()]);
    std::fs::remove_dir_all(&dir).ok();
}

/// Fábrica de unidades com uma tabela sintética, no formato de `CreateClass`.
#[test]
fn pawn_factory_reads_getters_and_board_takes_the_pawn() {
    use itb_core::board_occupancy::{consts::*, OccupancyBoard, Point};
    let h = host();
    h.lua.load(r#"
        Besta = { Health = 3, MoveSpeed = 2, DefaultTeam = TEAM_ENEMY, Pushable = true, SkillList = {"Garra", "Mordida"}, Massive = true }
        for k, v in pairs({Health = 1, MoveSpeed = 1, DefaultTeam = 1, Pushable = 1, SkillList = 1, Massive = 1}) do
            local key = k
            Besta["Get" .. key] = function(self) return self[key] end
        end
        function SetBoard(b) Board = b end
    "#).exec().unwrap();
    let shared = std::rc::Rc::new(std::cell::RefCell::new(OccupancyBoard::new(8, 8).unwrap()));
    h.set_board(shared.clone()).unwrap();
    // antes de entrar no tabuleiro a unidade já responde
    assert_eq!(eval::<(i32, i32, bool)>(&h, r#"p = PAWN_FACTORY:CreatePawn("Besta") return p:GetId(), p:GetHealth(), p:IsEnemy()"#), (5, 3, true));
    assert_eq!(eval::<(i32, i32)>(&h, "local l = Board:AddPawn(p, Point(2,3)) return l.x, l.y"), (2, 3));
    // a mesma referência agora aponta para o tabuleiro
    assert_eq!(eval::<(i32, i32)>(&h, "local s = p:GetSpace() return s.x, s.y"), (2, 3));
    assert!(eval::<bool>(&h, "return Board:GetPawn(Point(2,3)) == p"));
    // forma com nome; time explícito vence o padrão
    assert_eq!(eval::<(i32, i32)>(&h, r#"local l = Board:AddPawn("Besta", Point(4,4)) return l.x, l.y"#), (4, 4));
    assert_eq!(eval::<i32>(&h, r#"return PAWN_FACTORY:CreatePawn("Besta", TEAM_PLAYER):GetTeam()"#), TEAM_PLAYER);
    let b = shared.borrow();
    assert_eq!(b.pawns[0].skills, vec!["Move".to_string(), "Garra".to_string(), "Mordida".to_string()]);
    assert_eq!((b.pawns[0].base_move_0x63a8f0, b.pawns[0].status.byte_0x91e, b.pawns[0].movement.massive), (2, 1, true));
    assert_eq!(b.get_pawn_at(Point::new(4, 4)), Some(1));
    assert_eq!(b.pawns[1].id, 6);
}

/// `Game` real, `SpawnPawn` e fila vistos pela Lua, sem arquivos do jogo.
#[test]
fn game_and_spawn_queue_through_lua() {
    use itb_core::board_occupancy::{consts::*, OccupancyBoard};
    use itb_core::game::GameState;
    let h = host();
    h.lua.load(r#"
        Besta = { Health = 3, DefaultTeam = TEAM_ENEMY, GetHealth = function(s) return s.Health end, GetDefaultTeam = function(s) return s.DefaultTeam end }
        function SetBoard(b) Board = b end
        function SetGame(g) Game = g end
        function getCityPower(sector) return 7 end
    "#).exec().unwrap();
    let game = h.new_game(GameState { sector_0x5238: 1, ..Default::default() }).unwrap();
    assert_eq!(eval::<(i32, i32, i32, bool)>(&h, "return Game:GetSector(), Game:GetTeamTurn(), Game:GetTurnCount(), Game:IsIslandUnlocked(1)"), (2, TEAM_NONE, 0, false));
    assert_eq!(eval::<(i32, i32)>(&h, "local p = Game:GetPower() return p:GetValue(), p:GetMax()"), (5, 7));
    game.borrow_mut().turn = Some((2, TEAM_PLAYER));
    assert_eq!(eval::<(i32, i32)>(&h, "return Game:GetTeamTurn(), Game:GetTurnCount()"), (TEAM_PLAYER, 2));
    // método ainda não reconstruído continua stub
    h.lua.load(r#"Game:AddNote("x")"#).exec().unwrap();
    assert_eq!(h.report.borrow().stub_calls.get("GameMap:AddNote"), Some(&1));
    let shared = std::rc::Rc::new(std::cell::RefCell::new(OccupancyBoard::new(8, 8).unwrap()));
    h.set_board(shared.clone()).unwrap();
    // srand(1): 41, 18467, 6334, 26500... CreatePawn gasta 3, e 26500 % 12 = 4; a referência segue válida
    h.lua.load(r#"p = PAWN_FACTORY:CreatePawn("Besta") Board:SpawnPawn(p)"#).exec().unwrap();
    assert!(eval::<bool>(&h, "return Board:IsSpawning(Point(6,2))"));
    assert_eq!(eval::<i32>(&h, "return Board:GetPawns(TEAM_ENEMY):size()"), 0);
    assert_eq!(eval::<i32>(&h, r#"return Board:SpawnPawn("Besta", Point(5,2))"#), 6);
    h.lua.load("Board:SpawnQueued()").exec().unwrap();
    assert_eq!(eval::<(i32, i32)>(&h, "local s = p:GetSpace() return s.x, s.y"), (6, 2));
    assert_eq!(eval::<(i32, i32, i32)>(&h, "local l = Board:GetPawns(TEAM_ENEMY) return l:size(), l:index(1), l:index(2)"), (2, 5, 6));
    // forma sem casa em AddPawn, terreno e bloqueio
    h.lua.load("Board:SetTerrain(Point(7,2), TERRAIN_MOUNTAIN) Board:BlockSpawn(Point(5,3), 1)").exec().unwrap();
    assert_eq!(shared.borrow().tile(itb_core::board_occupancy::Point::new(7, 2)).terrain, TERRAIN_MOUNTAIN);
    assert!(eval::<bool>(&h, r#"local l = Board:AddPawn("Besta") return Board:IsPawnSpace(l)"#));
}

/// Integração: `startNewGame` e `Mission:BaseStart` originais terminam sem tocar stub.
#[test]
#[ignore = "precisa dos arquivos originais do jogo (ITB_GAME_DIR)"]
fn original_mission_start_runs_without_stubs() {
    let dir = std::env::var("ITB_GAME_DIR").expect("defina ITB_GAME_DIR");
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.boot().unwrap();
    h.new_game(Default::default()).unwrap();
    let (board, _) = h.load_map("any3").unwrap();
    h.report.borrow_mut().stub_calls.clear();
    h.lua.load("startNewGame() Mission_Acid:new():BaseStart()").exec().unwrap();
    assert!(h.report.borrow().stub_calls.is_empty(), "{:?}", h.report.borrow().stub_calls);
    {
        let b = board.borrow();
        assert_eq!(b.spawn_queue.len(), 3);
        for (i, p) in b.spawn_queue.iter().zip(&b.spawn_list) {
            assert_eq!(b.pawns[*i].team, itb_core::board_occupancy::consts::TEAM_ENEMY);
            assert!((5..=7).contains(&p.x) && (2..=5).contains(&p.y), "{p:?}");
        }
    }
    h.lua.load("Board:SpawnQueued()").exec().unwrap();
    assert_eq!(eval::<i32>(&h, "return Board:GetPawns(TEAM_ENEMY):size()"), 3);
}

/// Integração: toda unidade original nasce do seu tipo.
#[test]
#[ignore = "precisa dos arquivos originais do jogo (ITB_GAME_DIR)"]
fn every_original_pawn_type_can_be_created() {
    use itb_core::board_occupancy::consts::*;
    let dir = std::env::var("ITB_GAME_DIR").expect("defina ITB_GAME_DIR");
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.boot().unwrap();
    let names: Vec<String> = h.lua.load("return PawnList").eval().unwrap();
    assert!(names.len() > 100, "{}", names.len());
    for n in &names {
        let p = itb_lua::pawns::create_pawn(&h.lua, n, TEAM_NONE).unwrap_or_else(|e| panic!("{n}: {e}"));
        assert_eq!(&p.type_name, n);
    }
    let mech = itb_lua::pawns::create_pawn(&h.lua, "PunchMech", TEAM_NONE).unwrap();
    assert_eq!((mech.team, mech.health, mech.base_move_0x63a8f0, mech.movement.massive), (TEAM_PLAYER, 3, 3, true));
    assert_eq!(mech.skills, vec!["Move".to_string(), "Prime_Punchmech".to_string()]);
    let bug = itb_lua::pawns::create_pawn(&h.lua, "Firefly1", TEAM_NONE).unwrap();
    assert_eq!(bug.team, TEAM_ENEMY);
    let missing = itb_lua::pawns::create_pawn(&h.lua, "Nao_Existe", TEAM_NONE).unwrap();
    assert_eq!(missing.type_name, "PunchMech");
}

/// Integração: todos os mapas originais carregam (`ITB_GAME_DIR=... cargo test -- --ignored`).
#[test]
#[ignore = "precisa dos arquivos originais do jogo (ITB_GAME_DIR)"]
fn every_original_map_loads() {
    use itb_core::board_occupancy::consts::*;
    let dir = std::env::var("ITB_GAME_DIR").expect("defina ITB_GAME_DIR");
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.boot().unwrap();
    let names = h.load_map_data().unwrap();
    assert!(names.len() >= 376, "{}", names.len());
    assert_eq!(eval::<i64>(&h, "return #MAP_LIST") as usize, names.len());
    for name in &names {
        let (board, loaded) = h.load_map(name).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(loaded.unsupported.is_empty(), "{name}: {:?}", loaded.unsupported);
        let b = board.borrow();
        assert_eq!((b.get_size().x, b.get_size().y), (8, 8), "{name}");
        for t in &loaded.data.tiles {
            if !t.lava {
                assert_eq!(b.tile(t.loc).terrain, t.terrain, "{name} {:?}", t.loc);
            } else {
                assert_eq!(b.tile(t.loc).terrain, TERRAIN_WATER, "{name} {:?}", t.loc);
            }
            assert!(b.is_valid(t.loc), "{name} {:?}", t.loc);
        }
    }
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

/// Integração: a preparação de `BoardPlayer::LoadMission` roda em todos os mapas, com as funções Lua originais e
/// sem stub. Setor 0 de grama: metade dos prédios com vida 1 e metade com vida 2 (`getBuildingHealth(0)`).
#[test]
#[ignore = "precisa dos arquivos originais do jogo (ITB_GAME_DIR)"]
fn original_mission_setup_runs_on_every_map() {
    use itb_core::board_occupancy::consts::*;
    let dir = std::env::var("ITB_GAME_DIR").expect("defina ITB_GAME_DIR");
    let h = Host::new(&dir, HostConfig::default()).unwrap();
    h.boot().unwrap();
    h.new_game(Default::default()).unwrap();
    h.lua.load("startNewGame() GAME.Missions = { [1] = CreateMission('Mission_Acid') }").exec().unwrap();
    let names = h.load_map_data().unwrap();
    h.report.borrow_mut().stub_calls.clear();
    for name in &names {
        let (board, _) = h.load_mission(name, 0, "grass", "Mission1").unwrap_or_else(|e| panic!("{name}: {e}"));
        let b = board.borrow();
        let n = b.buildings.len() as i32;
        let um = b.buildings.iter().filter(|&&p| b.get_health(p) == 1).count() as i32;
        let dois = b.buildings.iter().filter(|&&p| b.get_health(p) == 2).count() as i32;
        let meio = itb_core::mission_setup::building_count(50, n as usize);
        assert_eq!((um, dois), (meio.min(n), (n - meio).clamp(0, meio)), "{name}");
        for x in 0..8 {
            for y in 0..8 {
                let p = itb_core::board_occupancy::Point::new(x, y);
                if b.get_terrain(p) == TERRAIN_RUBBLE { assert_eq!(b.get_health(p), 0, "{name} {p:?}"); }            }
        }
    }
    assert!(h.report.borrow().stub_calls.is_empty(), "{:?}", h.report.borrow().stub_calls);
}
