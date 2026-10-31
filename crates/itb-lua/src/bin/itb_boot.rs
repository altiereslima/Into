//! `itb-boot <pasta-do-jogo>`: carrega os scripts originais no host Rust e relata o que falta.
//! A pasta também pode vir de `ITB_GAME_DIR`.
use itb_lua::{Host, HostConfig, Outcome, Status};
use std::io::Write;

fn main() {
    let dir = std::env::args().nth(1).or_else(|| std::env::var("ITB_GAME_DIR").ok()).unwrap_or_else(|| {
        eprintln!("uso: itb-boot <pasta do Into the Breach>  (ou defina ITB_GAME_DIR)");
        std::process::exit(2);
    });
    let host = Host::new(&dir, HostConfig::default()).expect("falha ao montar o host Lua");
    if let Err(e) = host.boot() {
        eprintln!("boot interrompido: {e}");
    }
    let rep = host.report.borrow();
    let ok = rep.scripts.iter().filter(|(_, o)| *o == Outcome::Ok).count();
    let missing: Vec<_> = rep.scripts.iter().filter(|(_, o)| *o == Outcome::Missing).collect();
    let errors: Vec<_> = rep.scripts.iter().filter(|(_, o)| matches!(o, Outcome::Error(_))).collect();
    let mut out = std::io::stdout().lock();
    writeln!(out, "# Boot dos scripts originais").unwrap();
    writeln!(out, "arquivos: {} | ok: {} | ausentes: {} | com erro: {}", rep.scripts.len(), ok, missing.len(), errors.len()).unwrap();
    for (p, _) in &missing { writeln!(out, "AUSENTE  {p}").unwrap(); }
    for (p, o) in &errors {
        if let Outcome::Error(e) = o { writeln!(out, "ERRO     {p}").unwrap(); for l in e.lines().take(8) { writeln!(out, "         {l}").unwrap(); } }
    }
    let cov = host.coverage_counts();
    let total: usize = cov.values().sum();
    writeln!(out, "\n# Cobertura da API nativa ({total} nomes: métodos, propriedades, construtores, funções globais)").unwrap();
    for st in [Status::Reconstructed, Status::Host, Status::Stub] {
        let n = cov.get(&st).copied().unwrap_or(0);
        writeln!(out, "{:<14} {:>4}  ({:.1}%)", format!("{st:?}"), n, 100.0 * n as f64 / total as f64).unwrap();
    }
    writeln!(out, "\n# Stubs chamados durante o boot (prioridade de implementação)").unwrap();
    let mut calls: Vec<_> = rep.stub_calls.iter().collect();
    calls.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (k, n) in calls { writeln!(out, "{n:>6}  {k}").unwrap(); }
    if !rep.console.is_empty() {
        writeln!(out, "\n# ConsolePrint ({} linhas; primeiras 10)", rep.console.len()).unwrap();
        for l in rep.console.iter().take(10) { writeln!(out, "  {l}").unwrap(); }
    }
}
