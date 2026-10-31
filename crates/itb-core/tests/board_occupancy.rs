//! Testes da Etapa 05: cada caso aponta o desvio do código de máquina que exercita.
use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, Point, Tile};

fn pawn(id: i32, team: i32, health: i32, x: i32, y: i32) -> PawnState {
    PawnState { id, team, health, max_health: 3, space: Point::new(x, y), ..Default::default() }
}

fn board() -> OccupancyBoard { OccupancyBoard::new(8, 8).unwrap() }

#[test]
fn pawn_accessors_match_field_reads() {
    let mut p = pawn(7, TEAM_PLAYER, 2, 1, 2);
    assert_eq!(p.get_health(), 2);
    assert!(!p.is_dead());
    assert!(p.is_damaged()); // 2 != 3
    p.health = 3;
    assert!(!p.is_damaged());
    p.health = 5; // 0x00638D50 usa `!=`, não `<`
    assert!(p.is_damaged());
    p.health = 0;
    assert!(p.is_dead()); // setle: 0 conta como morto
    p.health = -4;
    assert!(p.is_dead());
    assert!(p.is_player() && !p.is_enemy());
    p.set_team(TEAM_ENEMY);
    assert!(p.is_enemy() && !p.is_player());
    p.set_team(TEAM_MECH); // TEAM_MECH (4) não é "player" para IsPlayer
    assert!(!p.is_player());
    assert_eq!(p.get_id(), 7);
    assert!(!p.is_mech());
    p.set_mech();
    assert!(p.is_mech());
    assert_eq!(p.get_space(), Point::new(1, 2));
}

#[test]
fn is_corpse_follows_all_branches_of_0x629540() {
    let mut p = pawn(1, TEAM_PLAYER, 0, 0, 0);
    assert!(!p.is_corpse(false)); // nenhuma condição
    p.mech = true;
    assert!(p.is_corpse(false)); // mech e estado 0
    for s in [2, 3, 4] {
        p.state_0x9e0 = s;
        assert!(!p.is_corpse(false), "estado {s} desvia para o teste de 12");
    }
    p.state_0x9e0 = 5;
    assert!(p.is_corpse(false));
    p.mech = false;
    p.flag_0xf80 = true;
    assert!(p.is_corpse(false)); // +0xF80 equivale a mech neste teste
    p.flag_0xf80 = false;
    p.value_0x10e8 = 12;
    assert!(!p.is_corpse(false)); // vai para 0x639E10(12) == false
    p.query_0x639e10_12 = true;
    assert!(p.is_corpse(false));
    p.value_0x10e8 = 0;
    assert!(!p.is_corpse(false)); // lista global sem 12 → false sem consultar
    assert!(p.is_corpse(true)); // lista global com 12 → consulta
}

#[test]
fn tile_has_pawn_with_and_without_alive_check() {
    let mut b = board();
    let dead = b.add_pawn(pawn(1, TEAM_ENEMY, 0, 2, 2)).unwrap();
    let t = b.tile(Point::new(2, 2)).clone();
    assert!(b.tile_has_pawn(&t, false)); // só lista não vazia
    assert!(!b.tile_has_pawn(&t, true)); // morto, não cadáver
    b.pawns[dead].mech = true; // mech morto vira cadáver que ocupa
    assert!(b.tile_has_pawn(&t, true));
    assert!(!b.tile_has_pawn(&Tile::default(), false));
}

#[test]
fn is_pawn_space_rejects_invalid_and_defaults_to_alive_check() {
    let mut b = board();
    b.add_pawn(pawn(1, TEAM_ENEMY, 0, 3, 3)).unwrap();
    assert!(!b.is_pawn_space(Point::new(3, 3)));
    assert!(b.is_pawn_space_ex(Point::new(3, 3), false));
    assert!(!b.is_pawn_space_ex(Point::new(-1, 3), false));
    // sentinela povoada não muda o resultado: IsValid retorna antes.
    b.sentinel.pawns.push(0);
    assert!(!b.is_pawn_space_ex(Point::new(8, 0), false));
}

#[test]
fn get_pawn_point_returns_first_list_entry_even_if_dead() {
    let mut b = board();
    let dead = b.add_pawn(pawn(1, TEAM_ENEMY, 0, 4, 4)).unwrap();
    let alive = b.add_pawn(pawn(2, TEAM_PLAYER, 3, 4, 4)).unwrap();
    // 0x570E57: mov eax,[esi+0xA0]; mov eax,[eax] — sempre o índice 0.
    assert_eq!(b.get_pawn_at(Point::new(4, 4)), Some(dead));
    // A variante (Point,bool) procura a primeira NÃO morta.
    assert_eq!(b.get_pawn_at_ex(Point::new(4, 4), true), Some(alive));
    assert_eq!(b.get_pawn_at(Point::new(5, 5)), None);
    assert_eq!(b.get_pawn_at(Point::new(99, 0)), None);
}

#[test]
fn get_pawn_point_bool_returns_last_when_all_dead() {
    let mut b = board();
    let a = b.add_pawn(pawn(1, TEAM_ENEMY, 0, 1, 1)).unwrap();
    let c = b.add_pawn(pawn(2, TEAM_ENEMY, -1, 1, 1)).unwrap();
    // check_alive=false: lista não vazia; nenhuma viva → última da lista.
    assert_eq!(b.get_pawn_at_ex(Point::new(1, 1), false), Some(c));
    // check_alive=true sem cadáver: falha em IsPawnSpace.
    assert_eq!(b.get_pawn_at_ex(Point::new(1, 1), true), None);
    b.pawns[a].mech = true;
    assert_eq!(b.get_pawn_at_ex(Point::new(1, 1), true), Some(c));
}

#[test]
fn get_pawn_by_id_alive_and_space() {
    let mut b = board();
    b.add_pawn(pawn(10, TEAM_PLAYER, 3, 0, 0)).unwrap();
    let corpse = b.add_pawn(pawn(11, TEAM_PLAYER, 0, 0, 1)).unwrap();
    b.pawns[corpse].mech = true;
    let dup = b.add_pawn(pawn(10, TEAM_ENEMY, 1, 7, 7)).unwrap();
    assert_eq!(b.get_pawn_by_id(10), Some(0)); // primeira ocorrência
    assert_ne!(b.get_pawn_by_id(10), Some(dup));
    assert_eq!(b.get_pawn_by_id(99), None);
    assert!(b.is_pawn_alive(10));
    assert!(!b.is_pawn_alive(11)); // cadáver não está "vivo"
    assert!(!b.is_pawn_alive(99));
    assert_eq!(b.get_pawn_space(11), Point::new(0, 1));
    assert_eq!(b.get_pawn_space(99), Point::INVALID);
    // GetSpace lê o campo da unidade, não a casa onde está listada.
    b.pawns[0].space = Point::new(6, 6);
    assert_eq!(b.get_pawn_space(10), Point::new(6, 6));
}

#[test]
fn get_pawn_team_uses_first_entry_and_sentinel() {
    let mut b = board();
    assert_eq!(b.get_pawn_team(Point::new(2, 2)), TEAM_NONE);
    let d = b.add_pawn(pawn(1, TEAM_ENEMY, 0, 2, 2)).unwrap();
    b.add_pawn(pawn(2, TEAM_PLAYER, 3, 2, 2)).unwrap();
    assert_eq!(b.get_pawn_team(Point::new(2, 2)), TEAM_ENEMY); // primeiro item, morto
    b.pawns[d].health = 1;
    assert_eq!(b.get_pawn_team(Point::new(2, 2)), TEAM_ENEMY);
    // Inválida → casa sentinela (não retorna cedo).
    assert_eq!(b.get_pawn_team(Point::new(-5, 0)), TEAM_NONE);
    b.sentinel.pawns.push(1);
    assert_eq!(b.get_pawn_team(Point::new(-5, 0)), TEAM_PLAYER);
}

#[test]
fn terrain_table_of_0x5ae510() {
    let t = |terrain: i32| Tile { terrain, ..Default::default() };
    let blocks = OccupancyBoard::tile_terrain_blocks;
    // prédio: tudo menos phasing
    assert!(blocks(&t(TERRAIN_BUILDING), PATH_GROUND));
    assert!(blocks(&t(TERRAIN_BUILDING), PATH_FLYER));
    assert!(!blocks(&t(TERRAIN_BUILDING), PATH_PHASING));
    // água: ground e burrower
    assert!(blocks(&t(TERRAIN_WATER), PATH_GROUND));
    assert!(blocks(&t(TERRAIN_WATER), PATH_BURROWER));
    assert!(!blocks(&t(TERRAIN_WATER), PATH_MASSIVE));
    assert!(!blocks(&t(TERRAIN_WATER), PATH_FLYER));
    // montanha: sempre
    for prof in [PATH_GROUND, PATH_FLYER, PATH_PHASING, PATH_PROJECTILE] {
        assert!(blocks(&t(TERRAIN_MOUNTAIN), prof));
    }
    // gelo: só burrower
    assert!(blocks(&t(TERRAIN_ICE), PATH_BURROWER));
    assert!(!blocks(&t(TERRAIN_ICE), PATH_GROUND));
    // buraco: livre para 9,3,1,8,5
    for prof in [PATH_PHASING, PATH_PROJECTILE, PATH_FLYER, 8, 5] {
        assert!(!blocks(&t(TERRAIN_HOLE), prof));
    }
    for prof in [PATH_GROUND, PATH_MASSIVE, PATH_ROADRUNNER, PATH_BURROWER, 6] {
        assert!(blocks(&t(TERRAIN_HOLE), prof));
    }
    // fora de 1..=9 (estrada, fogo, ácido, lava) e 2/6/7/8: livre
    for terr in [TERRAIN_ROAD, TERRAIN_RUBBLE, TERRAIN_FOREST, TERRAIN_SAND, 8, TERRAIN_FIRE, TERRAIN_ACID, TERRAIN_LAVA, -1] {
        assert!(!blocks(&t(terr), PATH_GROUND), "terreno {terr}");
    }
    // máscara 0xF: 0x13 & 0xF = 3 (projectile) no buraco
    assert!(!blocks(&t(TERRAIN_HOLE), 0x13));
    // esteira só para burrower (e também com máscara)
    let conv = Tile { custom_tile: "tiles/conveyor_up.png".into(), ..Default::default() };
    assert!(blocks(&conv, PATH_BURROWER));
    assert!(blocks(&conv, 0x17));
    assert!(!blocks(&conv, PATH_GROUND));
}

#[test]
fn is_blocked_combines_bounds_terrain_and_pawns() {
    let mut b = board();
    assert!(b.is_blocked(Point::new(-1, 0), PATH_FLYER));
    assert!(b.is_blocked(Point::new(0, 8), PATH_PHASING));
    assert!(!b.is_blocked(Point::new(0, 0), PATH_GROUND));
    b.tile_mut(Point::new(1, 0)).unwrap().terrain = TERRAIN_MOUNTAIN;
    assert!(b.is_blocked(Point::new(1, 0), PATH_PHASING));
    // unidade viva do jogador bloqueia todos, menos phasing
    b.add_pawn(pawn(1, TEAM_PLAYER, 3, 2, 0)).unwrap();
    assert!(b.is_blocked(Point::new(2, 0), PATH_GROUND));
    assert!(b.is_blocked(Point::new(2, 0), PATH_FLYER));
    assert!(!b.is_blocked(Point::new(2, 0), PATH_PHASING));
    // inimigo bloqueia até phasing
    b.add_pawn(pawn(2, TEAM_ENEMY, 3, 3, 0)).unwrap();
    assert!(b.is_blocked(Point::new(3, 0), PATH_PHASING));
    // a regra de phasing compara o valor inteiro, sem máscara
    assert!(b.is_blocked(Point::new(2, 0), PATH_PHASING | 0x10));
    // unidade morta comum não bloqueia
    b.add_pawn(pawn(3, TEAM_ENEMY, 0, 4, 0)).unwrap();
    assert!(!b.is_blocked(Point::new(4, 0), PATH_GROUND));
}

#[test]
fn rust_constants_match_values_extracted_from_exe() {
    // CSV gerado por tools/map_lua_api_v2.py a partir de lua_pushinteger no EXE.
    let csv = include_str!("../../../re/evidencias/lua_global_values.csv");
    let get = |name: &str| -> i32 {
        csv.lines()
            .filter_map(|l| { let c: Vec<&str> = l.split(',').collect(); (c.len() == 4 && c[0] == name && c[2] == "integer").then(|| c[3].parse().unwrap()) })
            .next()
            .unwrap_or_else(|| panic!("{name} ausente no CSV"))
    };
    let expected = [
        ("TEAM_PLAYER", TEAM_PLAYER), ("TEAM_NONE", TEAM_NONE), ("TEAM_MECH", TEAM_MECH), ("TEAM_ENEMY", TEAM_ENEMY),
        ("PATH_GROUND", PATH_GROUND), ("PATH_FLYER", PATH_FLYER), ("PATH_MASSIVE", PATH_MASSIVE),
        ("PATH_PROJECTILE", PATH_PROJECTILE), ("PATH_ROADRUNNER", PATH_ROADRUNNER), ("PATH_BURROWER", PATH_BURROWER),
        ("PATH_PHASING", PATH_PHASING), ("TERRAIN_ROAD", TERRAIN_ROAD), ("TERRAIN_BUILDING", TERRAIN_BUILDING),
        ("TERRAIN_RUBBLE", TERRAIN_RUBBLE), ("TERRAIN_WATER", TERRAIN_WATER), ("TERRAIN_MOUNTAIN", TERRAIN_MOUNTAIN),
        ("TERRAIN_ICE", TERRAIN_ICE), ("TERRAIN_FOREST", TERRAIN_FOREST), ("TERRAIN_SAND", TERRAIN_SAND),
        ("TERRAIN_HOLE", TERRAIN_HOLE), ("TERRAIN_FIRE", TERRAIN_FIRE), ("TERRAIN_ACID", TERRAIN_ACID),
        ("TERRAIN_LAVA", TERRAIN_LAVA),
    ];
    for (name, value) in expected {
        assert_eq!(get(name), value, "{name}");
    }
    // Constante antiga da Etapa 03 confirmada: retorno 2 de GetPawnTeam == TEAM_NONE.
    assert_eq!(itb_core::board_queries::ORIGINAL_ABSENT_PAWN_TEAM, TEAM_NONE);
}
