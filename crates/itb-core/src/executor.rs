//! Executor de `SkillEffect` com relógio — Etapa 12.
//!
//! Reconstrói a ordem em que o motor aplica uma `DamageList`:
//! * `Board:AddEffect` (0x0055F7B0) põe a lista na fila Board+0x2C50 com cronômetro −1 (Board+0x2C5C).
//! * A atualização do tabuleiro (0x00568CF0) percorre a fila: cronômetro > 0 desconta o tempo e espera;
//!   −1 espera o tabuleiro ficar ocioso (estados 1, 2, 8 de 0x005689F0); −2 espera só se o estado for 8
//!   (projéteis). A primeira lista pronta sai da fila e é executada; no máximo uma por atualização.
//! * O executor (0x005601C0, WeaponTarget slot 0x0C) percorre as entradas: corpo a corpo (tipo de
//!   movimento 3) guarda o dano na unidade atacante, que o aplica 0,15 s depois (0x00632B40 →
//!   0x006308A1); tipo 0 vai direto para `DamageSpace`; os outros tipos viram projéteis (0x00656140)
//!   que aplicam o dano ao chegar; em seguida, se for movimento, move a unidade. Uma entrada com
//!   `fDelay != 0` encerra a execução: o resto da lista volta para o FIM da fila com esse cronômetro.
//! * `Pawn::Push` arma um cronômetro de 0,4 s (0x0062CD51); ao terminar, `ResolvePush`.
//!
//! Tempos de voo de projéteis e de animação de movimento ainda não foram lidos: ficam em `Timing`.

use crate::apply::Hit;
use crate::board_occupancy::{OccupancyBoard, PawnIdx};

/// Durações em segundos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    /// Empurrão: 0,4 s (0x3ECCCCCD gravado em Pawn+0x968 por 0x0062CC00). Lido no EXE.
    pub push: f32,
    /// Corpo a corpo: 0,15 s (0x3E19999A em Pawn+0x968 por 0x00632B40). Lido no EXE.
    pub melee: f32,
    /// Voo de projétil (tipos 2 e 5). SUPOSIÇÃO — a atualização dos projéteis ainda não foi lida.
    pub projectile: f32,
    /// Voo de artilharia, ataque aéreo e lançador (tipos 1, 3, 4, 6). SUPOSIÇÃO.
    pub artillery: f32,
}

impl Default for Timing {
    fn default() -> Self { Self { push: 0.4, melee: 0.15, projectile: 0.25, artillery: 0.5 } }
}

#[derive(Clone, Debug)]
enum Pending {
    Push { pawn: PawnIdx },
    Melee { pawn: PawnIdx, hit: Hit },
    Land { hit: Hit },
}

#[derive(Clone, Debug)]
struct Timed { at: f32, seq: u64, what: Pending }

#[derive(Clone, Debug)]
struct Queued { hits: Vec<Hit>, timer: f32 }

/// Fila de efeitos do tabuleiro e animações pendentes.
#[derive(Clone, Debug, Default)]
pub struct Executor {
    pub now: f32,
    pub timing: Timing,
    queue: Vec<Queued>,
    pending: Vec<Timed>,
    seq: u64,
    /// Registro legível do que aconteceu e quando (para relatórios e testes).
    pub log: Vec<String>,
}

impl Executor {
    pub fn with_timing(timing: Timing) -> Self { Self { timing, ..Default::default() } }

    /// 0x0055F7B0 `Board:AddEffect(SkillEffect)`: entra no fim da fila com cronômetro −1.
    pub fn add_effect(&mut self, hits: Vec<Hit>) { self.queue.push(Queued { hits, timer: -1.0 }); }

    fn busy(&self) -> bool { !self.pending.is_empty() }
    fn projectiles(&self) -> bool { self.pending.iter().any(|t| matches!(t.what, Pending::Land { .. })) }

    fn schedule(&mut self, dt: f32, what: Pending) {
        self.seq += 1;
        self.pending.push(Timed { at: self.now + dt, seq: self.seq, what });
    }

    /// Converte em cronômetros de 0,4 s os empurrões que o tabuleiro acabou de armar
    /// (um novo `Push` na mesma unidade reinicia o cronômetro).
    fn collect_pushes(&mut self, b: &mut OccupancyBoard) {
        let started: Vec<PawnIdx> = std::mem::take(&mut b.pushes_started);
        for i in started {
            self.pending.retain(|t| !matches!(t.what, Pending::Push { pawn } if pawn == i));
            self.schedule(self.timing.push, Pending::Push { pawn: i });
        }
    }

    /// 0x005601C0: executa a lista até a primeira entrada com atraso.
    fn run_list(&mut self, b: &mut OccupancyBoard, hits: &[Hit]) {
        for (i, h) in hits.iter().enumerate() {
            if h.move_type == 3 {
                let attacker = h.path.first().and_then(|&p| b.get_pawn_at(p));
                match attacker {
                    Some(a) => {
                        self.log.push(format!("{:.2}s corpo a corpo de {} armado", self.now, a));
                        self.schedule(self.timing.melee, Pending::Melee { pawn: a, hit: h.clone() });
                    }
                    None => b.damage_space(h), // "COULD NOT FIND UNIT FOR MELEE ATTACK?"
                }
            } else if h.kind == 0 {
                b.damage_space(h);
            } else {
                let dt = if matches!(h.kind, 2 | 5) { self.timing.projectile } else { self.timing.artillery };
                self.log.push(format!("{:.2}s projétil tipo {} disparado contra ({},{})", self.now, h.kind, h.loc.x, h.loc.y));
                self.schedule(dt, Pending::Land { hit: h.clone() });
            }
            if OccupancyBoard::hit_is_movement(h) { b.apply_movement(h); }
            self.collect_pushes(b);
            if h.delay != 0.0 {
                let rest = hits[i + 1..].to_vec();
                self.log.push(format!("{:.2}s pausa {} ({} entradas restantes)", self.now, h.delay, rest.len()));
                self.queue.push(Queued { hits: rest, timer: h.delay });
                break;
            }
        }
    }

    /// Primeira lista pronta da fila (0x00568CF0). Listas vazias saem sem contar como execução.
    fn take_ready(&mut self) -> Option<Vec<Hit>> {
        let (busy, proj) = (self.busy(), self.projectiles());
        let mut k = 0;
        while k < self.queue.len() {
            let t = self.queue[k].timer;
            let waiting = t > 0.0 || (t == -1.0 && busy) || (t == -2.0 && proj);
            if waiting { k += 1; continue; }
            let q = self.queue.remove(k);
            if q.hits.is_empty() { continue; }
            return Some(q.hits);
        }
        None
    }

    /// Roda até não haver nada na fila nem animação pendente.
    pub fn run_until_idle(&mut self, b: &mut OccupancyBoard) {
        b.pushes_started.clear();
        for _guard in 0..100_000 {
            if let Some(hits) = self.take_ready() {
                self.run_list(b, &hits);
                continue;
            }
            // nada pronto: avança o relógio até o próximo acontecimento
            let next_pending = self.pending.iter().map(|t| t.at).fold(f32::INFINITY, f32::min);
            let next_timer = self.queue.iter().filter(|q| q.timer > 0.0).map(|q| self.now + q.timer).fold(f32::INFINITY, f32::min);
            let next = next_pending.min(next_timer);
            if !next.is_finite() {
                // só restam esperas por ociosidade: com tudo ocioso elas ficam prontas
                if self.queue.is_empty() { return; }
                if self.busy() { return; }
                for q in &mut self.queue { if q.timer < 0.0 { q.timer = 0.0; } }
                continue;
            }
            let dt = next - self.now;
            for q in &mut self.queue { if q.timer > 0.0 { q.timer -= dt; if q.timer <= 0.0 { q.timer = 0.0; } } }
            self.now = next;
            // acontecimentos vencidos, em ordem de tempo e depois de criação
            let mut due: Vec<Timed> = Vec::new();
            self.pending.retain(|t| if t.at <= self.now { due.push(t.clone()); false } else { true });
            due.sort_by(|a, b| a.at.total_cmp(&b.at).then(a.seq.cmp(&b.seq)));
            for t in due {
                match t.what {
                    Pending::Push { pawn } => {
                        self.log.push(format!("{:.2}s empurrão de {} resolvido", self.now, pawn));
                        b.pawn_resolve_push(pawn);
                    }
                    Pending::Melee { pawn, hit } => {
                        self.log.push(format!("{:.2}s golpe corpo a corpo de {} em ({},{})", self.now, pawn, hit.loc.x, hit.loc.y));
                        b.damage_space(&hit);
                    }
                    Pending::Land { hit } => {
                        self.log.push(format!("{:.2}s projétil chegou em ({},{})", self.now, hit.loc.x, hit.loc.y));
                        b.damage_space(&hit);
                    }
                }
                self.collect_pushes(b);
            }
        }
    }
}
