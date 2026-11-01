//! Host Lua 5.1: monta `_G` com a API nativa (real ou stub) e carrega os scripts ORIGINAIS
//! da instalação do usuário, na mesma ordem do motor (`scripts/scripts.lua` → `GetScripts()`).

use crate::api_manifest::{Const, CLASSES, CONSTANTS, GLOBAL_FUNCTIONS, INSTANCES};
use crate::game::SharedGame;
use crate::values::{arg_int, construct_color, construct_point, construct_rect, construct_button, no_overload, LColor, LPoint};
use itb_core::game::GameState;
use itb_core::geometry::{get_direction, Color, IPoint};
use itb_core::rng::MsvcRand;
use mlua::{Function, Lua, MultiValue, Table, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Origem da implementação de cada nome da API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Comportamento reconstruído do EXE, com teste.
    Reconstructed,
    /// Resposta de plataforma/configuração do host (tela, idioma, diretório...).
    Host,
    /// Ainda não implementado: registra a chamada e devolve `nil`.
    Stub,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Missing,
    Error(String),
}

#[derive(Default, Debug)]
pub struct Report {
    /// Chamadas a stubs durante a execução: "Classe:metodo" / "funcao()" → contagem.
    pub stub_calls: BTreeMap<String, u64>,
    pub console: Vec<String>,
    pub scripts: Vec<(String, Outcome)>,
    /// Estado de cada nome da API: "Classe:metodo", "Classe()", "Classe.prop", "funcao()".
    pub coverage: BTreeMap<String, Status>,
}

/// Respostas de plataforma. Os valores padrão imitam um PC em 1280x720, dificuldade normal,
/// conteúdo Advanced Edition ligado.
#[derive(Clone, Debug)]
pub struct HostConfig {
    /// Janela física; o tamanho virtual vem de `itb_core::display` (fórmulas do EXE).
    pub window_w: i32,
    pub window_h: i32,
    pub difficulty: i32,
    pub advanced_content: bool,
    pub rand_seed: u32,
    /// Estado global lido por `IsPassiveSkill` (0x0065AEB0): string em 0x008D0E94 (prefixo forçado)
    /// e nomes para os quais algum objeto da lista 0x008D0E48 responde `0x00625320(nome)`.
    pub forced_passive: String,
    pub passive_skills: Vec<String>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self { window_w: 1280, window_h: 720, difficulty: 1, advanced_content: true, rand_seed: 1,
               forced_passive: String::new(), passive_skills: Vec::new() }
    }
}

pub struct Host {
    pub lua: Lua,
    pub report: Rc<RefCell<Report>>,
    pub game_dir: PathBuf,
    pub rng: Rc<RefCell<MsvcRand>>,
}

fn stub_fn(lua: &Lua, report: &Rc<RefCell<Report>>, key: String) -> mlua::Result<Function> {
    let rep = report.clone();
    lua.create_function(move |_, _: MultiValue| {
        *rep.borrow_mut().stub_calls.entry(key.clone()).or_default() += 1;
        Ok(Value::Nil)
    })
}

impl Host {
    pub fn new(game_dir: impl AsRef<Path>, cfg: HostConfig) -> mlua::Result<Self> {
        let host = Self {
            lua: Lua::new(),
            report: Rc::new(RefCell::new(Report::default())),
            game_dir: game_dir.as_ref().to_path_buf(),
            rng: Rc::new(RefCell::new(MsvcRand::new(cfg.rand_seed))),
        };
        host.lua.set_app_data(host.rng.clone());
        host.install_constants()?;
        host.install_stub_classes()?;
        host.install_stub_globals()?;
        host.install_values()?;
        host.install_globals(&cfg)?;
        host.install_instances()?;
        Ok(host)
    }

    fn mark(&self, key: impl Into<String>, st: Status) {
        self.report.borrow_mut().coverage.insert(key.into(), st);
    }

    fn install_constants(&self) -> mlua::Result<()> {
        let g = self.lua.globals();
        for (name, v) in CONSTANTS {
            let val = match *v {
                Const::Int(i) => Value::Number(i as f64),
                Const::Bool(b) => Value::Boolean(b),
                Const::Point(x, y) => Value::UserData(self.lua.create_userdata(LPoint(IPoint::new(x, y)))?),
                Const::Color(r, gg, b, a) => Value::UserData(self.lua.create_userdata(LColor(Color::rgba(r, gg, b, a)))?),
            };
            g.set(*name, val)?;
        }
        Ok(())
    }

    /// Toda classe vira uma tabela "chamável" cujas instâncias têm os métodos como stubs.
    fn install_stub_classes(&self) -> mlua::Result<()> {
        let g = self.lua.globals();
        for c in CLASSES {
            let methods = self.lua.create_table()?;
            for m in c.methods {
                let key = format!("{}:{}", c.name, m.name);
                methods.set(m.name, stub_fn(&self.lua, &self.report, key.clone())?)?;
                self.mark(key, Status::Stub);
            }
            for p in c.properties {
                self.mark(format!("{}.{}", c.name, p), Status::Stub);
            }
            let inst_mt = self.lua.create_table()?;
            inst_mt.set("__index", methods.clone())?;
            inst_mt.set("__luabind_class", c.name)?;
            let class_tbl = self.lua.create_table()?;
            let class_mt = self.lua.create_table()?;
            let rep = self.report.clone();
            let key = format!("{}()", c.name);
            let inst_mt2 = inst_mt.clone();
            class_mt.set("__call", self.lua.create_function(move |lua, _: MultiValue| {
                *rep.borrow_mut().stub_calls.entry(key.clone()).or_default() += 1;
                let t = lua.create_table()?;
                t.set_metatable(Some(inst_mt2.clone()))?;
                Ok(t)
            })?)?;
            class_tbl.set_metatable(Some(class_mt))?;
            self.lua.set_named_registry_value(&format!("itb_methods_{}", c.name), methods.clone())?;
            class_tbl.set("__methods", methods)?;
            class_tbl.set("__instance_mt", inst_mt)?;
            if c.constructors > 0 {
                self.mark(format!("{}()", c.name), Status::Stub);
            }
            g.set(c.name, class_tbl)?;
        }
        Ok(())
    }

    fn install_stub_globals(&self) -> mlua::Result<()> {
        let g = self.lua.globals();
        for f in GLOBAL_FUNCTIONS {
            let key = format!("{}()", f.name);
            g.set(f.name, stub_fn(&self.lua, &self.report, key.clone())?)?;
            self.mark(key, Status::Stub);
        }
        Ok(())
    }

    /// Point e GL_Color reais (userdata) no lugar dos stubs.
    fn install_values(&self) -> mlua::Result<()> {
        let g = self.lua.globals();
        let point = self.lua.create_table()?;
        let mt = self.lua.create_table()?;
        mt.set("__call", self.lua.create_function(|_, args: MultiValue| {
            let mut it = args.into_iter();
            it.next(); // a própria tabela da classe
            construct_point(&it.collect())
        })?)?;
        point.set_metatable(Some(mt))?;
        g.set("Point", point)?;
        for k in ["Point()", "Point:__add", "Point:__sub", "Point:__eq", "Point:__mul", "Point:Manhattan",
                  "Point:GetString", "Point:GetLuaString", "Point.x", "Point.y"] {
            self.mark(k, Status::Reconstructed);
        }
        self.value_class("Rect2D", construct_rect)?;
        for k in ["Rect2D()", "Rect2D.x", "Rect2D.y", "Rect2D.w", "Rect2D.h"] {
            self.mark(k, Status::Reconstructed);
        }
        self.value_class("Button", construct_button)?;
        for k in ["Button()", "Button.image", "Button.pos", "Button.hitstats", "Button.label"] {
            self.mark(k, Status::Reconstructed);
        }
        for rc in crate::records::RECORD_CLASSES {
            let cls = self.lua.create_table()?;
            let mt = self.lua.create_table()?;
            mt.set("__call", self.lua.create_function(move |_, args: MultiValue| {
                let rest: MultiValue = args.into_iter().skip(1).collect();
                crate::records::construct(rc, &rest)
            })?)?;
            cls.set_metatable(Some(mt))?;
            self.lua.globals().set(rc.name, cls)?;
            self.mark(format!("{}()", rc.name), Status::Reconstructed);
            for p in rc.properties {
                self.mark(format!("{}.{}", rc.name, p), Status::Reconstructed);
            }
        }
        // SkillEffect, listas e métodos reais de SpaceDamage (Etapa 08)
        use crate::skill::*;
        install_space_damage_methods(&self.lua)?;
        crate::records::install_objective_methods(&self.lua)?;
        self.mark("Objective:Failed", Status::Reconstructed);
        for k in ["SpaceDamage:IsMovement", "SpaceDamage:MoveStart", "SpaceDamage:MoveEnd"] {
            self.mark(k, Status::Reconstructed);
        }
        self.value_class("SkillEffect", |a: &MultiValue| if a.is_empty() { Ok(LSkillEffect::default()) } else { Err(no_overload("SkillEffect()")) })?;
        self.value_class("DamageList", |a: &MultiValue| if a.is_empty() { Ok(LDamageList::new()) } else { Err(no_overload("DamageList()")) })?;
        self.value_class("PointList", |a: &MultiValue| if a.is_empty() { Ok(LPointList::default()) } else { Err(no_overload("PointList()")) })?;
        self.value_class("IntList", |a: &MultiValue| if a.is_empty() { Ok(LIntList::default()) } else { Err(no_overload("IntList()")) })?;
        self.value_class("UIntList", |a: &MultiValue| if a.is_empty() { Ok(LUIntList::default()) } else { Err(no_overload("UIntList()")) })?;
        for k in SKILL_EFFECT_DONE {
            self.mark(format!("SkillEffect:{k}"), Status::Reconstructed);
        }
        for k in ["SkillEffect()", "SkillEffect.effect", "SkillEffect.q_effect", "SkillEffect.impact_sound", "SkillEffect.piOrigin",
                  "SkillEffect.iOwner", "DamageList()", "PointList()", "IntList()", "UIntList()"] {
            self.mark(k, Status::Reconstructed);
        }
        for (cls, ms) in [("DamageList", &["size", "index", "push_back", "back", "empty"][..]),
                          ("PointList", &["size", "index", "push_back", "back", "erase", "empty"][..]),
                          ("IntList", &["size", "index", "push_back", "back", "erase", "empty"][..]),
                          ("UIntList", &["size", "index", "push_back", "back"][..])] {
            for m in ms {
                self.mark(format!("{cls}:{m}"), Status::Reconstructed);
            }
        }
        for k in crate::world::BOARD_DONE { self.mark(format!("Board:{k}"), Status::Reconstructed); }
        for k in crate::world::BOARD_PARTIAL { self.mark(format!("Board:{k}"), Status::Reconstructed); }
        for k in crate::world::PAWN_DONE { self.mark(format!("BoardPawn:{k}"), Status::Reconstructed); }
        for k in crate::world::PAWN_HOST { self.mark(format!("BoardPawn:{k}"), Status::Host); }
        for k in crate::game::GAME_DONE { self.mark(format!("GameMap:{k}"), Status::Reconstructed); }
        for k in ["GetValue", "GetMax"] { self.mark(format!("ValueBar:{k}"), Status::Reconstructed); }
        let color = self.lua.create_table()?;
        let mt = self.lua.create_table()?;
        mt.set("__call", self.lua.create_function(|_, args: MultiValue| {
            let mut it = args.into_iter();
            it.next();
            construct_color(&it.collect())
        })?)?;
        color.set_metatable(Some(mt))?;
        g.set("GL_Color", color)?;
        for k in ["GL_Color()", "GL_Color:Normalize", "GL_Color:WithAlpha", "GL_Color.r", "GL_Color.g", "GL_Color.b", "GL_Color.a"] {
            self.mark(k, Status::Reconstructed);
        }
        Ok(())
    }

    /// Classe de valor: tabela global chamável que constrói userdata com `ctor(args)`.
    fn value_class<T, F>(&self, name: &str, ctor: F) -> mlua::Result<()>
    where
        T: mlua::IntoLua,
        F: Fn(&MultiValue) -> mlua::Result<T> + 'static,
    {
        let cls = self.lua.create_table()?;
        let mt = self.lua.create_table()?;
        mt.set("__call", self.lua.create_function(move |_, args: MultiValue| {
            let rest: MultiValue = args.into_iter().skip(1).collect();
            ctor(&rest)
        })?)?;
        cls.set_metatable(Some(mt))?;
        self.lua.globals().set(name, cls)
    }

    fn set_fn<A, R, F>(&self, name: &str, st: Status, f: F) -> mlua::Result<()>
    where
        A: mlua::FromLuaMulti,
        R: mlua::IntoLuaMulti,
        F: Fn(&Lua, A) -> mlua::Result<R> + 'static,
    {
        self.lua.globals().set(name, self.lua.create_function(f)?)?;
        self.mark(format!("{name}()"), st);
        Ok(())
    }

    fn install_globals(&self, cfg: &HostConfig) -> mlua::Result<()> {
        use Status::*;
        // --- reconstruídas do EXE
        self.set_fn("GetDirection", Reconstructed, |_, p: LPoint| Ok(get_direction(p.0)))?;
        let rng = self.rng.clone();
        self.set_fn("random_int", Reconstructed, move |_, args: MultiValue| {
            let a: Vec<Option<i32>> = args.iter().map(arg_int).collect();
            match a.as_slice() {
                [Some(n)] => Ok(rng.borrow_mut().random_int(*n)),
                [Some(x), Some(y)] => Ok(rng.borrow_mut().random_int_range(*x, *y)),
                _ => Err(no_overload("random_int(int), random_int(int,int)")),
            }
        })?;
        let rng = self.rng.clone();
        self.set_fn("random_bool", Reconstructed, move |_, args: MultiValue| {
            let a: Vec<Option<i32>> = args.iter().map(arg_int).collect();
            let r = match a.as_slice() {
                [Some(n)] => rng.borrow_mut().random_bool(*n),
                [Some(x), Some(y)] => rng.borrow_mut().random_bool_chance(*x, *y),
                _ => return Err(no_overload("random_bool(int), random_bool(int,int)")),
            };
            r.map_err(|_| mlua::Error::RuntimeError("random_bool: divisão inteira por zero (o EXE original travaria)".into()))
        })?;
        // --- respostas de plataforma
        let d = itb_core::display::Display::from_window(cfg.window_w, cfg.window_h);
        let (diff, adv) = (cfg.difficulty, cfg.advanced_content);
        self.set_fn("ScreenSizeX", Reconstructed, move |_, ()| Ok(d.screen_size_x()))?;
        self.set_fn("ScreenSizeY", Reconstructed, move |_, ()| Ok(d.screen_size_y()))?;
        self.set_fn("GetDefaultScale", Reconstructed, move |_, ()| Ok(d.default_scale()))?;
        self.set_fn("GetBoardScale", Reconstructed, move |_, ()| Ok(d.board_scale()))?;
        self.set_fn("GetDifficulty", Host, move |_, ()| Ok(diff))?;
        self.set_fn("IsRelease", Host, |_, ()| Ok(true))?;
        for n in ["IsPhoneUI", "IsTouch", "IsGamepad", "IsLargeFont"] {
            self.set_fn(n, Host, |_, ()| Ok(false))?;
        }
        for n in ["IsNewMissions", "IsNewEquipment", "IsNewEnemies"] {
            self.set_fn(n, Host, move |_, ()| Ok(adv))?;
        }
        let mut wd = self.game_dir.to_string_lossy().replace('\\', "/");
        if !wd.ends_with('/') { wd.push('/'); }
        self.set_fn("GetWorkingDir", Host, move |_, ()| Ok(wd.clone()))?;
        // 0x00656480 SoundEffect(Point, string): SpaceDamage(p) com sSound
        self.set_fn("SoundEffect", Reconstructed, |_, (p, snd): (LPoint, String)| {
            let mut d = crate::records::space_damage(&[]).expect("SpaceDamage()");
            d.set("loc", crate::records::Field::Point(p.0));
            d.set("sSound", crate::records::Field::Str(snd));
            Ok(d)
        })?;
        // 0x0065AEB0 IsPassiveSkill(string): a string global começa com o nome, ou algum objeto
        // da lista global aceita o nome. O estado vem da configuração do host.
        let (forced, passives) = (cfg.forced_passive.clone(), cfg.passive_skills.clone());
        self.set_fn("IsPassiveSkill", Reconstructed, move |_, name: Value| {
            let n = crate::records::arg_str(&name).ok_or_else(|| no_overload("IsPassiveSkill(string)"))?;
            Ok((forced.len() >= n.len() && forced.as_bytes()[..n.len()] == *n.as_bytes()) || passives.contains(&n))
        })?;
        let rep = self.report.clone();
        self.set_fn("ConsolePrint", Host, move |_, s: String| { rep.borrow_mut().console.push(s); Ok(()) })?;
        Ok(())
    }

    /// Instâncias globais. `PAWN_FACTORY` existe desde o registro; `Board`/`Game` só em missão.
    fn install_instances(&self) -> mlua::Result<()> {
        let g = self.lua.globals();
        for (name, class) in INSTANCES {
            if *name == "PAWN_FACTORY" {
                let cls: Table = g.get(*class)?;
                let mt: Table = cls.get("__instance_mt")?;
                let inst = self.lua.create_table()?;
                inst.set_metatable(Some(mt))?;
                g.set(*name, inst)?;
            }
        }
        crate::pawns::install(&self.lua)?;
        self.mark("PawnFactory:CreatePawn", Status::Reconstructed);
        Ok(())
    }

    /// Executa um arquivo relativo ao diretório do jogo. Remove BOM UTF-8 se houver.
    pub fn run_file(&self, rel: &str) -> Outcome {
        let path = self.game_dir.join(rel);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => return Outcome::Missing,
        };
        let src = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
        match self.lua.load(src).set_name(format!("@{rel}")).exec() {
            Ok(()) => Outcome::Ok,
            Err(e) => Outcome::Error(e.to_string()),
        }
    }

    /// Sequência de boot: `scripts/scripts.lua`, depois cada arquivo de `GetScripts()` em ordem.
    /// Um erro num arquivo não interrompe os seguintes (registramos e seguimos).
    pub fn boot(&self) -> mlua::Result<()> {
        let first = self.run_file("scripts/scripts.lua");
        self.report.borrow_mut().scripts.push(("scripts/scripts.lua".into(), first.clone()));
        if first != Outcome::Ok {
            return Ok(());
        }
        // ActiveLua (vftable 0x00836FC8): slot 2 (0x00675D00) lê GetScripts(); 0x0044D1A0 carrega cada um
        // ("%s doesn't exist!" para ausentes) e depois chama o slot 1 (0x00675C70):
        // SetPawn(nil), SetBoard(nil), SetGame(nil).
        let get: Function = self.lua.globals().get("GetScripts")?;
        let list: Vec<String> = get.call(())?;
        self.report.borrow_mut().console.push(format!("loading {} scripts", list.len()));
        for rel in list {
            let out = self.run_file(&rel);
            if out == Outcome::Missing {
                self.report.borrow_mut().console.push(format!("{rel} doesn't exist!"));
            }
            self.report.borrow_mut().scripts.push((rel, out));
        }
        self.clear_engine_globals()
    }

    /// 0x00675C70: avisa os scripts de que não há Pawn/Board/Game ativos.
    pub fn clear_engine_globals(&self) -> mlua::Result<()> {
        for f in ["SetPawn", "SetBoard", "SetGame"] {
            if let Ok(func) = self.lua.globals().get::<Function>(f) {
                func.call::<()>(Value::Nil)?;
            }
        }
        Ok(())
    }

    /// Entrega um tabuleiro aos scripts como o motor faz: chama a função Lua `SetBoard(board)`.
    pub fn set_board(&self, board: crate::world::SharedBoard) -> mlua::Result<crate::world::LBoard> {
        let lb = crate::world::LBoard(board);
        let f: Function = self.lua.globals().get("SetBoard")?;
        f.call::<()>(lb.clone())?;
        Ok(lb)
    }

    /// Cria o `Game` e o entrega aos scripts com `SetGame`. O máximo da energia vem de `getCityPower(0)`, como
    /// em 0x609BFE (1.2.93). A energia atual 5 foi medida no jogo original, na dificuldade Normal; a rotina
    /// que a grava não foi lida.
    pub fn new_game(&self, game: GameState) -> mlua::Result<SharedGame> {
        let shared = Rc::new(RefCell::new(game));
        let globals = self.lua.globals();
        if let Some(f) = globals.get::<Option<Function>>("getCityPower")? {
            let max: i32 = f.call(0)?;
            let mut g = shared.borrow_mut();
            g.power.0 = 5;
            g.set_power_max(max);
        }
        let set: Function = globals.get("SetGame")?;
        set.call::<()>(crate::game::LGame(shared.clone()))?;
        Ok(shared)
    }

    /// Define a unidade ativa: chama `SetPawn(pawn)`.
    pub fn set_pawn(&self, board: &crate::world::SharedBoard, idx: usize) -> mlua::Result<()> {
        let f: Function = self.lua.globals().get("SetPawn")?;
        f.call::<()>(crate::world::LPawn { board: board.clone(), idx })
    }

    /// Resumo da cobertura da API por estado.
    pub fn coverage_counts(&self) -> BTreeMap<Status, usize> {
        let mut m = BTreeMap::new();
        for st in self.report.borrow().coverage.values() {
            *m.entry(*st).or_default() += 1;
        }
        m
    }
}
