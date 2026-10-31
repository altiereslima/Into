# RE estática — Breach.exe v1.2.88 — Etapa 05: ocupação, bloqueio e acessores de Pawn

SHA256 `4bd9e4cc43140244d7def56d440346be836efc5b072dd3e8b0dda3ff31fc84b0`, base `0x00400000`.
Desmontagens completas em `etapa05_disasm/` (geradas por `tools/dump_disasm_etapa05.py`).
Implementação: `src/board_occupancy.rs`; testes: `tests/board_occupancy.rs` (11 casos, `cargo test` executado).

## Hierarquia confirmada por RTTI (MSVC)

| Classe | Bases (offset do subobjeto) | vftables |
|---|---|---|
| `Board` | `GridSearchable`@0, `WeaponTarget`@12, `PawnHolder`@16, `Shaker`@20 (sem vftable) | @0 `0x0082A140`, @12 `0x0082A164`, @16 `0x0082A130` |
| `Pawn` (Lua: `BoardPawn`) | `SkillManager`@0, `Shaker`@112 | @0 `0x0082A1F8` |

Nenhuma classe do EXE herda de `Board` nem de `Pawn` (varredura de todos os Class Hierarchy Descriptors),
portanto a resolução de thunks `vcall` pela vftable dessas classes é exata.
Isso também fecha a pendência da Etapa 03: `Board` vftable@0 slot `0x08` = `0x00561370` (`IsValid`).

Slots usados: `WeaponTarget` (Board+12) `0x4C` → `0x0056EE70`, `0x50` → `0x0056EE50`;
`PawnHolder` (Board+16) `0x00` → `0x00567700`; `Pawn` `0x10` → `0x00629530` (`IsDead`), `0x28` → `0x00635FA0` (`GetSpace`).

## Rotinas internas (não expostas à Lua)

| VA | Nome atribuído | Semântica reconstruída |
|---|---|---|
| `0x0059F050` | `Tile::HasPawn(bool)` | `false`: lista `[tile+0xA0..0xA4]` não vazia. `true`: existe pawn com `!IsDead() \|\| IsCorpse()` |
| `0x005AE490` | `Tile::GetPawn(bool)` | `HasPawn(b)` falso → NULL. Senão primeira com `!IsDead()`; se todas mortas, a **última** da lista |
| `0x005AE510` | `Tile::IsTerrainBlocked(pathProf)` | `prof = pathProf & 0xF`; burrower(7) + `"conveyor"` em `std::string [tile+0x27B4]` → bloqueado; depois tabela por terreno (abaixo) |

Tabela de saltos `0x005AE5AC` (índice = terreno−1; terreno fora de 1..9 → livre):

| Terreno | Bloqueia |
|---|---|
| 1 BUILDING | todos exceto PHASING (9) |
| 2 RUBBLE, 6 FOREST, 7 SAND, 8 | ninguém |
| 3 WATER | GROUND (0) e BURROWER (7) |
| 4 MOUNTAIN | todos |
| 5 ICE | BURROWER (7) |
| 9 HOLE | todos exceto 9, 3, 1, 8, 5 |
| 0, 11 FIRE, 12 ACID, 14 LAVA… | ninguém (nesta rotina) |

## Rotinas expostas à Lua (Board)

| Lua | Assinatura C++ (RTTI) | VA | Comportamento | Peculiaridade preservada |
|---|---|---|---|---|
| `IsPawnSpace(p)` | `bool Board::(Point)` | `0x0056EE50` | chama slot 0x4C com `true` | — |
| `IsPawnSpace(p,b)` | `bool Board::(Point,bool)` | `0x0056EE70` | `IsValid` falso → false; senão `HasPawn(b)` | — |
| `GetPawn(p)` | `Pawn* Board::(Point)` | `0x00570E00` | se `IsPawnSpace(p)`, devolve `tile.pawns[0]` | devolve o **primeiro** item mesmo morto |
| `GetPawn(p,b)` | `Pawn* Board::(Point,bool)` | `0x00567750` | `IsPawnSpace(p,b)` e `Tile::GetPawn(b)` | última da lista se todas mortas |
| `GetPawn(id)` | `Pawn* Board::(int)` (thunk, adj 16) | `0x00567700` | busca linear em `[Board+0x3C..0x40]` por `[pawn+0x9A4]` | primeira ocorrência de id repetido |
| `IsPawnAlive(id)` | `bool Board::(int)` | `0x00567840` | `GetPawn(id)` e `!IsDead()` | cadáver não conta como vivo |
| `GetPawnSpace(id)` | `Point Board::(int)` | `0x00567970` | `pawn->GetSpace()` ou `(-1,-1)` | lê a posição gravada no pawn |
| `GetPawnTeam(p)` | `int Board::(Point)` | `0x00568950` | `HasPawn(true)` → `tile.pawns[0]->team`; senão 2 | 2 = `TEAM_NONE` (constante do EXE); inválida usa sentinela |
| `IsBlocked(p,prof)` | `bool Board::(Point,int)` (thunk slot 0xC) | `0x0055F030` | inválida → true; terreno → true; pawn → true exceto `prof==9`, que só bloqueia se time==6 | `prof==9` sem máscara 0xF |

## Rotinas expostas à Lua (BoardPawn = `Pawn`)

Acessores triviais (leitura/escrita direta de campo) — contados à parte no progresso:
`GetHealth 0x004F6410 [+0x8A8]`, `IsDead 0x00629530 ([+0x8A8] <= 0)`, `IsDamaged 0x00638D50 ([+0x8A8] != [+0x8AC])`,
`GetTeam 0x00639FC0 [+0xB0]`, `SetTeam 0x0062F2D0`, `IsPlayer 0x0062F840 (team==1)`, `IsEnemy 0x0062F870 (team==6)`,
`GetId 0x006370D0 [+0x9A4]`, `IsMech 0x00639040 [+0x9E4]`, `SetMech 0x00639050 (grava 1)`, `GetSpace 0x00635FA0 [+0x8EC,+0x8F0]`.

`IsCorpse 0x00629540` — **parcial**: fluxo completo reconstruído, mas depende de `Pawn::0x00639E10(12)`
(não reconstruída; tratada como dado de entrada) e da lista global de inteiros `0x008CF84C..0x008CF850`.

```
if ((mech[+0x9E4] || [+0xF80]) && [+0x9E0] not in {2,3,4}) return true;
if ([+0x10E8] == 12 || global_list contains 12) return this->0x639E10(12);
return false;
```

## Constantes usadas (extraídas do EXE, `lua_global_values.csv`)
`TEAM_PLAYER=1 TEAM_NONE=2 TEAM_MECH=4 TEAM_ENEMY=6`; `PATH_GROUND=0 FLYER=1 MASSIVE=2 PROJECTILE=3 ROADRUNNER=4 BURROWER=7 PHASING=9`;
`TERRAIN_ROAD=0 BUILDING=1 RUBBLE=2 WATER=3 MOUNTAIN=4 ICE=5 FOREST=6 SAND=7 HOLE=9 FIRE=11 ACID=12 LAVA=14`.
O teste `rust_constants_match_values_extracted_from_exe` compara o Rust com esse CSV.

## Limites
- Prova estática + testes de unidade do modelo. Não houve execução lado a lado com o jogo original.
- Os valores 5 e 8 de `pathProf` aparecem na tabela do buraco mas não têm nome Lua registrado.
- `[+0xF80]`, `[+0x9E0]`, `[+0x10E8]` continuam sem nome semântico.
