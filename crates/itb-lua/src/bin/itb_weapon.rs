//! `itb-weapon <pasta-do-jogo> <Arma> [x1 y1 x2 y2]`
//!
//! Executa o código ORIGINAL de uma arma (`GetTargetArea` e `GetSkillEffect` dos scripts) sobre um
//! tabuleiro 8x8 do motor Rust: mech do jogador em (x1,y1), inimigo em (x2,y2), um prédio em (5,5).
//! Mostra a área de alvo e a lista de efeitos gerada — o que o motor original aplicaria.
use itb_core::board_occupancy::{consts::*, OccupancyBoard, PawnState, Point};
use itb_lua::records::{Field, Record};
use itb_lua::skill::{LPointList, LSkillEffect};
use itb_lua::{Host, HostConfig};
use mlua::{Function, Table, Value};
use std::cell::RefCell;
use std::rc::Rc;

fn fmt_damage(d: &Record) -> String {
    let mut parts = Vec::new();
    for (k, f) in d.fields.iter().chain(d.hidden.iter()) {
        let default = match (*k, f) {
            ("loc", _) => false,
            (_, Field::Int(0)) | (_, Field::Bool(false)) | (_, Field::Float(0.0)) => true,
            (_, Field::Str(s)) if s.is_empty() => true,
            (_, Field::Points(v)) if v.is_empty() => true,
            ("iPush", Field::Int(4)) | ("iPawnTeam", Field::Int(2)) | ("iTerrain", Field::Int(10)) => true,
            ("anim_param_0x34", Field::Int(2)) | ("i_0xc0", Field::Int(2)) | ("i_0x130", Field::Int(3)) => true,
            ("source_0x9c" | "p_0x110", Field::Point(p)) if p.x == -1 && p.y == -1 => true,
            _ => false,
        };
        if default { continue; }
        let v = match f {
            Field::Int(i) => i.to_string(),
            Field::Float(x) => x.to_string(),
            Field::Bool(b) => b.to_string(),
            Field::Str(s) => format!("{s:?}"),
            Field::Point(p) => format!("({},{})", p.x, p.y),
            Field::Points(v) => v.iter().map(|p| format!("({},{})", p.x, p.y)).collect::<Vec<_>>().join("→"),
        };
        parts.push(format!("{k}={v}"));
    }
    parts.join("  ")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: itb-weapon <pasta do jogo> <Arma> [x1 y1 x2 y2]   (ex.: Prime_Punchmech 2 4 2 3)");
        std::process::exit(2);
    }
    let n = |i: usize, d: i32| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (p1, p2) = (Point::new(n(3, 3), n(4, 4)), Point::new(n(5, 3), n(6, 3)));
    let host = Host::new(&args[1], HostConfig::default()).expect("host");
    host.boot().expect("boot");

    let mut b = OccupancyBoard::new(8, 8).unwrap();
    b.tile_mut(Point::new(5, 5)).unwrap().terrain = TERRAIN_BUILDING;
    let mech = b.add_pawn(PawnState { id: 0, team: TEAM_PLAYER, health: 3, max_health: 3, space: p1, mech: true, ..Default::default() }).unwrap();
    if p2 != p1 {
        b.add_pawn(PawnState { id: 1, team: TEAM_ENEMY, health: 3, max_health: 3, space: p2, ..Default::default() }).unwrap();
    }
    let board = Rc::new(RefCell::new(b));
    host.set_board(board.clone()).unwrap();
    host.set_pawn(&board, mech).unwrap();

    if args[2] == "--todas" {
        return sweep(&host, &board, mech, p1, p2);
    }
    let weapon: Table = match host.lua.globals().get::<Value>(args[2].as_str()) {
        Ok(Value::Table(t)) => t,
        _ => { eprintln!("arma {:?} não existe nos scripts", args[2]); std::process::exit(1); }
    };
    let lp = |p: Point| itb_lua::values::LPoint(itb_core::geometry::IPoint::new(p.x, p.y));
    println!("# {}  — mech em ({},{}), inimigo em ({},{})", args[2], p1.x, p1.y, p2.x, p2.y);

    let area: mlua::Result<Value> = weapon.get::<Function>("GetTargetArea").and_then(|f| f.call((weapon.clone(), lp(p1))));
    match area {
        Ok(Value::UserData(u)) => {
            let l = u.borrow::<LPointList>().unwrap();
            println!("GetTargetArea: {} casas: {}", l.0.len(),
                     l.0.iter().map(|p| format!("({},{})", p.x, p.y)).collect::<Vec<_>>().join(" "));
        }
        Ok(other) => println!("GetTargetArea devolveu {}", other.type_name()),
        Err(e) => println!("GetTargetArea falhou: {}", e.to_string().lines().next().unwrap_or("")),
    }

    let fx: mlua::Result<Value> = weapon.get::<Function>("GetSkillEffect").and_then(|f| f.call((weapon.clone(), lp(p1), lp(p2))));
    match fx {
        Ok(Value::UserData(u)) => {
            let s = u.borrow::<LSkillEffect>().unwrap();
            for (name, list) in [("effect", &s.effect), ("q_effect", &s.q_effect)] {
                let l = list.borrow();
                println!("{name}: {} itens", l.len());
                for (i, d) in l.iter().enumerate() {
                    println!("  [{}] {}", i + 1, fmt_damage(d));
                }
            }
        }
        Ok(other) => println!("GetSkillEffect devolveu {}", other.type_name()),
        Err(e) => {
            println!("GetSkillEffect falhou:");
            for l in e.to_string().lines().take(6) { println!("  {l}"); }
        }
    }
    let rep = host.report.borrow();
    let mut stubs: Vec<_> = rep.stub_calls.iter().filter(|(k, _)| *k != "GetMechColorsInTexture()").collect();
    stubs.sort();
    if !stubs.is_empty() {
        println!("stubs chamados: {}", stubs.iter().map(|(k, n)| format!("{k}×{n}")).collect::<Vec<_>>().join(", "));
    }
}

/// Executa GetTargetArea + GetSkillEffect de TODAS as armas definidas nos scripts e resume.
fn sweep(host: &Host, board: &Rc<RefCell<OccupancyBoard>>, mech: usize, p1: Point, p2: Point) {
    let names: Vec<String> = host.lua.load(r#"
        local t = {}
        for k, v in pairs(_G) do
            if type(k) == "string" and type(v) == "table" and type(v.GetSkillEffect) == "function"
               and type(v.GetTargetArea) == "function" and k ~= "Skill" then t[#t + 1] = k end
        end
        table.sort(t) return t"#).eval().unwrap();
    let lp = |p: Point| itb_lua::values::LPoint(itb_core::geometry::IPoint::new(p.x, p.y));
    let (mut clean, mut with_stubs, mut failed) = (0, 0, Vec::new());
    let mut stub_freq: std::collections::BTreeMap<String, usize> = Default::default();
    for name in &names {
        let before: std::collections::BTreeMap<String, u64> = host.report.borrow().stub_calls.clone();
        host.set_board(board.clone()).unwrap();
        host.set_pawn(board, mech).unwrap();
        let w: Table = host.lua.globals().get(name.as_str()).unwrap();
        let r1 = w.get::<Function>("GetTargetArea").and_then(|f| f.call::<Value>((w.clone(), lp(p1))));
        let r2 = w.get::<Function>("GetSkillEffect").and_then(|f| f.call::<Value>((w.clone(), lp(p1), lp(p2))));
        let after = host.report.borrow().stub_calls.clone();
        let new_stubs: Vec<String> = after.iter().filter(|(k, n)| before.get(*k) != Some(n)).map(|(k, _)| k.clone()).collect();
        match (r1, r2) {
            (Ok(_), Ok(_)) if new_stubs.is_empty() => clean += 1,
            (Ok(_), Ok(_)) => with_stubs += 1,
            (a, b) => failed.push((name.clone(), a.err().or(b.err()).map(|e| e.to_string().lines().next().unwrap_or("").to_string()).unwrap_or_default())),
        }
        for k in new_stubs { *stub_freq.entry(k).or_default() += 1; }
    }
    println!("# Varredura de armas ({} encontradas nos scripts)", names.len());
    println!("sem stubs e sem erro: {clean} | rodaram tocando stubs: {with_stubs} | com erro: {}", failed.len());
    let mut freq: Vec<_> = stub_freq.into_iter().collect();
    freq.sort_by_key(|e| std::cmp::Reverse(e.1));
    println!("\n## Stubs que mais bloqueiam armas (nº de armas)");
    for (k, n) in freq.iter().take(25) { println!("{n:>4}  {k}"); }
    let mut errs: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (n, e) in &failed { errs.entry(e.chars().take(110).collect()).or_default().push(n.clone()); }
    println!("\n## Erros (agrupados)");
    for (e, ns) in errs { println!("{:>4}  {e}   ex.: {}", ns.len(), ns.iter().take(3).cloned().collect::<Vec<_>>().join(", ")); }
}
