//! `itb-uso <pasta-do-jogo> [Mission_X ...]`: joga missões no motor Rust, chamando os pontos de entrada que o motor
//! original usa na batalha, e conta os stubs tocados em cada fase. Não é uma reconstrução do laço de jogo: a ordem e o
//! número de chamadas são aproximados, para dizer o que falta reconstruir e com que prioridade.
//!
//! Por missão: `CreateMission`, mapa por `Mission:GetMap()` ou `RandomMap(GetMapTag())`, `Host::load_mission`,
//! `BaseStart`, 3 mechs, `BaseDeployment` e 4 turnos (`BaseNextTurn`, `UpdateSpawning`, `SpawnQueued`, IA dos inimigos,
//! armas dos mechs, ambiente, `BaseUpdate`, objetivos), depois `MissionEnd`.
use itb_lua::{Host, HostConfig};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

const ROTEIRO: &str = r#"
local tipo = ...
local function chama(fase, f, ...)
	USO_FASE(fase)
	local r = { pcall(f, ...) }
	if not r[1] then USO_ERRO(fase, tostring(r[2])) end
	return r[1], r[2], r[3]
end
local function lista(l)
	local t = {}
	if l == nil then return t end
	for i = 1, l:size() do t[i] = l:index(i) end
	return t
end
local M = GAME.Missions[1]
chama("BaseStart", M.BaseStart, M)
for _, tipo_mech in ipairs({ "PunchMech", "TankMech", "ArtiMech" }) do
	chama("posicionamento", function() Board:AddPawn(PAWN_FACTORY:CreatePawn(tipo_mech)) end)
end
chama("BaseDeployment", M.BaseDeployment, M)

local function armas(pawn)
	local def = _G[pawn:GetType()]
	return (def and def.SkillList) or {}
end

local function age(fase, pawn, pontua)
	SetPawn(pawn)
	local origem = pawn:GetSpace()
	if pontua then
		local ok, alcance = chama(fase .. ":GetReachable", function() return Board:GetReachable(origem, pawn:GetMoveSpeed(), pawn:GetPathProf()) end)
		if ok then
			for _, p in ipairs(lista(alcance)) do chama(fase .. ":ScorePositioning", ScorePositioning, p, pawn) end
		end
	end
	for _, nome in ipairs(armas(pawn)) do
		local w = _G[nome]
		if type(w) == "table" and w.GetTargetArea then
			local ok, area = chama(fase .. ":GetTargetArea", w.GetTargetArea, w, origem)
			local alvos = ok and lista(area) or {}
			local melhor, nota = nil, -1e9
			for _, alvo in ipairs(alvos) do
				if pontua then
					local ok2, s = chama(fase .. ":GetTargetScore", w.GetTargetScore, w, origem, alvo)
					if ok2 and type(s) == "number" and s > nota then melhor, nota = alvo, s end
				elseif melhor == nil then
					melhor = alvo
				end
			end
			if melhor then
				local ok3, fx = chama(fase .. ":GetSkillEffect", w.GetSkillEffect, w, origem, melhor)
				if ok3 and fx then chama(fase .. ":AddEffect", function() Board:AddEffect(fx) end) end
			end
		end
	end
end

for turno = 1, 4 do
	chama("BaseNextTurn", M.BaseNextTurn, M)
	chama("UpdateSpawning", M.UpdateSpawning, M)
	chama("SpawnQueued", function() Board:SpawnQueued() end)
	local ok, ids = chama("inimigos", function() return Board:GetPawns(TEAM_ENEMY) end)
	for _, id in ipairs(ok and lista(ids) or {}) do
		local pawn = Board:GetPawn(id)
		if pawn and not pawn:IsDead() then age("inimigo", pawn, true) end
	end
	local ok2, mechs = chama("mechs", function() return Board:GetPawns(TEAM_PLAYER) end)
	for _, id in ipairs(ok2 and lista(mechs) or {}) do
		local pawn = Board:GetPawn(id)
		if pawn and not pawn:IsDead() then age("mech", pawn, false) end
	end
	local ok3, amb = chama("IsEnvironmentEffect", M.IsEnvironmentEffect, M)
	if ok3 and amb then
		chama("PlanEnvironment", M.PlanEnvironment, M)
		chama("ApplyEnvironmentEffect", M.ApplyEnvironmentEffect, M)
	end
	chama("BaseUpdate", M.BaseUpdate, M)
	chama("IsEndBlocked", M.IsEndBlocked, M)
	chama("BaseObjectives", M.BaseObjectives, M)
end
chama("MissionEnd", M.MissionEnd, M)
chama("BaseCompletedObjectives", M.BaseCompletedObjectives, M)
"#;

#[derive(Default)]
struct Uso {
    fase: String,
    /// (stub, fase) → chamadas
    por_fase: BTreeMap<(String, String), u64>,
    /// stub → missões que o tocaram
    missoes: BTreeMap<String, BTreeSet<String>>,
    /// (fase, mensagem) → vezes
    erros: BTreeMap<(String, String), u64>,
    antes: BTreeMap<String, u64>,
    missao: String,
}

fn fecha_fase(uso: &mut Uso, agora: &BTreeMap<String, u64>) {
    for (k, &v) in agora {
        let d = v - uso.antes.get(k).copied().unwrap_or(0);
        if d > 0 {
            *uso.por_fase.entry((k.clone(), uso.fase.clone())).or_default() += d;
            uso.missoes.entry(k.clone()).or_default().insert(uso.missao.clone());
        }
    }
    uso.antes = agora.clone();
}

fn resume(e: &str) -> String {
    let l = e.lines().next().unwrap_or("").to_string();
    // tira o caminho e a linha do script, que variam
    match l.find(": ") { Some(i) if l[..i].contains(".lua:") => l[i + 2..].to_string(), _ => l }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("uso: itb-uso <pasta do jogo> [Mission_X ...]");
        std::process::exit(2);
    }
    let host = Host::new(&args[1], HostConfig { advanced_content: false, ..Default::default() }).expect("host");
    host.boot().expect("boot");
    host.new_game(Default::default()).expect("Game");
    host.lua.load("startNewGame()").exec().expect("startNewGame");
    host.load_map_data().expect("mapas");
    let mut tipos: Vec<String> = args[2..].to_vec();
    if tipos.is_empty() {
        tipos = host.lua.load(r#"local t = {} for k, v in pairs(_G) do
            if type(k) == "string" and k:match("^Mission_") and type(v) == "table" and v.BaseStart and k ~= "Mission_ActiveTutorial" then t[#t+1] = k end
            end table.sort(t) return t"#).eval().expect("tipos de missão");
    }
    let uso = Rc::new(RefCell::new(Uso::default()));
    let (u, rep) = (uso.clone(), host.report.clone());
    host.lua.globals().set("USO_FASE", host.lua.create_function(move |_, fase: String| {
        let agora = rep.borrow().stub_calls.clone();
        let mut u = u.borrow_mut();
        fecha_fase(&mut u, &agora);
        u.fase = fase;
        Ok(())
    }).unwrap()).unwrap();
    let u = uso.clone();
    host.lua.globals().set("USO_ERRO", host.lua.create_function(move |_, (fase, msg): (String, String)| {
        *u.borrow_mut().erros.entry((fase, resume(&msg))).or_default() += 1;
        Ok(())
    }).unwrap()).unwrap();

    let mut resumo = Vec::new();
    for tipo in &tipos {
        {
            let agora = host.report.borrow().stub_calls.clone();
            let mut u = uso.borrow_mut();
            u.missao = tipo.clone();
            u.fase = "criação e mapa".into();
            u.antes = agora;
        }
        let cria = host.lua.load(r#"local tipo = ...
            GAME.Missions = { [1] = CreateMission(tipo) }
            local m = GAME.Missions[1]
            local ok, nome = pcall(function() if m.MapList and #m.MapList > 0 then return m:GetMap() end return RandomMap(m:GetMapTag(), 1) end)
            if ok and type(nome) == "string" and _G[nome] then return nome end
            return "any1""#).call::<String>(tipo.as_str());
        let mapa = match cria {
            Ok(m) => m,
            Err(e) => { resumo.push(format!("{tipo}: missão não criada: {}", resume(&e.to_string()))); continue; }
        };
        if let Err(e) = host.load_mission(&mapa, 0, "grass", "Mission1") {
            resumo.push(format!("{tipo}: {mapa} não preparado: {}", resume(&e.to_string())));
            continue;
        }
        if let Err(e) = host.lua.load(ROTEIRO).set_name("@itb-uso").call::<()>(tipo.as_str()) {
            resumo.push(format!("{tipo}: roteiro parou: {}", resume(&e.to_string())));
        }
        let agora = host.report.borrow().stub_calls.clone();
        fecha_fase(&mut uso.borrow_mut(), &agora);
        resumo.push(format!("{tipo}: mapa {mapa}"));
    }

    let u = uso.borrow();
    let mut total: BTreeMap<String, (u64, BTreeSet<String>)> = BTreeMap::new();
    for ((stub, fase), n) in &u.por_fase {
        let e = total.entry(stub.clone()).or_default();
        e.0 += n;
        e.1.insert(fase.clone());
    }
    let mut ordem: Vec<_> = total.iter().collect();
    ordem.sort_by(|a, b| u.missoes[b.0].len().cmp(&u.missoes[a.0].len()).then(b.1 .0.cmp(&a.1 .0)).then(a.0.cmp(b.0)));
    println!("# Stubs tocados ao jogar missões no motor Rust ({} tipos de missão)\n", tipos.len());
    println!("Ordem: em quantas missões aparece, depois total de chamadas.\n");
    println!("| Stub | Missões | Chamadas | Fases |\n|---|---|---|---|");
    for (stub, (n, fases)) in &ordem {
        let f: Vec<&str> = fases.iter().map(String::as_str).collect();
        println!("| `{stub}` | {} | {n} | {} |", u.missoes[*stub].len(), f.join(", "));
    }
    let mut erros: Vec<_> = u.erros.iter().collect();
    erros.sort_by(|a, b| b.1.cmp(a.1));
    println!("\n## Erros de script (fase, mensagem, vezes)\n");
    for ((fase, msg), n) in erros.iter().take(60) { println!("- {fase}: {msg} ({n})"); }
    println!("\n## Missões\n");
    for l in &resumo { println!("- {l}"); }
}
