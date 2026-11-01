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

## Bloco 3
Conferido contra o jogo: `Mission_Train` no mapa `trainAE4`, 0 divergências em 67.433 linhas, com a seção nova
`mutacao` (acidifica, enfumaça, racha, congela e tira o escudo de 5 casas e congela um mech, lendo o estado a cada passo).

| Rotina | Regra |
|---|---|
| `Board::IsSafe(Point)` 0x56CE40 | **verdadeiro quando seguro**; falso, nesta ordem, com `IsPod`, ácido (+0x2AF1), `IsFire` (0x568AC0), `IsTargeted`, vetores de projéteis e alertas da casa não vazios (+0x2644/+0x2648 e +0x7E0/+0x7E4, de `OnRenderProjectiles` e `AddAlert`), terreno 3 cru (+0x2AE0 por 0x55EAB0), `IsSpawning`, `IsSmoke` (0x566260) e o slot 0x0C (`IsBlocked(p, 0)`); no fim `!IsTargeted(p)`. A análise do Android não conseguiu fixar o sentido do retorno |
| `Board::StopWeather()` 0x56D5C0 | `RainCloud::SetWeather(0, 0, (0,0), (8,8), -1.0f)` em Board+0x2DA0 |
| `Board::GetBuildings()` 0x5663F0 | cópia da lista Board+0x2C44 como `PointList` |
| `Board::SetCustomTile(Point, string)` 0x565F00 | nome em +0x27B4 |
| `Board::SetAcid` 0x566150, `SetSmoke` 0x566060, `SetCracked` 0x56A7E0, `SetFrozen` 0x56D5F0, `RemoveShield` 0x569770 | só repassam para `BoardSpace::SetAcid` (embutida), `SetSmoke(bool, bool)` 0x5AC5A0, `SetCracked` 0x5ABAA0, `SetFrozen` 0x59F700 e `RemoveForceField` 0x59FC80, que o Rust já tinha da 1.2.88 |
| `Pawn::SetFrozen(bool)` 0x637810 | grava +0x8D1; ao congelar, `SetOnFire(false)` (0x639F00) e `SetInfected(false)` (0x6380B0); o resto é animação e som |

Efeito na fila de uso: de 26 para 19 stubs.

## Bloco 4
Conferido contra o jogo em duas partidas: `Mission_Dam`/`opendam7` (tudo igual, menos ids absolutos de unidade, que o
contador do original acumula desde o menu) e `Mission_Mines`/`any25`, já com ids relativos: 0 divergências em 66.525
linhas. A seção `bloco4` põe itens, limpa casas, usa zonas, mede `GetDeployLocScore` (nota, 3 `rand()` e 1 id) e tira um
mech do tabuleiro. A análise do `.so` (`so_analise/bloco4_pc.md`, `rand_unidade.md`, `stubs_pawn.md`) foi o guia, com
duas correções: em `ClearSpace`, Board+0x2D50/+0x2D44 são a lista e a fila de surgimento (não paredes), e
`AddPawn(nome, zona)` não tem ambiguidade de string (é o padrão do MSVC de montar antes o argumento da chamada de fora).

| Rotina | Regra |
|---|---|
| `Board::RemovePawn(Pawn*)` 0x561D20 | casa da unidade (slot 0x28): +0x2630 = 1 e sai da lista da casa (`BoardSpace::FindPawn` 0x5A0060); o mesmo nas casas extras (slot 0x14, ainda não no Rust); limpa o cache do Dijkstra (Board+4); +0x944 = 0 com `SetWeaponTarget(0)`; sai da lista Board+0x3C |
| `Board::RemovePawn(Point)` 0x561F00 | enquanto `HasPawn(true)` na casa, tira a primeira da lista da casa |
| `BoardSpace::Clear()` 0x5A0330 | `Pawn::Kill(false)` embutida em cada unidade da casa, `OnInit(0, 0)` (0x5A0270) e `custom` (+0x27B4) vazio |
| `Board::SetWall(bool, Point, int)` 0x574450 | parede `d` da casa e a oposta `(d+2)%4` na vizinha |
| `Board::ClearSpace(Point)` 0x565D40 | `RemovePawn(p)`, `Clear`, 4 paredes desligadas e, se a casa está na lista de surgimento, sai dela e da fila paralela, com +0x27E8 = 0 |
| `Board::AddPawn(Pawn*, string)` 0x56EE40 | zona por `std::map::operator[]` (ausente entra vazia); casas com `!IsBlocked(p, 0)`; `rand() % n`; `AddPawn(pawn, casa)`; sem casa, log e (-1, -1) |
| `Board::AddPawn(string, string)` 0x56ECE0 | `AddPawn(CreatePawn(nome, 2), zona)` |
| `Board::SpawnPawn(Pawn*, string)` 0x5690E0 | casas com `!IsBlocked(p, GetPathProf)`; `rand() % n`; `SpawnPawn(pawn, casa)`; sem casa, `SpawnPawn(pawn, (-1,-1))` |
| `Board::SpawnPawn(string, string)` 0x569250 | `SpawnPawn(CreatePawn(nome, 2), zona)` e devolve o id (+0x9A4) |
| `Board::SetItem(Point, string)` 0x5727C0 → `BoardSpace::SetItem` 0x5A0C50 | nome vazio: nada. Nome em +0x2AF4 (tamanho +0x2B04, o antigo `item_0x2b04`), imagem `Image` em +0x2880 e `SpaceDamage` da chave `Damage` em +0x2884 |
| `BoardSpace::IsDangerousItem` 0x5A11E0 | com os nomes do registro luabind: `iDamage` (+0x08), `iPush != 4` (+0x0C), `iShield` (+0x10), `iFire` (+0x18), `iSmoke` (+0x24), `sPawn` não vazio (+0xA4, tamanho em +0xB4), `iAcid` (+0x28), `iFrozen` (+0x1C) |
| `Board::SetTerrainIcon(Point, string)` 0x568880 | ícone em +0x2684 |
| `Board::GetDeployLocScore(Point)` 0x56CD40 | com unidade na casa, −10; senão `Pawn("Wall", 6)` na pilha (3 `rand()`: duas habilidades do piloto padrão em 0x623BB5 e a semente em 0x628D9A; e um id), `AddPawn`, `ScorePositioning(p, pawn)` em Lua, `RemovePawn` |

Os 3 `rand()` de toda criação de unidade, que o projeto não tinha localizado, são do piloto padrão `Pilot_Artificial`:
duas habilidades sorteadas em `Pilot::OnInit` (0x623BB5, duas vezes) e a semente da unidade (0x628D9A, +0x924).

Efeito na fila de uso: de 19 para 14 stubs; nas 85 missões sobram 5 erros de script.

## Bloco 5
Conferido contra o jogo: `Mission_Dam` no mapa `opendam19`, 0 divergências em 67.813 linhas, com a seção `bloco5`
(paredes por direção e perfil, dois pares de teleporte, aviso de ambiente, escudo, ácido, energia, minas e `VoiceEvent`).
Guia: `so_analise/bloco5_pc.md`.

| Rotina | Regra |
|---|---|
| `Pawn::SetShield(bool)` 0x634490 (no Android, `Pawn::SetForceField`) | desligar com `IsTurnShield` (0x62D4A0) não faz nada; ligar grava +0x8D4 só se a unidade não está morta; desligar grava 0; o resto é animação e som |
| `Board::AddShield(Point)` 0x569720 | `BoardSpace::SetForceField` 0x59FA40, que o Rust já tinha (`tile_shield_on`) |
| `Pawn::SetPowered(bool)` 0x637580 | +0x10C0; ao ligar, um parâmetro visual de 16 bytes em +0x10D4 |
| `Pawn::SetAcid(bool)` 0x63A660, `SetActive` 0x634C00 | iguais ao que o Rust já tinha (`pawn_set_acid`, `set_active`) |
| `Pawn::SetMissionCritical(bool)` 0x637CB0 | +0x10C2 |
| `Pawn::IsAvoidingMines()` 0x6384B0 | `LuaData::GetBool("AvoidingMines")` |
| `Board::IsEnvironmentDanger(Point)` 0x573240 | tamanho da imagem de aviso (+0x266C, da string em +0x265C) != 0 |
| `Board::MarkSpaceImage(Point, string, GL_Color)` 0x571800 | `Tutorial::AddTutorial("Tutorial_Environment", x, y)`, imagem em +0x265C e cor em +0x2674 |
| `Board::IsWall(Point, int dir, int perfil)` 0x5743B0 | parede em +0x2AC4+dir (dir 4 = qualquer); com parede, falso para os perfis 1, 3, 5 e 6 |
| `Board::SetWall(bool, Point, int)` 0x574450 | lida no bloco 4 |
| `Board::AddTeleport(Point, Point)` 0x56C390 | grupo 1 se a lista Board+0x7470 estava vazia, senão 2; os dois pontos entram na lista; `BoardSpace::Clear` nas duas casas; `SetTeleport(grupo)` embutida grava +0x2BA0, salvo em buraco ou água |
| `GameMap::TriggerSound(string)` 0x60BCA0 | só som |
| `VoiceEvent(string, int, int)` (registro 0x67AB90) | `id` +0x00, `pawn1` +0x18, `pawn2` +0x1C; o corpo do construtor não foi localizado, e o teste confirmou a cópia dos argumentos na ordem |

A lista de teleportes é Board+0x7470, a mesma que `Board::IsDangerous` (0x572610) consulta: casa de teleporte conta como
perigosa. O Rust chamava essa lista de `danger_list_b`; agora é `teleports`.

Efeito na fila de uso: de 14 stubs para nenhum. Nas 85 missões sobra 1 erro de script (`BaseStart: attempt to index
local 'choice'`), que não vem de stub.

