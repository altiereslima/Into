# Criação de unidades: leitura na 1.2.93

Todos os endereços são da 1.2.93. Código: `itb-core/src/pawn_factory.rs` e `itb-lua/src/pawns.rs`.

## `PawnFactory`
`CreatePawn(string)` 0x6425E0 chama `CreatePawn(string, 2)` 0x6423F0.

`CreateEmptyPawn` 0x642670:
1. Se a global do tipo não existe (0x44D330), registra erro e usa `"PunchMech"`.
2. Time 2 significa "use o padrão": chama `GetDefaultTeam`. Outro valor é usado como veio.
3. `Pawn::Pawn` 0x627F30, que chama `Pawn::OnInit` 0x629700.
4. Adiciona a arma `"Move"`.

`CreatePawn` depois lê `GetSkillList` e adiciona cada arma.

## Propriedades
0x4492B0 (inteiro), 0x4493D0 (booleano) e 0x4491D0 (texto) montam `"Get" + nome` e chamam esse método na tabela do tipo.
Os getters vêm de `CreateClass` em `global.lua`; os valores padrão, da tabela `Pawn`.

| Propriedade | Campo | Modelado em |
|---|---|---|
| `Corpse` | +0xF80 | `flag_0xf80` |
| `Flying` | +0x1314 | `movement.flying` |
| `Teleporter` | +0x1315 | `movement.teleporter` |
| `Jumper` | +0x131C | `movement.jumper` |
| `Health` | +0x988 e `Health(n)`: atual +0x8A8, máximo +0x8AC | `health`, `max_health` |
| `MoveSpeed` | +0x990 | `base_move_0x63a8f0` |
| `Pushable` | +0x91E | `byte_0x91e` |
| `IgnoreFlip` | +0x91F, invertido | `ignore_flip` |
| `Neutral` | +0x97C | `neutral` |
| `Massive` | +0x99D | `byte_0x99d`, `movement.massive` |
| `DefaultFaction` | +0x10BC | `field_0x10bc` |

Lidas sob demanda, fora de `OnInit`: `Burrows` (`IsBurrower` 0x637470), `NonGrid` (`IsNonGridStructure` 0x63B750).
`Armor` e `IgnoreFire` entram em `IsArmor` 0x63CFA0 e `IsFireImmune` 0x639A70, junto com outras condições.

Lidas em `OnInit`, mas ainda sem uso no Rust: `Leader`, `ImageOffset`, `Image`, `Minor`, `SpaceColor`, `Class`, `GetExtraSpaces`,
`SoundLocation`, `ImpactMaterial`, `Corporate` (piloto), `Tooltip`.

## Estado inicial (`Pawn::Pawn`)
Time vem do parâmetro; +0x978 = 4 (sem empurrão); +0x9E0 = 5; mech 0; +0xA64 = 0; +0x10C0 (`IsPowered`) = 1; +0x10E8 = 0.

## `rand()` ao criar
O teste diferencial mediu 3 `rand()` em cada `CreatePawn`, nos 145 tipos de `PawnList`, que tem 146 entradas porque um nome se repete. Um está em `Pawn::Pawn` (0x628D9A, logo depois de `OnInit`, grava em
+0x924). Os outros dois não foram localizados; `Pilot::OnInit` e 0x6402B0 também chamam `rand()`, mas o caminho até eles a partir da criação não foi seguido.

## Id
0x628DA7 incrementa o contador global 0x88FD64 e grava o valor em +0x9A4. No arquivo o contador vale 4, então a primeira
unidade criada recebe 5, se nada mais o tiver alterado. Outras rotinas gravam o id ao carregar jogo salvo.

## `Board::AddPawn(Pawn*, Point)` (0x56E830)
1. Liga a unidade ao tabuleiro; casa inválida usa `Board::GetAvailableSpace` 0x572A00.
2. `Pawn::SetSpace`.
3. Se +0x91C != 1 e `IsPowered`: fica 0 quando não é mech, tem só uma arma e `GetMoveSpeed` é 0; senão, 1.
4. Acrescenta à lista Board+0x3C e a refaz em três passadas, mantendo a ordem dentro de cada uma:

| Passada | Quem entra |
|---|---|
| 0 | time do jogador com facção 1 (`IsTeam(8)`), time do jogador não neutro, e `"Filler_Pawn"` do time do jogador |
| 1 | todos os outros times |
| 2 | time do jogador, neutro, que não seja `"Filler_Pawn"` nem bot |

Devolve a casa final. As seis variantes em `Board:AddPawn` devolvem `Point`.

`(Pawn*, zona)` e `(string, zona)` continuam stubs; as outras estão ligadas.

## Surgimento
`Board::SpawnPawn(Pawn*)` 0x5690C0 chama a variante com casa (-1,-1). `Board::SpawnPawn(Pawn*, Point)` 0x568CA0:

| Terreno da casa | Efeito |
|---|---|
| Buraco | `AddPawn` e `+0x9E0 = 1` |
| Água | `AddPawn` e `TriggerSplash`. Voador: `+0x9E0 = 1`. Quem não voa segue um caminho com `general_DiamondTarget` que não foi lido até o fim |
| Outro, time do jogador | `AddPawn`, `StartTurn(0)`, `SetActive(true)` |
| Outro, demais times | 0x5693A0 põe na fila e chama `SpawnAnimation` |

Casa inválida usa o terreno da sentinela, que é chão. As formas com nome devolvem o id (0x568F30, 0x569030).

Fila (0x5693A0): casa inválida chama `GetAvailableSpace`; depois acrescenta a unidade em Board+0x2D44 e a casa em Board+0x2D50 e
Board+0x2D5C. A unidade não entra na lista do tabuleiro nem em casa alguma, e por isso `GetPawn(id)` não a acha.

`Board::SpawnQueued` 0x569410 repete `UpdateSpawning(false, false)` 0x566470 até o cursor chegar ao fim, se Board+0x2CA8 for 0:
* Casa livre (`IsBlocked(p, 0)` falso): `SetActive(true)`, `AddPawn`, +0x27E8 = 0 e sai da fila.
* Casa bloqueada: `/ui/battle/emerge_resisted`, `Alert_Blocked`, conquistas; se quem ocupa não for `IsTeam(4)` com `Passive_Burrows`, a casa
  toma `SpaceDamage(1)`; o cursor avança, e a unidade fica na fila.

## Casa disponível
`Board::GetAvailableSpace(Pawn*)` 0x572A00:
1. Zona: `IsTeam(1)` usa `GetDropZone(false)`; os demais, a zona `"enemy"`.
2. Filtra com o teste 0x572E70, usando o time e `GetPathingProfile`.
3. Se vazia, Board+0x2CA8 for 0 e o time não for 1: filtra de novo com o time 9.
4. Sorteia `rand() % n`. Se a casa for floresta, `SetTerrain(chão)`.
5. Ainda vazia: registra erro e percorre o tabuleiro em x e depois y, mantendo só a coluna disponível mais à direita. Sorteia, sem mexer em floresta.

Teste 0x572E70 (time, perfil, x, y), com time 9 tratado como 6:
* Sempre recusa: item (+0x2B04), cápsula (+0x2AE4 == 1), `BlockSpawn` 1 ou 2, `IsDangerous`.
* Time 1: recusa `IsBlocked(p, perfil)`, ácido, +0x2AF0, fogo e buraco.
* Demais: recusa `IsBlocked(p, 0)`, gelo, água, ácido, +0x2AF0 e, salvo time 9, floresta.
* Por fim, recusa casas já em Board+0x2D50.
* Times 3 e -3 têm outro caminho, que usa `GetDropZone` e não foi lido.

`Board::GetDropZone(false)` 0x560A90: zona `"deployment"`, mantendo casas disponíveis (time 1, perfil 2) ou com mech. Sem zona: colunas 1 a 3,
sem as bordas de cima e de baixo. Com menos de 3 casas há um caminho (0x560E3A) que não foi lido.

## `Game`
| Rotina | Regra |
|---|---|
| `GetSector` 0x60D9A0 | `clamp(+0x5238 + 1, 1, 4)` |
| `GetTeamTurn` 0x60D960 | `BoardPlayer` em +0xC204: seu +0x158; sem ele, 2 |
| `GetTurnCount` 0x60D980 | `BoardPlayer`: seu +0x154; sem ele, 0 |
| `IsIslandUnlocked` 0x60A110 | `n <= perfil+0x30` |
| `GetPower` 0x60D8C0 | cópia do objeto de vida em +0xC03C (atual +0xC040, máximo +0xC044) |

`getCityPower` grava o máximo e prende a vida atual em 0..máximo (0x609BFE, 0x60CA65). O valor inicial da vida atual não foi achado; o host começa cheio.

## Outras rotinas de `Board`
`GetPawns` 0x566310 → 0x573D60: ids na ordem da lista; morto só entra se for cadáver ou mech. `SetTerrain` 0x565EE0 → 0x565E90: `BoardSpace::SetTerrain` e
0x573260, que não foi lida. `BlockSpawn` 0x56C0B0: `std::map` em Board+0x7458. `ClearBlockSpawns` 0x56C0E0: tipo 1 vira 0, e Board+0x7460 é esvaziada.

## Experimento: `Mission:BaseStart`
`itb-map <jogo> any3 --missao Mission_Acid` cria o `Game`, chama `startNewGame()` e `BaseStart()`. Os dois terminam sem tocar stub, e os inimigos vão para a fila.
Com o `rand()` do host na semente 1 e nesse caminho: `Firefly1@(6,4) Moth1@(7,4) Firefly1@(5,3)`. A sequência não é a do jogo real, que consome `rand()` em outras rotinas antes disso.

## Uso nos scripts das rotinas que faltam
| Rotina | Usos |
|---|---|
| `Game:AddObjective` | 55 |
| `Board:MarkSpaceDesc`, `Board:MarkSpaceImage` | 16, 15 |
| `Board:ClearSpace` | 11 |
| `Game:ClearTips`, `Game:GetPawn` | 11, 10 |

## Fluxo do motor
`BoardPlayer::SelectMap` 0x582A60 chama `GetMapTag`, `GetMapVetoes`, `GetMap` e `RandomMap`. `BoardPlayer::Start` 0x583480 chama
`StartState`, `SetupButtons`, `UpdateObjectives` e, entre outros, `BaseStart` (string referenciada em 0x5831E8).
Essas rotinas só foram lidas por strings e chamadas, não por inteiro.

O nome `Board::SetBuildingHP` existe no Android, mas não foi casado com o PC.
