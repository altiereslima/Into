//! `itb-map <pasta-do-jogo> <nome>`: carrega um mapa original e desenha o tabuleiro.
//! `itb-map <pasta-do-jogo> --todos`: carrega todos os mapas e resume.
//! `itb-map <pasta-do-jogo> <nome> --missao <Tabela>`: experimento; roda `Tabela:new():BaseStart()` e lista
//! os stubs tocados, para orientar o que reconstruir.
use itb_core::board_occupancy::{consts::*, OccupancyBoard, Point};
use itb_lua::{Host, HostConfig};
use std::collections::BTreeMap;

fn glyph(b: &OccupancyBoard, p: Point) -> char {
    let t = b.tile(p);
    if t.lava_0x2ac8 { return 'L'; }
    let c = match t.terrain {
        TERRAIN_ROAD => '.',
        TERRAIN_BUILDING => 'B',
        TERRAIN_RUBBLE => 'r',
        TERRAIN_WATER => '~',
        TERRAIN_MOUNTAIN => 'M',
        TERRAIN_ICE => 'i',
        TERRAIN_FOREST => 'F',
        TERRAIN_SAND => 's',
        TERRAIN_HOLE => 'O',
        _ => '?',
    };
    if t.acid != 0 && c == '~' { 'a' } else { c }
}

fn draw(b: &OccupancyBoard) {
    let s = b.get_size();
    for y in 0..s.y {
        let line: String = (0..s.x).map(|x| glyph(b, Point::new(x, y))).collect();
        println!("  {line}");
    }
    println!("  . chão  B prédio  r entulho  ~ água  a água com ácido  L lava  M montanha  i gelo  F floresta  s areia  O buraco");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: itb-map <pasta do jogo> <nome do mapa> | --todos");
        std::process::exit(2);
    }
    let host = Host::new(&args[1], HostConfig::default()).expect("host");
    host.boot().expect("boot");
    if args[2] != "--todos" {
        let (board, loaded) = host.load_map(&args[2]).expect("mapa");
        let b = board.borrow();
        println!("# {}  tamanho {}x{}  casas listadas {}", args[2], loaded.data.size.x, loaded.data.size.y, loaded.data.tiles.len());
        draw(&b);
        let health: Vec<String> = loaded.data.tiles.iter().filter(|t| t.terrain == TERRAIN_BUILDING)
            .map(|t| format!("({},{}) {}/{}", t.loc.x, t.loc.y, b.tile(t.loc).health, b.tile(t.loc).max_health)).collect();
        println!("prédios: {}", health.join("  "));
        println!("zonas: {:?}", loaded.data.zones.keys().collect::<Vec<_>>());
        println!("etiquetas: {:?}", loaded.data.tags);
        for &i in &b.pawn_list {
            let p = &b.pawns[i];
            println!("unidade {} {} time {} em ({},{}) vida {}/{} armas {:?}", p.id, p.type_name, p.team, p.space.x, p.space.y, p.health, p.max_health, p.skills);
        }
        println!("chaves não aplicadas: {:?}", loaded.unsupported);
        drop(b);
        if args.get(3).map(String::as_str) == Some("--missao") {
            let table = args.get(4).expect("nome da tabela da missão");
            host.report.borrow_mut().stub_calls.clear();
            host.new_game(Default::default()).expect("Game");
            if let Err(e) = host.lua.load("startNewGame()").exec() {
                println!("\nstartNewGame parou: {}", e.to_string().lines().take(4).collect::<Vec<_>>().join(" | "));
            }
            match host.lua.load(format!("local m = {table}:new() m:BaseStart() return true")).exec() {
                Ok(()) => println!("\nBaseStart terminou sem erro"),
                Err(e) => println!("\nBaseStart parou: {}", e.to_string().lines().take(4).collect::<Vec<_>>().join(" | ")),
            }
            let r = host.report.borrow();
            let mut calls: Vec<_> = r.stub_calls.iter().collect();
            calls.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            println!("stubs tocados: {calls:?}");
            let b = board.borrow();
            println!("unidades depois: {}", b.pawn_list.iter().map(|&i| format!("{}@({},{})", b.pawns[i].type_name, b.pawns[i].space.x, b.pawns[i].space.y)).collect::<Vec<_>>().join(" "));
            println!("na fila de surgimento: {}", b.spawn_queue.iter().zip(&b.spawn_list).map(|(&i, p)| format!("{}@({},{})", b.pawns[i].type_name, p.x, p.y)).collect::<Vec<_>>().join(" "));
        }
        return;
    }
    let names = host.load_map_data().expect("maphelper e mapas");
    let (mut ok, mut failed, mut tiles) = (0, Vec::new(), 0);
    let mut terrain: BTreeMap<i32, usize> = BTreeMap::new();
    let (mut lava, mut acid, mut fire, mut populated, mut pending) = (0, 0, 0, 0, 0);
    let mut unsupported: BTreeMap<String, usize> = BTreeMap::new();
    let mut sizes: BTreeMap<(i32, i32), usize> = BTreeMap::new();
    for name in &names {
        match host.load_map(name) {
            Ok((board, loaded)) => {
                ok += 1;
                tiles += loaded.data.tiles.len();
                for k in loaded.unsupported { *unsupported.entry(k).or_default() += 1; }
                let b = board.borrow();
                let s = b.get_size();
                *sizes.entry((s.x, s.y)).or_default() += 1;
                pending += b.pawns.len();
                for t in &loaded.data.tiles { populated += (t.populated == 1) as usize; }
                for y in 0..s.y {
                    for x in 0..s.x {
                        let t = b.tile(Point::new(x, y));
                        *terrain.entry(t.terrain).or_default() += 1;
                        lava += t.lava_0x2ac8 as usize;
                        acid += (t.acid != 0) as usize;
                        fire += (t.fire != 0) as usize;
                    }
                }
            }
            Err(e) => failed.push((name.clone(), e.to_string())),
        }
    }
    println!("# Carga dos mapas originais");
    println!("arquivos: {} | carregados: {ok} | com erro: {}", names.len(), failed.len());
    for (n, e) in &failed { println!("ERRO {n}: {}", e.lines().next().unwrap_or("")); }
    let in_list = host.lua.load("return #MAP_LIST").eval::<i64>().unwrap_or(-1);
    println!("registrados em MAP_LIST por AddMap: {in_list}");
    println!("tamanhos: {sizes:?}");
    println!("casas listadas: {tiles}");
    println!("terreno por casa: {terrain:?}");
    println!("lava: {lava} | ácido: {acid} | fogo: {fire} | populated: {populated} | unidades criadas pelos mapas: {pending}");
    println!("chaves não aplicadas: {unsupported:?}");
    let r = host.report.borrow();
    println!("stubs tocados: {:?}", r.stub_calls.keys().collect::<Vec<_>>());
}
