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
