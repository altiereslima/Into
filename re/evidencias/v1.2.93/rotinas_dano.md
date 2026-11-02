# Dano aplicado e defesa da rede: leitura e teste na 1.2.93

O código de dano do Rust (`itb-core/src/apply.rs`) foi escrito a partir da 1.2.88. Em vez de reler rotina a rotina, a
seção `aplicado` do teste diferencial aplica `Board:DamageSpace` no jogo original e compara o estado da casa e da unidade
logo depois. Casos: prédio (1 e 4 de dano), montanha (dois golpes), floresta (dano e fogo), água (fogo e gelo), chão
(fumaça, ácido, rachadura, `iTerrain` buraco e água), inimigo (dano, morte, fogo, ácido, gelo, escudo), mech (dano e
morte) e um golpe de 1 em cada prédio restante.

## Resultado
Duas partidas (`Mission_Mines`/`any25` e `Mission_Train`/`trainAE3`). Tudo igual, salvo:

| Caso | Original | Rust antes | Causa |
|---|---|---|---|
| prédio povoado com dano | às vezes intacto | sempre perdia vida | defesa da rede, abaixo; corrigido |
| chão com `iTerrain = 9` | ainda chão logo depois | buraco na hora | buraco animado, já em `docs/DIVERGENCIAS.md` |

Com a correção, a última partida deu 1 divergência em 67.704 linhas (só o buraco). A sequência de 6 prédios sorteou igual,
com dois prédios resistindo e a energia terminando em 0/7 nos dois lados.

## `BoardSpace::DamageSpace`, ramo de prédio (0x5AD383)
Dano efetivo = `min(dano, vida)`. Com terreno 1 e dano efetivo > 0: se a casa é povoada (+0x2710), a flag global
0x8B8460 está desligada (não identificada) e há `PowerBar` (0x8B4F50), chama `PowerBar::Damage(dano efetivo)` em
0x5AD3FE. Se a rede resiste, só toca o tutorial `Tutorial_Resisted` e som, e o prédio fica intacto.

## `PowerBar` (GameMap+0xC03C)
| Campo | Papel |
|---|---|
| +0x04 / +0x08 | energia atual e máxima (`Game:GetPower`) |
| +0xE0 | defesa-base; o construtor (0x5C2B9A) grava 15 |
| +0xEC | bônus de defesa; o construtor zera; gravado em 0x5C2653 e 0x5C133B (não lidos) |
| +0x135 | com ela ligada a rede nunca resiste |
| +0x138 | `overflow` no save; soma na defesa |
| +0x13C | `seed` no save: semente da defesa |

`0x5C5950` = defesa = `+0x138 + +0xEC + +0xE0`.

## `PowerBar::Damage(int)` (0x5C59F0)
1. `srand(+0x13C)`.
2. `rand() % 100 < defesa` e +0x135 desligada: resiste (evento 9, som `/ui/battle/resisted`), devolve falso.
3. Senão 0x5C5400(−dano, 0): eventos e `ValueBar::AnimatedModify(−dano)` (0x4E0E80) na energia; devolve verdadeiro.
4. `+0x13C = unique_rand(+0x13C)`: `rand()`, repetido até 10 vezes enquanto igual à semente; se ainda igual, +2.
5. `srand((int)(QueryPerformanceCounter / QueryPerformanceFrequency * 1000.0))`: o `rand()` global volta ao relógio.

Por isso o teste diferencial lê a semente do save do original (`["seed"]`, gravado por 0x5C5DC5 e lido por 0x5C5F5C), e
nada que dependa de `rand()` pode vir depois de um golpe em prédio na coleta.

## No Rust
`itb-core/src/game.rs` (`PowerGrid`, compartilhado entre o `GameState` e o tabuleiro da missão) e `damage_building` em
`apply.rs`. O `srand` pelo relógio usa `PowerGrid::clock_seed`, que o host escolhe.
