//! Aplicação de efeitos ao tabuleiro — Etapa 11 (versão 1).
//!
//! Reconstrói, a partir do assembly do `Breach.exe` v1.2.88, a parte de ESTADO de:
//! `Board::DamageSpace` (0x0055F200), `Tile::ApplyDamage` (0x005ABA90), `Pawn::ApplyDamage`
//! (0x006295B0), `Health::Add` (0x004DFFC0), empurrão (`Tile::Push` 0x005ADBB0, `Pawn::Push`
//! 0x0062CC00, `Pawn::ResolvePush` 0x0062CDD0, `Board::Bump` 0x0055F680) e os modificadores de casa e
//! de unidade que essas rotinas chamam. Provas: `re/evidencias/rotinas_dano.md`.
//!
//! Fora do escopo desta versão (registrado como `BoardEvent`, sem efeito no estado): rede elétrica,
//! itens/minas, cápsulas, criação de unidades (`sPawn`), teias, retirada (`bEvacuate`), passivas de
//! "Retaliation", conquistas, sons, animações e tutoriais. O original resolve empurrões depois de uma
//! animação (0,4 s); aqui eles são resolvidos logo após cada `SpaceDamage` (ver DIVERGENCIAS).

use crate::board_occupancy::{consts::*, OccupancyBoard, PawnIdx, Point, Tile};
use crate::damage::{DAMAGE_DEATH, DAMAGE_ZERO};

/// Vetores de direção (0x008D0DE0, iguais a 0x008CF9AC): UP, RIGHT, DOWN, LEFT.
pub const DIR_VEC: [Point; 4] = [Point { x: 0, y: -1 }, Point { x: 1, y: 0 }, Point { x: 0, y: 1 }, Point { x: -1, y: 0 }];
pub const DIR_NONE: i32 = 4;
pub const DIR_FLIP: i32 = 6;

/// Campos de `SpaceDamage` que as rotinas de aplicação leem (offsets do objeto original).
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub loc: Point,          // +0x00
    pub damage: i32,         // +0x08 iDamage
    pub push: i32,           // +0x0C iPush
    pub shield: i32,         // +0x10 iShield
    pub fire: i32,           // +0x18 iFire
    pub frozen: i32,         // +0x1C iFrozen
    pub injure: i32,         // +0x20 iInjure
    pub smoke: i32,          // +0x24 iSmoke
    pub acid: i32,           // +0x28 iAcid
    pub crack: i32,          // +0x2C iCrack
    pub pawn: String,        // +0xA4 sPawn
    pub pawn_team: i32,      // +0xBC iPawnTeam
    pub evacuate: bool,      // +0xC7 bEvacuate
    pub terrain: i32,        // +0xDC iTerrain (10 = sem mudança)
    pub script: String,      // +0xE0 sScript
    pub web_from: Point,     // +0x110
    pub item: String,        // +0x118 sItem
    pub i_0x130: i32,        // +0x130 (2 força o modo 2 em Tile::ApplyDamage)
    pub kind: i32,           // +0x98 (0 normal, 1 artilharia, 2 projétil, 3 ataque aéreo...)
    pub path: Vec<Point>,    // +0xCC caminho de movimento
    pub move_type: i32,      // +0xD8 (1 salto, 2 investida, 3 corpo a corpo, 4 teleporte, 5 escavar)
    pub delay: f32,          // +0xC8 fDelay (0 = segue; > 0 segundos; −1 FULL_DELAY; −2 PROJ_DELAY)
}

impl Hit {
    /// Padrões dos construtores de `SpaceDamage` (ver `records.rs`).
    pub fn new(loc: Point, damage: i32) -> Self {
        Self { loc, damage, push: DIR_NONE, shield: 0, fire: 0, frozen: 0, injure: 0, smoke: 0, acid: 0, crack: 0,
               pawn: String::new(), pawn_team: TEAM_NONE, evacuate: false, terrain: 10, script: String::new(),
               web_from: Point::INVALID, item: String::new(), i_0x130: 3,
               kind: 0, path: Vec::new(), move_type: 0, delay: 0.0 }
    }
}

/// Efeitos registrados e não simulados nesta versão.
#[derive(Clone, Debug, PartialEq)]
pub enum BoardEvent {
    Script(String),
    GridDamage { loc: Point, amount: i32 },
    ItemTriggered(Point),
    PodDestroyed(Point),
    SpawnPawn { loc: Point, kind: String, team: i32 },
    SetItem { loc: Point, item: String },
    Web { loc: Point, from: Point },
    Evacuate(PawnIdx),
    Burrowed(PawnIdx),
    Flip(PawnIdx),
    Retaliation(PawnIdx),
    BuildingPowerCheck(Point),
    /// Surgimento impedido por quem ocupa a casa (`Alert_Blocked`).
    EmergeBlocked(Point),
}

/// 0x004DFFC0 `Health::Add(delta)` sobre (atual, máximo): limita o delta ao possível, soma e
/// prende o resultado em [0, máximo]. Devolve o delta efetivo.
pub fn health_add(cur: &mut i32, max: i32, delta: i32) -> i32 {
    let actual = if delta < 0 { delta.max(cur.wrapping_neg()) } else { delta.min(max.wrapping_sub(*cur)) };
    *cur = cur.wrapping_add(actual);
    let v = if max < *cur { max } else { *cur };
    *cur = if v > 0 { v } else { 0 };
    actual
}

fn add(a: Point, b: Point) -> Point { Point::new(a.x.wrapping_add(b.x), a.y.wrapping_add(b.y)) }

impl OccupancyBoard {
    /// Casa real ou a sentinela (Board+0x5C), como o EXE faz com coordenadas inválidas.
    pub fn tile_mut_or_sentinel(&mut self, p: Point) -> &mut Tile {
        if self.is_valid(p) { self.tile_mut(p).expect("válida") } else { &mut self.sentinel }
    }

    fn passive(&self, name: &str) -> bool { self.passive_skills.iter().any(|s| s == name) }

    /// 0x00629540-equivalente para a lista da casa: índices das unidades em `p` (cópia).
    fn tile_pawns(&self, p: Point) -> Vec<PawnIdx> { self.tile(p).pawns.clone() }

    /// Primeira unidade da casa se `HasPawn(true)`.
    fn first_pawn(&self, p: Point) -> Option<PawnIdx> {
        let t = self.tile(p);
        if self.tile_has_pawn(t, true) { t.pawns.first().copied() } else { None }
    }

    // ------------------------------------------------------------------ unidade

    /// 0x00637210 `SetInfected(bool)`.
    pub fn pawn_set_infected(&mut self, i: PawnIdx, v: bool) { self.pawns[i].status.infected = v; }

    /// 0x00636970 `SetFrozen(bool)`: grava +0x8D1; congelar apaga fogo e infecção.
    pub fn pawn_set_frozen(&mut self, i: PawnIdx, v: bool) {
        self.pawns[i].status.frozen = v;
        if v {
            self.pawn_set_fire(i, false);
            self.pawn_set_infected(i, false);
        }
    }

    /// 0x00639060 `SetFire(bool)`. Escudo impede pegar fogo. Pegar fogo limpa a infecção antes de
    /// testar a imunidade (0x00638BD0), que só é furada por "Passive_FireBoost" em mech. Tipos com
    /// +0x374 ignoram tudo. Pegar fogo apaga o gelo.
    pub fn pawn_set_fire(&mut self, i: PawnIdx, v: bool) {
        let p = &self.pawns[i];
        if p.is_shield() {
            if v { return; }
        } else if v {
            self.pawn_set_infected(i, false);
            let p = &self.pawns[i];
            if p.damage.query_0x638bd0 && !(self.passive("Passive_FireBoost") && p.mech) { return; }
        }
        if self.pawns[i].damage.type_ignores_fire_0x374 { return; }
        self.pawns[i].status.fire = v;
        if v {
            self.pawn_set_infected(i, false);
            self.pawn_set_frozen(i, false);
        }
    }

    /// 0x006397C0 `SetAcid(bool)`: escudo impede; "Thick" (`vftable[1]`) nunca fica com ácido.
    pub fn pawn_set_acid(&mut self, i: PawnIdx, v: bool) {
        if self.pawns[i].is_shield() {
            if v { return; }
        } else if v {
            self.pawn_set_infected(i, false);
        }
        if self.pawns[i].vslot1("Thick") {
            if v { return; }
            self.pawns[i].status.acid = false;
            return;
        }
        if v && !self.pawns[i].status.acid { self.pawn_set_infected(i, false); }
        self.pawns[i].status.acid = v;
    }

    /// 0x006335F0 `SetShield(bool)` (1.2.93: 0x634490), parte de estado: desligar com a passiva de escudo de turno
    /// (`IsTurnShield`, 0x62D4A0) não faz nada; ligar só grava +0x8D4 se a unidade não está morta (slot 0x10);
    /// desligar grava 0. O resto é animação e som.
    pub fn pawn_set_shield(&mut self, i: PawnIdx, v: bool) {
        let p = &mut self.pawns[i];
        if !v && p.damage.query_0x62c5f0 { return; }
        if v && p.is_dead() { return; }
        p.damage.shield_0x8d4 = v;
    }

    /// 0x00639A50 `Kill(bool)`: se vivo, marca +0x921, tira 100 de vida e limpa a infecção.
    pub fn pawn_kill(&mut self, i: PawnIdx) {
        if self.pawns[i].is_dead() { return; }
        let p = &mut self.pawns[i];
        p.status.died_0x921 = true;
        let max = p.max_health;
        health_add(&mut p.health, max, -100);
        self.pawn_set_infected(i, false);
    }

    /// 0x0062BF90 `ChangeHealth(delta, 1, modo)` (parte de estado). Morto que não é cadáver: nada.
    /// Dano com "Passive_PlayerTurnShield" ativa (0x0062C5F0): nada. Escavador ("Burrows") vivo que
    /// toma dano fora de chão rachado (ou com modo 1) se enterra (evento).
    pub fn pawn_change_health(&mut self, i: PawnIdx, delta: i32, mode: i32) -> i32 {
        let p = &self.pawns[i];
        if p.is_dead() && !p.is_corpse(self.global_list_has_12) { return 0; }
        if delta < 0 && p.damage.query_0x62c5f0 { return 0; }
        let p = &mut self.pawns[i];
        let max = p.max_health;
        let actual = health_add(&mut p.health, max, delta);
        let p = &self.pawns[i];
        if p.is_burrower() && delta < 0 && !p.is_dead()
            && (!self.is_terrain(p.get_space(), crate::board_terrain::TERRAIN_QUERY_CRACKED) || mode == 1) {
            self.events.push(BoardEvent::Burrowed(i));
        }
        if delta < 0 && self.pawns[i].vslot1("Retaliation") && !self.pawns[i].is_dead() {
            self.events.push(BoardEvent::Retaliation(i));
        }
        actual
    }

    /// 0x006295B0 `Pawn::ApplyDamage(SpaceDamage, int modo)`.
    pub fn pawn_apply_damage(&mut self, i: PawnIdx, hit: &Hit, mode: i32) {
        let mut acid = hit.acid;
        let mut dead_mech = false;
        if self.pawns[i].is_dead() {
            if !self.pawns[i].mech { return; }
            acid = 0;
            dead_mech = true;
        }
        let mut d = hit.damage;
        let mut armor_checked = false;
        if self.pawns[i].is_shield() && d != DAMAGE_ZERO && d > 0 {
            if d != DAMAGE_DEATH { d = 0; }
            self.pawn_set_shield(i, false);
        } else if self.pawns[i].status.frozen && d != DAMAGE_ZERO && d > 0 {
            if d != DAMAGE_DEATH { d = 0; }
            self.pawn_set_frozen(i, false);
        }
        if d == DAMAGE_ZERO {
            d = 0;
        } else if d > 0 {
            if mode == 0 {
                armor_checked = true;
                if self.pawns[i].damage.query_0x63c100 { d = (d - 1).max(0); }
                if self.pawns[i].status.acid { d = d.wrapping_add(d); }
                self.pawn_set_infected(i, false);
            }
            if d > 0 {
                match mode {
                    1 => {
                        self.pawn_set_infected(i, false);
                        if self.passive("Passive_ForceAmp") && self.pawns[i].is_team(7) { d += 1; }
                    }
                    2 => self.pawn_set_infected(i, false),
                    _ => {}
                }
            }
        }
        match hit.fire { 1 => self.pawn_set_fire(i, true), 2 => self.pawn_set_fire(i, false), _ => {} }
        match hit.injure { 1 => self.pawns[i].status.injured_0x8d6 = true, 2 => self.pawns[i].status.injured_0x8d6 = false, _ => {} }
        self.pawn_change_health(i, -d, mode);
        match acid { 1 => self.pawn_set_acid(i, true), 2 => self.pawn_set_acid(i, false), _ => {} }
        if !(d > 0 || armor_checked) && d < 0 {
            self.pawn_set_fire(i, false);
            self.pawn_set_frozen(i, false);
            self.pawn_set_infected(i, false);
            self.pawn_set_fire(i, false);
            self.pawn_set_acid(i, false);
        }
        if self.pawns[i].is_dead() && !dead_mech { self.pawns[i].status.died_0x921 = true; }
    }

    // ------------------------------------------------------------------ casa

    /// 0x0059EBE0 escudo da casa: prédio/montanha → contagem 2; senão escudo na unidade.
    pub fn tile_shield_on(&mut self, p: Point) {
        let t = self.tile(p).terrain;
        if t == TERRAIN_BUILDING || t == TERRAIN_MOUNTAIN {
            self.tile_mut_or_sentinel(p).shield_0x2aa4 = 2;
        } else if let Some(i) = self.first_pawn(p) {
            self.pawn_set_shield(i, true);
        }
    }

    /// 0x0059EE20 remove o escudo da casa e o da unidade.
    pub fn tile_shield_off(&mut self, p: Point) {
        self.tile_mut_or_sentinel(p).shield_0x2aa4 = 0;
        if let Some(i) = self.first_pawn(p) { self.pawn_set_shield(i, false); }
    }

    fn tile_shielded(&self, p: Point) -> bool {
        self.tile(p).shield_0x2aa4 > 0 || self.first_pawn(p).is_some_and(|i| self.pawns[i].is_shield())
    }

    /// 0x005AABD0 `SetAcid(bool)` da casa: nada na lava; ácido apaga fogo.
    pub fn tile_set_acid(&mut self, p: Point, v: bool) {
        let t = self.tile_mut_or_sentinel(p);
        if t.lava_0x2ac8 { return; }
        t.acid = v as u8;
        if v { t.fire = 0; }
    }

    /// 0x005AB740 `SetSmoke(bool, bool)`: fumaça apaga fogo e remove +0x2AA0.
    pub fn tile_set_smoke(&mut self, p: Point, v: bool) {
        self.tile_mut_or_sentinel(p).smoke = v as u8;
        if v {
            self.tile_mut_or_sentinel(p).fire = 0;
            self.tile_set_vines(p, false);
        }
    }

    /// 0x005A94B0 grava o byte +0x2AA0 (exceto gelo, água, buraco, montanha; não com fogo/fumaça).
    pub fn tile_set_vines(&mut self, p: Point, v: bool) {
        let t = self.tile(p);
        if (t.vines != 0) == v { return; }
        let terr = t.terrain;
        if terr == TERRAIN_ICE || Self::tile_is_terrain(t, TERRAIN_WATER) || terr == TERRAIN_HOLE || terr == TERRAIN_MOUNTAIN { return; }
        if t.fire == 0 && t.smoke == 0 {
            self.tile_mut_or_sentinel(p).vines = v as u8;
            if v && (terr == TERRAIN_FOREST || terr == TERRAIN_SAND) { self.set_terrain(p, TERRAIN_ROAD); }
        } else {
            self.tile_mut_or_sentinel(p).vines = 0;
        }
    }

    /// 0x005AB580 `SetFire(bool)` da casa. Buraco não pega fogo; água (sem a exceção +0x23AC) também
    /// não. Pegar fogo apaga fumaça, +0x2AA0, gelo do prédio e ácido. Floresta vira chão com fogo 2;
    /// gelo derrete (água) sem fogo; demais: fogo 1.
    pub fn tile_set_fire(&mut self, p: Point, v: bool) {
        let t = self.tile(p);
        if !v { self.tile_mut_or_sentinel(p).fire = 0; return; }
        if t.terrain == TERRAIN_HOLE { return; }
        if Self::tile_is_terrain(t, TERRAIN_WATER) { return; }
        if t.fire != 0 { return; }
        {
            let t = self.tile_mut_or_sentinel(p);
            t.smoke = 0;
            t.vines = 0;
        }
        self.tile_set_frozen(p, false);
        self.tile_set_acid(p, false);
        match self.tile(p).terrain {
            TERRAIN_FOREST => {
                self.tile_mut_or_sentinel(p).fire = 2;
                self.set_terrain(p, TERRAIN_ROAD);
            }
            TERRAIN_ICE => {
                self.set_terrain(p, TERRAIN_WATER);
                self.tile_mut_or_sentinel(p).fire = 0;
            }
            _ => self.tile_mut_or_sentinel(p).fire = 1,
        }
    }

    /// 0x0059E8A0 `SetFrozen(bool)` da casa: não congela lava. Água vira gelo; gelo volta à vida
    /// cheia; congelar apaga o fogo. Prédio/montanha guardam o gelo em +0x2AA1; nos demais
    /// terrenos o gelo vai para a unidade.
    pub fn tile_set_frozen(&mut self, p: Point, v: bool) {
        if self.tile(p).lava_0x2ac8 && v { return; }
        if v && Self::tile_is_terrain(self.tile(p), TERRAIN_WATER) { self.set_terrain(p, TERRAIN_ICE); }
        if self.tile(p).terrain == TERRAIN_ICE {
            let t = self.tile_mut_or_sentinel(p);
            t.health = t.max_health;
        }
        if v { self.tile_set_fire(p, false); }
        let terr = self.tile(p).terrain;
        if terr == TERRAIN_BUILDING || terr == TERRAIN_MOUNTAIN {
            self.tile_mut_or_sentinel(p).frozen_0x2aa1 = v;
        } else if let Some(i) = self.first_pawn(p) {
            self.pawn_set_frozen(i, v);
        }
    }

    /// 0x005AAC40 `SetCracked(bool)`. Só racha se `IsCrackable`; dispara item, destrói cápsula,
    /// zera +0x2BA0; montanha/gelo tomam 1 de dano em vez de rachar; floresta, entulho e areia viram
    /// chão; racha apagando ácido e fogo.
    pub fn tile_set_cracked(&mut self, p: Point, v: bool) {
        if !v { self.tile_mut_or_sentinel(p).cracked = 0; return; }
        if !self.is_crackable(p) { return; }
        if !self.tile(p).item.is_empty() { self.events.push(BoardEvent::ItemTriggered(p)); }
        self.tile_mut_or_sentinel(p).int_0x2ba0 = 0;
        if self.tile(p).pod_0x2ae4 == 1 { self.events.push(BoardEvent::PodDestroyed(p)); }
        let terr = self.tile(p).terrain;
        if terr == TERRAIN_ICE || terr == TERRAIN_MOUNTAIN {
            self.tile_damage_terrain(p, 1);
            return;
        }
        if terr == TERRAIN_FOREST || terr == TERRAIN_RUBBLE || terr == TERRAIN_SAND { self.set_terrain(p, TERRAIN_ROAD); }
        self.tile_set_acid(p, false);
        let t = self.tile_mut_or_sentinel(p);
        t.fire = 0;
        t.cracked = 1;
    }

    /// 0x005AAE10 dano em gelo/montanha (−1 de vida). Gelo a 0 vira água. Montanha a 0 (ou com
    /// `DAMAGE_DEATH`) vira entulho, ou chão se a casa tiver ácido.
    pub fn tile_damage_terrain(&mut self, p: Point, damage: i32) {
        let terr = self.tile(p).terrain;
        if terr != TERRAIN_ICE && terr != TERRAIN_MOUNTAIN { return; }
        let t = self.tile_mut_or_sentinel(p);
        let max = t.max_health;
        health_add(&mut t.health, max, -1);
        let hp = t.health;
        if terr == TERRAIN_ICE {
            if hp == 0 { self.set_terrain(p, TERRAIN_WATER); }
        } else if hp == 0 || damage == DAMAGE_DEATH {
            let to = if self.tile(p).acid == 0 { TERRAIN_RUBBLE } else { TERRAIN_ROAD };
            self.set_terrain(p, to);
        }
    }

    /// 0x005A0CB0 `SetTerrain(int)` (parte de estado).
    pub fn set_terrain(&mut self, p: Point, t: i32) {
        match t {
            14 => { self.tile_mut_or_sentinel(p).lava_0x2ac8 = true; self.set_terrain(p, TERRAIN_WATER); self.tile_mut_or_sentinel(p).fire = 0; return; }
            11 => { self.tile_set_fire(p, true); return; }
            12 => { self.tile_set_acid(p, true); return; }
            15 => { self.tile_mut_or_sentinel(p).danger_0x2af0 = true; return; }
            16 => { self.tile_set_cracked(p, true); return; }
            _ => {}
        }
        let old = self.tile(p).terrain;
        if old == TERRAIN_MOUNTAIN && self.tile(p).frozen_0x2aa1 && (t == TERRAIN_WATER || t == TERRAIN_HOLE) {
            self.tile_set_frozen(p, false);
        }
        self.tile_mut_or_sentinel(p).terrain = t;
        if t == TERRAIN_ICE || t == TERRAIN_MOUNTAIN {
            let tl = self.tile_mut_or_sentinel(p);
            tl.health = 2;
            tl.max_health = 2;
        }
        let tile = self.tile(p);
        if Self::tile_is_terrain(tile, TERRAIN_WATER) || tile.terrain == TERRAIN_HOLE {
            if tile.pod_0x2ae4 == 1 { self.events.push(BoardEvent::PodDestroyed(p)); }
            let tl = self.tile_mut_or_sentinel(p);
            tl.fire = 0;
            tl.cracked = 0;
            tl.int_0x2ba0 = 0;
        }
    }

    /// 0x005A10B0 `AddBuilding`: prédio existente ganha 1 de vida (e de máximo, se estava cheio);
    /// senão vira prédio com vida 1/1.
    pub fn tile_add_building(&mut self, p: Point) {
        if self.tile(p).terrain == TERRAIN_BUILDING {
            let t = self.tile_mut_or_sentinel(p);
            if t.health == t.max_health { t.max_health += 1; }
            let max = t.max_health;
            health_add(&mut t.health, max, 1);
            t.byte_0x2710 = true;
        } else {
            let t = self.tile_mut_or_sentinel(p);
            t.max_health = 1;
            t.health = 1;
            self.set_terrain(p, TERRAIN_BUILDING);
            self.tile_mut_or_sentinel(p).byte_0x2710 = true;
        }
    }

    /// 0x005ABA90 `Tile::ApplyDamage(SpaceDamage, int modo)` — parte de estado.
    pub fn tile_apply_damage(&mut self, p: Point, hit: &Hit, mode_arg: i32) {
        let mode = if hit.i_0x130 == 2 { 2 } else { mode_arg };
        let mut d = hit.damage;
        let mut frozen = hit.frozen;
        self.tile_mut_or_sentinel(p).hit_0x2630 = true;
        if hit.shield > 0 { self.tile_shield_on(p); } else if hit.shield < 0 { self.tile_shield_off(p); }
        let has_pawn = self.first_pawn(p).is_some();
        let player_shield = self.first_pawn(p).is_some_and(|i| self.pawns[i].damage.query_0x62c5f0);
        let pawn_frozen = self.first_pawn(p).is_some_and(|i| self.pawns[i].status.frozen);
        if self.tile_shielded(p) || self.tile(p).frozen_0x2aa1 || pawn_frozen || player_shield {
            if d != DAMAGE_ZERO && d > 0 {
                if self.tile_shielded(p) {
                    self.tile_shield_off(p);
                } else {
                    self.tile_set_frozen(p, false);
                }
                if d != DAMAGE_DEATH { d = 0; }
            }
            if self.tile_shielded(p) { frozen = 0; }
        }
        // cada unidade recebe a cópia JÁ ALTERADA (dano e gelo zerados pelo escudo/gelo da casa)
        if self.tile_has_pawn(self.tile(p), true) {
            let mut copy = hit.clone();
            copy.damage = d;
            copy.frozen = frozen;
            for i in self.tile_pawns(p) { self.pawn_apply_damage(i, &copy, mode); }
        }
        if d == DAMAGE_ZERO { d = 0; }
        let terr = self.tile(p).terrain;
        if d > 0 {
            if mode != 1 {
                if !self.tile(p).item.is_empty() { self.events.push(BoardEvent::ItemTriggered(p)); }
                if self.tile(p).pod_0x2ae4 == 1 { self.events.push(BoardEvent::PodDestroyed(p)); }
            }
            if self.tile(p).vines != 0 && mode != 1 { self.tile_set_vines(p, false); }
            if self.tile(p).cracked != 0 && mode != 1 {
                self.set_terrain(p, TERRAIN_HOLE);
            } else {
                match terr {
                    TERRAIN_ICE if mode != 1 => self.tile_damage_terrain(p, d),
                    TERRAIN_FOREST if mode != 1 && hit.terrain == 10 => self.tile_set_fire(p, true),
                    TERRAIN_SAND if mode != 1 => { self.tile_set_smoke(p, true); self.set_terrain(p, TERRAIN_ROAD); }
                    TERRAIN_MOUNTAIN => self.tile_damage_terrain(p, d),
                    _ => self.damage_building(p, d),
                }
            }
        }
        if self.tile(p).health == 0 && self.tile(p).terrain == TERRAIN_BUILDING && self.tile(p).unique_building.is_empty() {
            self.set_terrain(p, TERRAIN_RUBBLE);
        }
        if hit.terrain != 10 {
            // Para buraco o original só inicia a animação e troca o terreno depois; aqui a troca é imediata.
            if hit.terrain == 1 {
                for i in self.tile_pawns(p) { self.pawn_kill(i); }
                self.tile_add_building(p);
            } else {
                self.set_terrain(p, hit.terrain);
            }
        }
        if hit.crack == 1 { self.tile_set_cracked(p, true); }
        match hit.fire {
            1 => {
                if !self.tile(p).item.is_empty() { self.events.push(BoardEvent::ItemTriggered(p)); }
                self.tile_set_fire(p, true);
                for i in self.tile_pawns(p) { self.pawn_set_fire(i, true); }
            }
            2 => self.tile_mut_or_sentinel(p).fire = 0,
            _ => {}
        }
        if hit.acid == 1 && (!has_pawn || self.tile_shielded(p)) { self.tile_set_acid(p, true); }
        self.tile_push(p, hit.push);
        match hit.smoke { 1 => self.tile_set_smoke(p, true), 2 => self.tile_mut_or_sentinel(p).smoke = 0, _ => {} }
        match frozen { 1 => self.tile_set_frozen(p, true), 2 => self.tile_set_frozen(p, false), _ => {} }
        if hit.evacuate {
            for i in self.tile_pawns(p) { self.events.push(BoardEvent::Evacuate(i)); }
        }
        if hit.web_from.x >= 0 { self.events.push(BoardEvent::Web { loc: p, from: hit.web_from }); }
        if !hit.item.is_empty() { self.events.push(BoardEvent::SetItem { loc: p, item: hit.item.clone() }); }
    }

    /// Ramo de prédio de 0x005ABA90 (1.2.93: 0x5AD383..): tira `min(dano, vida)` da vida da casa
    /// (`Health::Add`) e registra o dano à rede elétrica. Dano <= 0 não mexe no prédio. Em casa povoada (+0x2710), com
    /// partida em curso, `PowerBar::Damage(dano)` vem antes (0x5AD3FE): se a rede resiste, o prédio fica intacto. A
    /// flag global 0x8B8460, que também desliga o sorteio, não foi identificada e fica desligada.
    /// Fora desta versão: o ramo de prédios de energia.
    fn damage_building(&mut self, p: Point, d: i32) {
        if self.tile(p).terrain != TERRAIN_BUILDING { return; }
        let cur = self.tile(p).health;
        let eff = d.min(cur);
        if eff <= 0 { return; }
        if self.tile(p).byte_0x2710 {
            if let (Some(grid), Some(rng)) = (&self.power_grid, &self.rng) {
                if !grid.borrow_mut().damage(eff, &mut rng.borrow_mut()) { return; }
            }
        }
        let t = self.tile_mut_or_sentinel(p);
        let max = t.max_health;
        let actual = health_add(&mut t.health, max, -eff);
        self.events.push(BoardEvent::GridDamage { loc: p, amount: -actual });
    }

    /// 0x0055F200 `Board:DamageSpace(SpaceDamage)` (parte de estado).
    pub fn damage_space(&mut self, hit: &Hit) {
        if !hit.script.is_empty() { self.events.push(BoardEvent::Script(hit.script.clone())); }
        if !self.is_valid(hit.loc) { return; }
        self.tile_apply_damage(hit.loc, hit, 0);
        if hit.terrain == 1 { self.events.push(BoardEvent::BuildingPowerCheck(hit.loc)); }
        if !hit.pawn.is_empty() {
            self.events.push(BoardEvent::SpawnPawn { loc: hit.loc, kind: hit.pawn.clone(), team: hit.pawn_team });
        }
    }

    // ------------------------------------------------------------------ empurrão

    /// 0x005ADBB0 `Tile::Push(dir)`: sem efeito com DIR_NONE ou em prédio; empurra a ÚLTIMA unidade
    /// da lista da casa.
    pub fn tile_push(&mut self, p: Point, dir: i32) {
        if dir == DIR_NONE || self.tile(p).terrain == TERRAIN_BUILDING { return; }
        if let Some(&last) = self.tile(p).pawns.last() { self.pawn_push(last, dir); }
    }

    /// 0x0062CC00 `Pawn::Push(dir)`: `DIR_FLIP` vira o ataque (evento) se `IsFlippable` (+0x91F) e
    /// viva; unidade "estável" (`IsGuarding`, +0x91E) não se move; senão grava a direção pendente se o
    /// destino é válido ou bloqueado. O caminho estável também faz `SetGuard(false)` (+0x8D2), não
    /// modelado: nenhuma leitura desse byte foi achada.
    pub fn pawn_push(&mut self, i: PawnIdx, dir: i32) {
        if dir == DIR_FLIP {
            if !self.pawns[i].status.ignore_flip && !self.pawns[i].is_dead() { self.events.push(BoardEvent::Flip(i)); }
            return;
        }
        if self.pawns[i].is_guarding() { return; }
        let Some(&v) = usize::try_from(dir).ok().and_then(|d| DIR_VEC.get(d)) else { return };
        let target = add(self.pawns[i].space, v);
        if self.pawn_push_blocked(i, dir) || self.is_valid(target) {
            self.pawns[i].pending_push_0x978 = Some(dir);
            self.pushes_started.push(i);
        }
    }

    /// 0x0062CAA0: destino bloqueia a entrada vinda de `dir` (0x00571B20 → 0x005AAA10).
    pub fn pawn_push_blocked(&self, i: PawnIdx, dir: i32) -> bool {
        if dir == DIR_FLIP { return false; }
        let Some(&v) = usize::try_from(dir).ok().and_then(|d| DIR_VEC.get(d)) else { return false };
        let target = add(self.pawns[i].space, v);
        if !self.is_valid(target) { return false; }
        self.tile_blocks_push(target, (dir + 2) & 3)
    }

    /// 0x005AAA10: prédio e montanha bloqueiam; unidade sólida (0x0062CB10) bloqueia sem o byte
    /// +0x2640; parede da casa na direção de entrada (0x0059F7C0) bloqueia.
    pub fn tile_blocks_push(&self, p: Point, from_dir: i32) -> bool {
        let t = self.tile(p);
        if t.terrain == TERRAIN_BUILDING || t.terrain == TERRAIN_MOUNTAIN { return true; }
        if let Some(&first) = t.pawns.first() {
            if self.pawn_is_solid(first) && !t.byte_0x2640 { return true; }
        }
        if from_dir == 4 { t.edge_block_0x2ac4.iter().any(|&b| b) }
        else { t.edge_block_0x2ac4.get(from_dir as usize).copied().unwrap_or(false) }
    }

    /// 0x0062CB10 (parte de estado): viva ou cadáver é sólida. O original também considera sólida a
    /// unidade morta durante parte da animação de morte; aqui a animação já terminou.
    pub fn pawn_is_solid(&self, i: PawnIdx) -> bool {
        let p = &self.pawns[i];
        !p.is_dead() || p.is_corpse(self.global_list_has_12)
    }

    /// 0x0062C910 `SetSpace` (parte de estado): tira a unidade da lista da casa antiga e a põe no
    /// fim da lista da casa nova.
    pub fn move_pawn(&mut self, i: PawnIdx, to: Point) {
        let from = self.pawns[i].space;
        if self.is_valid(from) {
            if let Ok(t) = self.tile_mut(from) { t.pawns.retain(|&j| j != i); }
        }
        self.pawns[i].space = to;
        if let Ok(t) = self.tile_mut(to) { t.pawns.push(i); }
    }

    /// 0x0062CDD0 `Pawn::ResolvePush()`: se bloqueado, a unidade toma 1 (modo 1) e a casa de destino
    /// é golpeada (`Board::Bump`); senão anda uma casa (0x0062F600).
    pub fn pawn_resolve_push(&mut self, i: PawnIdx) {
        let Some(dir) = self.pawns[i].pending_push_0x978.take() else { return };
        let target = add(self.pawns[i].space, DIR_VEC[dir as usize]);
        if self.pawn_push_blocked(i, dir) {
            let mut bump = Hit::new(Point::INVALID, 1);
            bump.damage = 1;
            self.pawn_apply_damage(i, &bump, 1);
            self.board_bump(target);
        } else {
            self.move_pawn(i, target);
            self.tile_on_enter(target);
        }
    }

    /// 0x0055F680 `Board::Bump(p, dir)` (parte de estado): `Tile::ApplyDamage(SpaceDamage(1), 1)`.
    pub fn board_bump(&mut self, p: Point) {
        let mut h = Hit::new(Point::INVALID, 1);
        h.damage = 1;
        self.tile_apply_damage(p, &h, 1);
    }

    /// 0x0059C5C0 ao entrar numa casa (versão 1): água (sem a exceção +0x23AC) afoga quem não voa e
    /// não tem +0x99D; quem sobrevive deixa ácido na água se estiver com ácido. Buraco: cai quem
    /// não voa. Item na casa: dispara (evento).
    pub fn tile_on_enter(&mut self, p: Point) {
        for i in self.tile_pawns(p) {
            let terr = self.tile(p).terrain;
            let pawn = &self.pawns[i];
            if Self::tile_is_terrain(self.tile(p), TERRAIN_WATER) {
                if pawn.is_flying() && !pawn.status.frozen { continue; }
                if pawn.damage.byte_0x99d != 0 && (!pawn.is_dead() || pawn.is_corpse(self.global_list_has_12)) {
                    if pawn.status.acid { self.tile_set_acid(p, true); }
                    continue;
                }
                self.pawn_kill(i);
            } else if terr == TERRAIN_HOLE {
                if !pawn.is_flying() { self.pawn_kill(i); }
            } else if !self.tile(p).item.is_empty() {
                self.events.push(BoardEvent::ItemTriggered(p));
            }
        }
    }

    /// Resolve já todos os empurrões pendentes, na ordem da arena (`PawnHolder`).
    pub fn resolve_pending_pushes(&mut self) {
        self.pushes_started.clear();
        for i in 0..self.pawns.len() {
            if self.pawns[i].pending_push_0x978.is_some() { self.pawn_resolve_push(i); }
        }
        self.pushes_started.clear();
    }

    /// `SpaceDamage:IsMovement()` (0x00655F10): caminho não vazio, tipo de movimento != 3, tipo 0.
    pub fn hit_is_movement(h: &Hit) -> bool { !h.path.is_empty() && h.move_type != 3 && h.kind == 0 }

    /// Movimento de um `SpaceDamage` (executor 0x005601C0): a unidade de `GetPawn(caminho[0], false)`
    /// (0x00567750) termina na última casa do caminho. O original anima o trajeto (0x00632670 andar,
    /// 0x00632750 salto, 0x006329F0 investida, teleporte em +0x10B4, 0x00636610 escavar); aqui a
    /// unidade chega na hora e passa pelo efeito de chegada.
    pub fn apply_movement(&mut self, h: &Hit) {
        let (Some(&from), Some(&to)) = (h.path.first(), h.path.last()) else { return };
        if from == to { return; }
        let Some(i) = self.get_pawn_at_ex(from, false) else { return };
        self.move_pawn(i, to);
        self.tile_on_enter(to);
    }

    /// Executa uma `DamageList` como o motor faz com `Board:AddEffect`, com o relógio de
    /// `executor::Executor` e os tempos padrão.
    pub fn apply_effects(&mut self, hits: &[Hit]) {
        let mut ex = crate::executor::Executor::default();
        ex.add_effect(hits.to_vec());
        ex.run_until_idle(self);
    }
}
