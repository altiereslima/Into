//! `itb-diff <pasta-do-jogo> <diff_original.txt>`: roda a mesma coleta do jogo original no motor Rust e
//! compara as saídas. Ver `ferramentas/diferencial/`.
use itb_core::board_occupancy::{OccupancyBoard, Point};
use itb_core::rng::MsvcRand;
use itb_lua::{Host, HostConfig};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

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

/// Estado do LCG logo depois do último dos valores.
fn estado_apos(valores: &[i32]) -> Option<u32> {
    let estado = confere_rand(valores)?;
    let mut r = MsvcRand::new(estado);
    for _ in 1..valores.len() { r.rand(); }
    Some(r.seed())
}

fn valor<'a>(linhas: &'a Linhas, secao: &str, chave: &str) -> Option<&'a str> {
    linhas.get(&(secao.to_string(), chave.to_string())).map(String::as_str)
}

/// O que o jogo real passou às funções Lua da preparação (linhas `preparo|...`).
struct Preparo {
    mapa: String,
    setor: i32,
    tipo_setor: String,
    missao: String,
    missao_id: String,
}

fn preparo_original(linhas: &Linhas) -> Option<Preparo> {
    let mapa = valor(linhas, "missao", "mapa_preparo").or(valor(linhas, "preparo", "mapa")).unwrap_or("?").to_string();
    linhas.keys().find(|k| k.0 == "preparo")?;
    let chamada = |nome: &str| linhas.iter().find(|(k, _)| k.0 == "preparo" && k.1.ends_with(nome)).map(|(_, v)| v.as_str());
    let setor = chamada(".getBuildingHealth")?.split('=').next()?.parse().ok()?;
    let tipo_setor = chamada(".getEnvironmentChance")?.split(',').next()?.to_string();
    let (missao, missao_id) = match valor(linhas, "preparo", "IsCracksAllowed") {
        Some(v) => (v.rsplit_once('=')?.0.to_string(), valor(linhas, "preparo", "missao_id").unwrap_or("nil").to_string()),
        None => (String::new(), "nil".to_string()),
    };
    Some(Preparo { mapa, setor, tipo_setor, missao, missao_id })
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
    let com_tabuleiro = original.contains_key(&("missao".to_string(), "mapa".to_string()));
    let preparo = preparo_original(&original);
    let sincronias_preparo: Rc<RefCell<Vec<(String, bool)>>> = Rc::default();
    let mut nomes = Vec::new();
    if com_tabuleiro {
        host.new_game(Default::default()).expect("Game");
        nomes = host.load_map_data().expect("mapas");
        host.lua.globals().set("DIF_MAPA", "real").unwrap();
        // o mesmo estado de rand() que o original gravou antes do sorteio
        let sincronia: Vec<i32> = original.iter().filter(|(k, _)| k.0 == "sincronia").filter_map(|(_, v)| v.parse().ok()).collect();
        let rng = host.rng.clone();
        let sync = host.lua.create_function(move |_, ()| {
            if let Some(estado) = estado_apos(&sincronia) { rng.borrow_mut().srand(estado); }
            Ok(())
        }).unwrap();
        host.lua.globals().set("DIF_SINCRONIZA", sync).unwrap();
        // na preparação: confere se o Rust está no estado do original; se não, volta a ele
        let valores: BTreeMap<String, Vec<i32>> = original.iter().filter(|(k, _)| k.0 == "sincronia_preparo")
            .map(|(k, v)| (k.1.clone(), v.split(' ').filter_map(|x| x.parse().ok()).collect())).collect();
        let rng = host.rng.clone();
        let registro = sincronias_preparo.clone();
        let sync = host.lua.create_function(move |_, chave: String| {
            let Some(v) = valores.get(&chave) else {
                registro.borrow_mut().push((chave, false));
                return Ok(());
            };
            let mut r = rng.borrow().clone();
            let igual = v.iter().all(|&x| r.rand() == x);
            if igual { *rng.borrow_mut() = r; } else if let Some(estado) = estado_apos(v) { rng.borrow_mut().srand(estado); }
            registro.borrow_mut().push((chave, igual));
            Ok(())
        }).unwrap();
        host.lua.globals().set("DIF_SINCRONIZA_PREPARO", sync).unwrap();
    } else {
        host.lua.globals().set("DIF_SINCRONIZA", host.lua.create_function(|_, ()| Ok(())).unwrap()).unwrap();
    }
    if let Err(e) = host.lua.load(COLETA).set_name("@coleta.lua").exec() {
        println!("a coleta parou no motor Rust: {e}");
    }
    if let Some(coleta_missao) = host.lua.globals().get::<Option<mlua::Function>>("DIF_COLETA_MISSAO").unwrap() {
        let terreno = terreno_original(&original);
        let parecido = || {
            let mut melhor: Option<(usize, String)> = None;
            for n in &nomes {
                let dados = host.read_map_file(n).expect("mapa");
                let b = OccupancyBoard::from_map(&dados.data).expect("tabuleiro");
                let dif = terreno.iter().filter(|(p, t)| b.tile(**p).terrain != **t).count();
                if melhor.as_ref().is_none_or(|m| dif < m.0) { melhor = Some((dif, n.clone())); }
            }
            melhor.expect("há mapas").1
        };
        match &preparo {
            Some(pr) => {
                let mapa = if nomes.contains(&pr.mapa) { pr.mapa.clone() } else { parecido() };
                // CallMethod acha a missão em GAME.Missions, como no jogo
                if !pr.missao.is_empty() && pr.missao_id != "nil" {
                    let cria = host.lua.load("local obj, id = ...\nif GAME == nil then startNewGame() end\nGAME.Missions = GAME.Missions or {}\nGAME.Missions[GAME:GetMissionId(obj)] = CreateMission(id)");
                    if let Err(e) = cria.call::<()>((pr.missao.as_str(), pr.missao_id.as_str())) {
                        cabecalho.push(format!("a missão {} ({}) não foi criada no Rust: {}", pr.missao, pr.missao_id, e.to_string().lines().next().unwrap_or("")));
                    }
                }
                let (board, _) = host.load_mission(&mapa, pr.setor, &pr.tipo_setor, &pr.missao).expect("missão");
                let b = board.borrow();
                let chuva = b.weather.as_ref().map_or("não".to_string(), |w| format!("tipo {} subtipo {}", w.kind, w.subtype));
                cabecalho.push(format!("preparo pelo Rust: mapa {mapa} (jogo real: {}), setor {} \"{}\", missão {} ({}); chuva: {chuva}",
                    pr.mapa, pr.setor, pr.tipo_setor, pr.missao, pr.missao_id));
                let mudadas: Vec<String> = terreno.iter().filter(|(p, t)| b.tile(**p).terrain != **t)
                    .map(|(p, t)| format!("({},{}) Rust {} | original {t}", p.x, p.y, b.tile(*p).terrain)).collect();
                cabecalho.push(format!("casas com terreno diferente do jogo real depois do preparo: {} {mudadas:?}", mudadas.len()));
            }
            None => {
                // saída antiga, sem a preparação: acha o mapa mais parecido e parte do terreno que o jogo tinha
                let mapa = parecido();
                let (board, _) = host.load_map(&mapa).expect("mapa");
                let mudadas: Vec<String> = terreno.iter().filter(|(p, t)| board.borrow().tile(**p).terrain != **t)
                    .map(|(p, t)| format!("({},{}) {}→{}", p.x, p.y, board.borrow().tile(*p).terrain, t)).collect();
                for (p, t) in &terreno {
                    if board.borrow().tile(*p).terrain != *t { board.borrow_mut().set_terrain(*p, *t); }
                }
                cabecalho.push(format!("mapa do jogo real: {mapa}; casas com terreno diferente do arquivo: {} {mudadas:?}", mudadas.len()));
            }
        }
        if let Err(e) = coleta_missao.call::<()>(()) {
            println!("a coleta da missão parou no motor Rust: {e}");
        }
    }
    let sinc = sincronias_preparo.borrow();
    if !sinc.is_empty() {
        let fora: Vec<&str> = sinc.iter().filter(|s| !s.1).map(|s| s.0.as_str()).collect();
        cabecalho.push(format!("rand na preparação: {} pontos, {} fora de sincronia {fora:?}", sinc.len(), fora.len()));
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
        if matches!(k.0.as_str(), "rand" | "sincronia" | "sincronia_preparo" | "missao") || k.1 == "mapa" && k.0 == "preparo" { continue; }
        let e = resumo.entry(k.0.clone()).or_default();
        match rust.get(k) {
            Some(r) if r == v => e.0 += 1,
            Some(r) => { e.1 += 1; diferencas.push(format!("{}|{}: original {v:?} | Rust {r:?}", k.0, k.1)); }
            None => { e.2 += 1; diferencas.push(format!("{}|{}: original {v:?} | Rust não tem", k.0, k.1)); }
        }
    }
    for (k, r) in &rust {
        if !matches!(k.0.as_str(), "rand" | "sincronia" | "sincronia_preparo" | "missao") && !original.contains_key(k) {
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
