//! `itb-diff <pasta-do-jogo> <diff_original.txt>`: roda a mesma coleta do jogo original no motor Rust e
//! compara as saídas. Ver `ferramentas/diferencial/`.
use itb_core::board_occupancy::{OccupancyBoard, Point};
use itb_core::rng::MsvcRand;
use itb_lua::{Host, HostConfig};
use std::collections::BTreeMap;

const COLETA: &str = include_str!("../../../../ferramentas/diferencial/coleta.lua");

type Linhas = BTreeMap<(String, String), String>;

fn ler(texto: &str) -> Linhas {
    let mut m = Linhas::new();
    for l in texto.lines() {
        if l.starts_with('#') { continue; }
        let mut p = l.splitn(3, '|');
        if let (Some(s), Some(k), Some(v)) = (p.next(), p.next(), p.next()) {
            if k.starts_with("DIF_") { continue; }
            m.insert((s.to_string(), k.to_string()), v.to_string());
        }
    }
    m
}

/// `random_int(32768)` expõe 15 bits do estado. Como o resto do estado cabe em 17 bits, dá para achar a
/// semente que explica o primeiro valor e conferir se ela gera a mesma sequência.
fn confere_rand(valores: &[i32]) -> Option<u32> {
    let primeiro = *valores.first()?;
    for alto in 0..2u32 {
        for baixo in 0..65536u32 {
            let estado = (alto << 31) | ((primeiro as u32) << 16) | baixo;
            let mut r = MsvcRand::new(estado);
            if valores[1..].iter().all(|&v| r.rand() == v) { return Some(estado); }
        }
    }
    None
}

/// Terreno que o jogo real tinha, das linhas `casa|x,y.terreno`.
fn terreno_original(linhas: &Linhas) -> BTreeMap<Point, i32> {
    linhas.iter().filter(|(k, _)| k.0 == "casa" && k.1.ends_with(".terreno")).filter_map(|(k, v)| {
        let (x, y) = k.1.strip_suffix(".terreno")?.split_once(',')?;
        Some((Point::new(x.parse().ok()?, y.parse().ok()?), v.parse().ok()?))
    }).collect()
}

fn rand_na_ordem(linhas: &Linhas) -> Vec<i32> {
    linhas.iter().filter(|(k, _)| k.0 == "rand").filter_map(|(_, v)| v.parse().ok()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: itb-diff <pasta do jogo> <diff_original.txt>");
        std::process::exit(2);
    }
    let bruto = std::fs::read(&args[2]).expect("ler a saída do jogo original");
    let original_texto = String::from_utf8_lossy(&bruto).to_string();
    let original = ler(&original_texto);

    // perfil do usuário nos testes: conteúdo novo desligado
    let host = Host::new(&args[1], HostConfig { advanced_content: false, ..Default::default() }).expect("host");
    host.boot().expect("boot");
    let mut cabecalho: Vec<String> = Vec::new();
    let saida = std::env::temp_dir().join(format!("itb_diff_{}.txt", std::process::id()));
    host.lua.globals().set("DIF_SAIDA", saida.to_string_lossy().replace('\\', "/")).unwrap();
    host.lua.globals().set("DIF_MODO", "rust").unwrap();
    // com tabuleiro: o mesmo mapa, e o mesmo estado de rand() que o original gravou antes do sorteio
    if original.contains_key(&("missao".to_string(), "mapa".to_string())) {
        host.new_game(Default::default()).expect("Game");
        // O motor escolhe o mapa e ainda mexe no terreno antes de BaseStart: acha o mais parecido e parte do
        // terreno que o jogo tinha, para comparar as regras sobre o mesmo tabuleiro.
        let terreno = terreno_original(&original);
        let nomes = host.load_map_data().expect("mapas");
        let mut melhor: Option<(usize, String)> = None;
        for n in &nomes {
            let dados = host.read_map_file(n).expect("mapa");
            let b = OccupancyBoard::from_map(&dados.data).expect("tabuleiro");
            let dif = terreno.iter().filter(|(p, t)| b.tile(**p).terrain != **t).count();
            if melhor.as_ref().is_none_or(|m| dif < m.0) { melhor = Some((dif, n.clone())); }
        }
        let (dif, mapa) = melhor.expect("há mapas");
        let (board, _) = host.load_map(&mapa).expect("mapa");
        let mudadas: Vec<String> = terreno.iter().filter(|(p, t)| board.borrow().tile(**p).terrain != **t)
            .map(|(p, t)| format!("({},{}) {}→{}", p.x, p.y, board.borrow().tile(*p).terrain, t)).collect();
        for (p, t) in &terreno {
            if board.borrow().tile(*p).terrain != *t { board.borrow_mut().set_terrain(*p, *t); }
        }
        cabecalho.push(format!("mapa do jogo real: {mapa}; casas com terreno diferente do arquivo: {dif} {mudadas:?}"));
        host.lua.globals().set("DIF_MAPA", "real").unwrap();
        let sincronia: Vec<i32> = original.iter().filter(|(k, _)| k.0 == "sincronia").filter_map(|(_, v)| v.parse().ok()).collect();
        let rng = host.rng.clone();
        let sync = host.lua.create_function(move |_, ()| {
            if let Some(estado) = confere_rand(&sincronia) {
                let mut r = MsvcRand::new(estado);
                for _ in 1..sincronia.len() { r.rand(); }
                rng.borrow_mut().srand(r.seed());
            }
            Ok(())
        }).unwrap();
        host.lua.globals().set("DIF_SINCRONIZA", sync).unwrap();
    } else {
        host.lua.globals().set("DIF_SINCRONIZA", host.lua.create_function(|_, ()| Ok(())).unwrap()).unwrap();
    }
    if let Err(e) = host.lua.load(COLETA).set_name("@coleta.lua").exec() {
        println!("a coleta parou no motor Rust: {e}");
    }
    let rust_texto = std::fs::read_to_string(&saida).unwrap_or_default();
    std::fs::remove_file(&saida).ok();
    let rust = ler(&rust_texto);

    println!("# Teste diferencial: jogo original x motor Rust");
    println!("linhas: original {} | Rust {}", original.len(), rust.len());
    for l in &cabecalho { println!("{l}"); }
    if let Some(ultima) = original_texto.lines().last() {
        if !ultima.starts_with("fim|") { println!("ATENÇÃO: a saída do original não chegou ao fim. Última linha: {ultima}"); }
    }

    let mut resumo: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let mut diferencas = Vec::new();
    for (k, v) in &original {
        if k.0 == "rand" || k.0 == "sincronia" || k.0 == "missao" { continue; }
        let e = resumo.entry(k.0.clone()).or_default();
        match rust.get(k) {
            Some(r) if r == v => e.0 += 1,
            Some(r) => { e.1 += 1; diferencas.push(format!("{}|{}: original {v:?} | Rust {r:?}", k.0, k.1)); }
            None => { e.2 += 1; diferencas.push(format!("{}|{}: original {v:?} | Rust não tem", k.0, k.1)); }
        }
    }
    for (k, r) in &rust {
        if k.0 != "rand" && k.0 != "missao" && !original.contains_key(k) {
            resumo.entry(k.0.clone()).or_default().3 += 1;
            diferencas.push(format!("{}|{}: Rust {r:?} | original não tem", k.0, k.1));
        }
    }
    println!("\n| Seção | Iguais | Diferentes | Só no original | Só no Rust |\n|---|---|---|---|---|");
    let mut total = (0, 0);
    for (s, (a, b, c, d)) in &resumo {
        println!("| {s} | {a} | {b} | {c} | {d} |");
        total.0 += a;
        total.1 += b + c + d;
    }
    println!("\nTotal: {} iguais, {} divergências.", total.0, total.1);

    let valores: Vec<i32> = rand_na_ordem(&original);
    if !valores.is_empty() {
        match confere_rand(&valores) {
            Some(estado) => println!("rand: os {} valores do original saem do LCG do Rust (estado {estado:#010x}).", valores.len()),
            None => println!("rand: os {} valores do original NÃO saem do LCG do Rust.", valores.len()),
        }
    }
    let host_rep = host.report.borrow();
    if !host_rep.stub_calls.is_empty() {
        let mut c: Vec<_> = host_rep.stub_calls.iter().collect();
        c.sort();
        println!("stubs tocados no Rust: {c:?}");
    }
    if !diferencas.is_empty() {
        println!("\n## Divergências");
        for d in &diferencas { println!("{d}"); }
    }
}
