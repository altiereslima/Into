# Etapa 11 — aplicação de dano, empurrão e modificadores de casa/unidade

Lido no assembly de `Breach.exe` v1.2.88 (capstone), com angr como apoio de leitura; offsets de
`SpaceDamage` vindos do inventário luabind (`property_offset`). Código: `itb-core/src/apply.rs`.

## SpaceDamage (offsets usados aqui)
loc 0x00 · iDamage 0x08 · iPush 0x0C · iShield 0x10 · iFire 0x18 · iFrozen 0x1C · iInjure 0x20 · iSmoke 0x24 ·
iAcid 0x28 · iCrack 0x2C · kind 0x98 · sPawn 0xA4 · iPawnTeam 0xBC · bEvacuate 0xC7 · fDelay 0xC8 · caminho 0xCC ·
tipo de movimento 0xD8 · iTerrain 0xDC · sScript 0xE0 · origem da teia 0x110 · sItem 0x118 · 0x130.

## Board::DamageSpace — 0x0055F200 (SpaceDamage) / 0x0055F5B0 (Point, int)
sScript não vazio → executa · som/impacto (visual) · casa válida → `Tile::ApplyDamage(sd, 0)` · iTerrain == 1 →
0x005723F0 · sPawn não vazio → cria unidade via `PAWN_FACTORY` (0x00641560) · no fim limpa o cache do Dijkstra
(Board+4, 0x004CF5D0). A versão (Point, int) aplica `SpaceDamage(int)` e também limpa o cache.

## Tile::ApplyDamage — 0x005ABA90 (ret 0x138)
1. modo = 2 se sd+0x130 == 2, senão o argumento (0 normal, 1 golpe de empurrão).
2. Estatísticas de conquistas (Conveyor_, LightningBolt, windpush_) — ignoradas.
3. +0x2630 = 1. iShield > 0 → 0x0059EBE0 (prédio/montanha: +0x2AA4 = 2; senão `SetShield(true)` na unidade);
   iShield < 0 → 0x0059EE20 (+0x2AA4 = 0 e `SetShield(false)`).
4. Se escudo da casa/unidade, gelo do prédio (+0x2AA1), gelo da unidade (+0x8D1) ou "Passive_PlayerTurnShield"
   (0x0062C5F0): dano > 0 e != 500 → remove o escudo (0x0059EE20) ou o gelo (0x0059E8A0(0)) e zera o dano
   (exceto 1000). Com escudo ainda presente, iFrozen = 0.
5. Com `HasPawn(true)`: para CADA unidade da lista, `Pawn::ApplyDamage(cópia alterada, modo)`.
6. 500 → 0. Dano > 0: (modo != 1) item dispara (0x005A03E0), cápsula destruída; +0x2AA0 → 0x005A94B0(0);
   rachado (modo != 1) → `SetTerrain(9)` e pula o resto do bloco. Senão por terreno: gelo (modo != 1) e montanha →
   0x005AAE10; floresta (modo != 1, iTerrain 10) → `SetFire(1)`; areia (modo != 1) → fumaça + chão; demais →
   prédio: `min(dano, vida)` via `Health::Add` em +0x74, rede elétrica, alertas (chance de resistir 0x005C4B90).
   Vida 0 em prédio não único (+0x27AC) → `SetTerrain(2)`.
7. iTerrain != 10: 3/14/9 iniciam animações; para 9 em casa que não é buraco a troca é ADIADA; 1 → mata as
   unidades e `AddBuilding` (0x005A10B0); outros → `SetTerrain` (0x005A0CB0).
8. iCrack 1 → 0x005AAC40(1). iFire 1 → item, `SetFire(1)` da casa e de cada unidade; 2 → apaga.
   iAcid 1 → ácido na casa só sem unidade (ou com escudo). `Tile::Push(iPush)` (0x005ADBB0).
   iSmoke 1 → 0x005AB740(1,0); 2 → +0x2B25 = 0. iFrozen 1/2 → 0x0059E8A0. bEvacuate → `Retreat` de cada unidade.
   Teia (origem >= 0), sItem → 0x0059FDF0.

## Pawn::ApplyDamage — 0x006295B0 (ret 0x138)
Morto: só mech continua (com iAcid ignorado). Escudo (`IsShield`) absorve dano > 0 != 500 (sobra 1000) e
`SetShield(false)`; senão gelo absorve e `SetFrozen(false)`. 500 → 0. Dano > 0: modo 0 → blindagem (0x0063C100)
−1 e ácido ×2, `SetInfected(false)`; modo 1 → `SetInfected(false)` e "Passive_ForceAmp" +1 contra `IsTeam(7)`;
modo 2 → `SetInfected(false)`. iFire 1/2, iInjure (+0x8D6). `ChangeHealth(−dano, 1, modo)` (0x0062BF90).
iAcid 1/2. Dano negativo (cura): apaga fogo, gelo, infecção, ácido. Morreu agora → +0x921.

## Vida — 0x0062BF90 e 0x004DFFC0
`ChangeHealth`: morto e não cadáver → nada; dano com "PlayerTurnShield" → nada; `Health::Add`; "Burrows" vivo
ferido (fora de chão rachado, ou modo 1) se enterra (0x00636610); "Retaliation". `Health::Add(delta)` em
{+4 atual, +8 máximo}: delta limitado a [−atual, máximo − atual], resultado preso em [0, máximo].

## Modificadores de unidade
`SetFire` 0x00639060 (escudo bloqueia; limpa infecção; imune 0x00638BD0 só pega com "Passive_FireBoost" em mech;
tipo +0x374 ignora; pegar fogo apaga gelo) · `SetFrozen` 0x00636970 (+0x8D1; congelar apaga fogo e infecção) ·
`SetAcid` 0x006397C0 (escudo bloqueia; "Thick" nunca) · `SetInfected` 0x00637210 (+0x8DC) · `SetShield`
0x006335F0 (+0x8D4) · `Kill` 0x00639A50 (+0x921, −100 de vida, sem infecção).

## Modificadores de casa
`SetFire` 0x005AB580 · `SetSmoke` 0x005AB740 · `SetAcid` 0x005AABD0 · `SetCracked` 0x005AAC40 · dano em gelo/montanha
0x005AAE10 · `SetFrozen` 0x0059E8A0 · `SetTerrain` 0x005A0CB0 (14 lava, 11 fogo, 12 ácido, 15 perigo, 16 racha;
gelo/montanha recebem `Health(2)` 0x004DFE20; água/buraco apagam fogo, rachadura e +0x2BA0) · `AddBuilding`
0x005A10B0 · +0x2AA0 0x005A94B0.

## Empurrão
`Tile::Push` 0x005ADBB0: DIR_NONE ou prédio → nada; empurra a ÚLTIMA unidade da lista; DIR_FLIP (6) também chama
0x005ADA60. `Pawn::Push` 0x0062CC00: FLIP → 0x00625710 se +0x91F (`IsFlippable`, inverso de `IgnoreFlip`) e viva,
lido na 1.2.93 e no Android 1.2.92; estável (+0x91E == 0, campo `Pushable`) → nada; grava +0x978 se o destino
bloqueia ou é válido. `Pawn::ResolvePush` 0x0062CDD0 (depois da animação): bloqueado (0x0062CAA0 → 0x00571B20 →
0x005AAA10: prédio, montanha, unidade sólida 0x0062CB10 sem +0x2640, parede 0x0059F7C0) → a unidade toma
`SpaceDamage(1)` no modo 1 e `Board::Bump` 0x0055F680 aplica `SpaceDamage(1)` modo 1 na casa de destino;
livre → 0x0062F600 (SetSpace, desloca o alvo do ataque preparado, efeito de chegada 0x0059C5C0).
