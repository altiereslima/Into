# Fila de uso: rotinas lidas na 1.2.93

Rotinas reconstruídas na ordem da fila de `itb-uso` (`docs/relatorios/uso_missao.txt`). Endereços da 1.2.93. O
`libmain.so` serviu de guia (`re/tools/chamados_android.py`); a prova é o assembly do PC. Código em
`itb-core/src/board_mission.rs`, `itb-lua/src/world.rs`, `game.rs` e `records.rs`.

## Bloco 1
Conferido contra o jogo: `Mission_Volatile` no mapa `mix6`, 0 divergências em 66.302 linhas (seções `casa`,
`com_unidades`, `entrada`, `unidade` e `objetivo`).

| Rotina | Regra |
|---|---|
| `Board::IsPowered(Point)` 0x5700A0 | casa por `GetSpace`; prédio único (+0x27AC != 0): vida atual == máximo (+0x78, +0x7C); senão +0x2710 e terreno 1 |
| `Board::IsFrozen(Point)` 0x56D650 | +0x2AA1, ou `BoardSpace::HasPawn(true)` (0x59FEB0) e a primeira unidade da lista da casa (+0xA0) com +0x8D1 |
| `Board::IsDangerousItem(Point)` 0x5725C0 → `BoardSpace::IsDangerousItem` 0x5A11E0 | item (+0x2B04) e algum destes diferente do inofensivo: +0x288C, +0x2890 != 4, +0x2894, +0x289C, +0x28A8, +0x2938, +0x28AC, +0x28A0 |
| `Board::GetEnemyCount()` 0x566FF0 | `GetPawnCount(6)` |
| `Board::GetPawnCount(int)` 0x571630 | na lista Board+0x3C, tipo != "BonusDebris" (0x46C1D0 com tamanho 11): `vivas` += `IsTeam(t)` e não `IsDead` (slot 0x10); `congeladas` += `IsTeam(t)` e +0x8D1, sem olhar morte. Time −1 soma a fila Board+0x2D44 e registra "Already Spawning = %d". Sem vivas, 0; senão `cvttss2si(max(vivas − congeladas / 3.0f, 1.0f))` (0x838F00 = 3.0, 0x838E6C = 1.0) |
| `Board::GetPawnCount(string)` 0x567000 | na lista, `GetType().substr(0, tamanho − 1)` (0x4930F0) igual ao nome; mortas contam |
| `Board::IsTargeted(Point)` 0x56E6D0 | para cada unidade da lista sem o slot 0x18: `Skill::ComputeAffectedPoints` (0x6656A0) nas armas com efeito, depois `SkillManager::IsTargeted` 0x6260C0, que exige ataque na fila (0x626430) e consulta a arma (+0x28) na casa alvo (+0x20/+0x24). **Parcial:** o Rust não tem a fila de ataques e responde falso |
| `Pawn::IsNonGridStructure()` 0x63B750 | `LuaData::GetBool("NonGrid")` na tabela do tipo (+0x94) |
| `Pawn::GetCustomPositionScore(Point)` 0x639C00 | nome do tipo (0x449370) e `CallMethod(tipo, "GetPositionScore", p)` por 0x641980 (`proxy_function_caller` com `lua_getfield` de "CallMethod", 0x598460); erro ou não número dão 0. Nenhum script define `GetPositionScore`, então é sempre 0 |
| `RepObj::Failed()` 0x5BA850 | `RepObj(texto, p1, p2, 0, potential)` (0x5B9670), categoria (+0x48) e +0x58 copiados, +0x55 = 1 |
| `GameMap::AddObjective`, 8 formas (0x60DA30..0x60E0C0) | completam tipo 0, a 0, b 1, c 0; com `BoardPlayer` (GameMap+0xC204), `BoardPlayer::AddObjective` (0x594C60 / 0x594CF0) → `ObjectiveList::AddObjective` 0x5B6A90 (BoardPlayer+0x235C), que acrescenta uma entrada de 0x58 bytes (0x5BBA30) |

O `libmain.so` confirma a estrutura: `Board::GetPawnCount(int)` chama `Pawn::IsFrozen`, `GetType`, `IsTeam` duas vezes e
registra "Already Spawning"; `Board::IsTargeted` chama `SkillManager::BoardUpdate` e `SkillManager::IsTargeted`;
`Pawn::IsNonGridStructure` chama `LuaData::GetBool`; `GameMap::AddObjective` passa por `BoardPlayer::AddObjective` e
`ObjectiveList::AddObjective`. No Android, `Pawn` herda de `SkillManager` no deslocamento 0, e a fila de ataque fica em
`SkillManager+0x20/+0x24/+0x28`.

Efeito na fila de uso: de 42 para 36 stubs. Os erros de `UpdateSpawning` caíram de 320 para 4, e apareceram stubs que
ficavam escondidos atrás dos que devolviam `nil` (`BoardPawn:IsRanged`, `Board:GetDistanceToBuilding`,
`Board:GetDistanceToPawn`).

## Bloco 2
Conferido contra o jogo: `Mission_Tanks` no mapa `any1`, 0 divergências em 66.666 linhas (seções `casa`, `com_unidades`,
`entrada`, `unidade` e `unico`, que compara o sorteio de `AddUniqueBuilding` com o `rand()` sincronizado).
A análise do `.so` (fora do repositório) foi o guia; três rotinas diferem do Android, e vale o PC.

| Rotina | Regra |
|---|---|
| `Board::GetDistanceToBuilding(Point)` 0x573EA0 | menor Manhattan até a lista de prédios (Board+0x2C44), começando em `INT_MAX` |
| `Board::GetDistanceToPawn(Point, int)` 0x573C00 | 0x4CEA50 (mapa de distâncias) chama o Dijkstra 0x4CEC40 com perfil = o próprio `int` e alcance 6 (0x4CEC40 recebe perfil, alcance, ?, x, y; o alcance é o que se compara com a distância em 0x4CEF6B). Menor distância até a casa (`GetSpace`, slot 0x28) de uma unidade da lista com `IsTeam(int)`, ou `INT_MAX`. Os scripts passam 1 ou 6: voador ou saltador |
| `Board::IsDamaged(Point)` 0x5747C0 | vida atual (+0x78) != máximo (+0x7C). No Android passa por `BoardSpace::IsDamaged` |
| `Board::GetMechDamage()` 0x5739C0 | soma de `0x6350A0() − vida (+0x8A8)` dos mechs (+0x9E4) da lista; 0x6350A0 recalcula o máximo por pilotos (`Zoltan_Skill`, 0x635960) e o devolve |
| `Board::AddUniqueBuilding(string)` 0x5744F0 | cópia da lista de prédios; enquanto houver: `rand() % n`, tira a casa; se o nome de prédio único da casa está vazio, `BoardSpace::SetUniqueBuilding` 0x5A1990 e devolve a casa; sem nenhuma, (-1, -1) |
| `BoardSpace::SetUniqueBuilding(string)` 0x5A1990 | máximo 1, vida presa em 0..1, nome em +0x279C (o tamanho fica em +0x27AC, que o Rust chamava de `unique_0x27ac`), imagens `_on`/`_broken` |
| `Board::MarkSpaceDesc` 0x565BC0 `(Point, string)` e 0x565C90 `(…, bool)` → `BoardSpace::MarkDescription` 0x5AEB00 | descrição em +0x26F8 (a que `IsCrackable` compara com "supervolcano") e +0x2BB9 = 0 |
| `Board::IsBusy()` 0x569850 → `GetBusyState` 0x569860 | animações, efeitos e filas pendentes. **Parcial:** o executor do Rust termina tudo na hora, então é falso |
| `Pawn::IsRanged()` 0x63CF50 | `LuaData::GetInt("Ranged") == 1` (o Android devolve o inteiro) |
| `Pawn::IsTempUnit()` 0x63A960 | `LuaData::GetBool("TempUnit")` |
| `GameMap::GetEventCount(int)` 0x60BD70 | lista global de eventos (0x8B8400) com verificação de faixa, senão "Error: Tried to retrieve invalid event id" e 0. **Parcial:** os eventos não existem no Rust; responde 0. No Android é sempre 0 |

Efeito na fila de uso: de 36 para 26 stubs; os 10.932 erros em `ScorePositioning` sumiram.

