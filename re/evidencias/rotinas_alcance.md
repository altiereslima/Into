# Etapa 10 — alcance (Dijkstra), caminho (A*), terreno e dano letal

Todas as rotinas abaixo foram lidas no assembly de `Breach.exe` v1.2.88 (capstone), com o angr só como
apoio de leitura. Código Rust: `itb-core/src/{pathfinding,board_terrain,damage}.rs`.

## Board (vftable @0 = 0x0082A140; WeaponTarget @+12 = 0x0082A164)
| Slot | Alvo | Papel |
|---|---|---|
| @0 0x04 | 0x0055DC10 | `GetSize` |
| @0 0x08 | 0x00561370 | `IsValid` |
| @0 0x0C | 0x0055F030 | `IsBlocked(x,y,prof)` |
| @0 0x10 | 0x00573540 | `IsEdgeBlocked(x,y,dir,prof)`: bytes da casa +0x2AC4+dir (dir 4 = qualquer); perfis 1/3/5/6 ignoram |
| @0 0x14 | 0x00561300 | `IsFree(x,y,prof)` |
| @0 0x18 | 0x005612A0 → 0x0059F3A0 | custo de entrar: 1000 se `prof == 3` (sem máscara) e terreno 1; senão 1 |
| @0 0x1C | 0x00561280 | custo de sair: sempre 0 |
| WT 0x50 | 0x0056EE50 | `IsPawnSpace(p)` = slot 0x4C(p, true) |
| WT 0x7C | 0x0055F0D0 | `IsTerrain(p, t)` → 0x005AE5D0 |

## GetReachable — 0x00573280 → 0x004CDA00 → 0x004CDE60
* 0x004CDE60 (Dijkstra), cache em Board+4 (`std::map<Point, {Point, map prev, map dist, int range, int prof}>`),
  reaproveitado quando origem, alcance e perfil coincidem.
* Inicializa `dist` (`std::map<Point,int>`, ordem x→y) com `INT_MAX` e `prev` com (-1,-1) para todas as casas.
* Fila: `std::set<{Point, float}>` ordenado por (float, x, y). Melhoria: `find` + `erase` da entrada antiga
  ({p, (float)dist antiga}) e `insert` da nova.
* Ao tirar da fila: `if dist[cur] >= range → para`. Vizinhos na ordem da tabela 0x008CF9AC (UP, RIGHT, DOWN,
  LEFT); descarta `IsEdgeBlocked(cur,dir,prof)` e `!IsFree(n,prof)`. Custo: 1 saindo da origem, senão
  `slot0x1C(cur) + slot0x18(n)`.
* 0x004CDA00: origem inválida → vazio; depois percorre `dist` (ordem x→y) e guarda `dist <= range && !IsBlocked`.
* 0x00573280: se `prof != 3` (inteiro), remove `IsBlocked(p)` e água crua (terreno 3 em +0x2AE0) quando
  `prof & 0xF == 6` (saltador).

## GetPath — 0x005733D0 → 0x004CE630 (A*)
* `from == to`, `!IsValid(from)` ou `!IsValid(to)` → vazio.
* Fila: `std::priority_queue` (vector de {Point, float}); `push_heap` 0x004CF0B0 → `_Push_heap_by_index`
  0x004D1A30; `pop_heap` em 0x004CE771 → `_Pop_heap_hole_by_index` 0x004D1AD0. Predicado: `a.f > b.f`,
  empate `a.x > b.x`, empate `a.y > b.y` (topo = menor).
* Mapas `unordered_map` (hash `x ^ ((x<<6) + (x>>2) + 0x9E3779B9 + y)`): fechados, vistos, `g`, `came_from`.
* Laço: tira o topo; se for o destino reconstrói; marca fechado; vizinhos UP, RIGHT, DOWN, LEFT:
  `IsEdgeBlocked` → pula; `!IsFree(n) && n != to` → pula (destino ocupado é aceito); fechado → pula;
  `ng = custo + g[cur]` (custo como no Dijkstra); se já visto e `ng >= g[n]` → pula; senão
  `came_from[n] = cur`, `f = (float)(|to.y−n.y| + |to.x−n.x|) × 1.01f (0x00837E1C) + (float)ng`, empilha,
  marca visto, `g[n] = ng`. Não há decrease-key nem teste de fechado ao tirar da fila.
* Reconstrução: do destino até a origem por `came_from` (para se x ou y < 0), inverte.

## IsTerrain — 0x005AE5D0
3: falso se byte +0x23AC e int +0x23CC < 15, senão terreno == 3 · 14: `IsTerrain(3)` e byte +0x2AC8 ·
18: `IsTerrain(5)` e campo +0x7C (objeto de vida em +0x78) == 1 · 16: byte +0x2B26 (`IsCracked`) · outros: terreno == t.

## IsCrackable — 0x00568860 → 0x005AE6B0
Falso se: unidade (HasPawn(true)) cujo tipo (+0x94) é "SatelliteRocket"; string +0x26F8 == "supervolcano";
int +0x2BA0 != 0; "conveyor" na string +0x27B4; água (regra do `IsTerrain(3)`); lava; terreno 9 ou 1.

## Outras consultas de casa
`IsPod` 0x00571680 (Board+0x2D70 == p e casa +0x2AE4 == 1) · `IsItem` 0x005716F0 (+0x2B04 != 0) ·
`IsDangerous` 0x005717A0 (byte +0x2AF0 ou p nas listas Board+0x7460 / +0x7470) · `IsSpawning` 0x0056D5B0
(byte +0x27E8 ou p na lista Board+0x2D50).

## IsDeadly — 0x005739E0; Pawn::WouldDie — 0x006362A0
* Atacante com `IsBoosted` (0x006313E0: byte +0x8D5, ou 0x00638C70(11), ou vftable[1]("Arrogant_Boost")
  com vida cheia): se dano != 500, dano > 0 e != 1000 → +1; dano original em −9..−1 → −1.
* Sem `IsPawnSpace(loc)` → falso. Senão `GetPawn(loc)` (primeiro da lista) → `WouldDie({sd})`.
* WouldDie: escudo = `IsShield` (0x00633AD0: 0x0062F2F0 ou byte +0x8D4); gelo = byte +0x8D1. Morto → falso.
  Por dano: escudo, senão gelo, absorvem dano > 0 e != 500 (sobra só 1000). 500 → 0. Se dano > 0:
  `IsTerrain(18)` e não voa → morre se byte +0x99D == 0; `IsTerrain(16)` e não voa → morre;
  blindagem (0x0063C100) −1 (mín. 0); ácido (+0x8D3) ×2. Vida −= dano. Resultado: vida <= 0.

## Pawn
`IsGrappled` 0x00631490 (byte +0xF81 e vivo) · `GetMoveSpeed` 0x0063A860 (0 se morto, agarrado ou sem energia
+0x10C0; +0xA64 se > 0; senão `GetBaseMove` 0x0063A8F0) · `IsTeleporter` 0x0063AF60 / `IsJumper` 0x0063AFC0
(propriedade do tipo e vivo) · `IsBurrower` 0x006365D0 (propriedade crua) · `IsFlying` 0x0063AC00
(`IsAbility("Flying")` ou byte +0x1314, e vivo) · `GetType` 0x0063F3B0 (string +0x94).
Correção da Etapa 09: `GetPathProf` usa `IsJumper()` e `IsFlying()`, que exigem unidade viva.

## SkillEffect
`AddAirstrike` 0x00653F20: construtor 0x00652610(0, p.y, imagem, 3) — casa (0, p.y), imagem +0x68, tipo 3,
fDelay −2 — e `AddDelay(p.x × 0.125 + 0.375)`. `AddReverseAirstrike` 0x00654030: casa (p.x, 0), tipo 6,
atraso com p.y. `AddDropper` 0x00654140: cópia do SpaceDamage com imagem, tipo 4, fDelay 0.
`AddEmitter` 0x00654500: `AddAnimation(p, "add_emitter_" + nome)`. `AddVoice`/`AddQueuedVoice`
0x00652AD0/0x00652BC0 → 0x00655FA0: `SpaceDamage()` com `sScript = PrepareVoiceEvent("evento",id)`.

## IsPassiveSkill — 0x0065AEB0
Verdadeiro se a string global 0x008D0E94 começa com o nome, ou se algum objeto da lista 0x008D0E48 aceita
o nome (0x00625320, não reconstruída — dado do host).
