# CHECKPOINT — Etapa 13 (em andamento): carga de mapas, sobre a base 1.2.93

**Data:** 2026-11-01. Etapas anteriores em `docs/historico/`.

**Para continuar o trabalho, leia `docs/CONTINUAR.md`.**

## Etapa 13: mapas de missão
Regras antes de gráficos, por escolha do usuário: o objetivo é chegar ao teste diferencial contra o jogo original.
- **Os `.map` são tabelas Lua.** `Host::load_map` executa o arquivo, e `OccupancyBoard::from_map` aplica as casas na ordem de `Board::LoadScript`. Provas em `re/evidencias/v1.2.93/rotinas_mapa.md`.
- **`Board:GetZone`** reconstruído, com a zona padrão `"enemy"` que o motor calcula quando o mapa não a declara.
- **`itb-map`** desenha um mapa (`itb-map <jogo> acid0`) ou resume todos (`--todos`).
- Achados: `health_min` é a vida atual; `team` não é lida na carga; casa não listada nasce com vida 4/4, e casa listada sem `health_max`, 2/2.
- **`PAWN_FACTORY:CreatePawn`** cria unidades das tabelas de `pawns.lua`. Toda unidade de `PawnList` nasce do seu tipo. Provas em `re/evidencias/v1.2.93/rotinas_unidades.md`.
- **`Board:AddPawn`** com casa escolhida, incluindo a reordenação da lista em três passadas. A arena do Rust mantém índices estáveis, e a ordem do motor fica em `pawn_list`.
- **`Board:SpawnPawn` e a fila de surgimento:** casa disponível, sorteio com `rand()` e entrada na fila, como em `Board::GetAvailableSpace` e `Board::SpawnQueued`; bloqueio de surgimento causa dano.
- **`Game` real:** `GetSector`, `GetTeamTurn`, `GetTurnCount`, `IsIslandUnlocked` e `GetPower`; os outros métodos seguem como stub. Também `Board:GetPawns`, `SetTerrain`, `BlockSpawn` e `ClearBlockSpawns`.
- **`startNewGame` e `Mission:BaseStart` originais terminam sem tocar nenhum stub:** `itb-map <jogo> any3 --missao Mission_Acid` mostra os inimigos na fila.
- **Fila por uso, bloco 5 (fila zerada):** `SetShield`, `SetPowered`, `SetAcid`, `SetActive`, `SetMissionCritical`, `IsAvoidingMines`, `AddShield`, `IsEnvironmentDanger`, `IsWall`, `SetWall`, `MarkSpaceImage`, `AddTeleport`, `TriggerSound` e a classe `VoiceEvent`. A lista Board+0x7470 era a de teleportes (`teleports`). Conferido contra o jogo: `Mission_Dam`/`opendam19`, 0 divergências em 67.813 linhas. As 85 missões rodam no Rust sem tocar stub.
- **Fila por uso, bloco 4:** `RemovePawn` (2 formas), `ClearSpace`, `SetItem` (com o dano do item por nome), `SetTerrainIcon`, `GetDeployLocScore` e as formas com zona de `AddPawn`/`SpawnPawn`. `item_0x2b04` virou `item` (nome). Os 3 `rand()` de criar unidade foram localizados (piloto padrão). Conferido contra o jogo: `Mission_Mines`/`any25`, 0 divergências em 66.525 linhas. Fila de uso: 14 stubs.
- **Fila por uso, bloco 3:** `IsSafe` (verdadeiro = segura), `StopWeather`, `GetBuildings`, `SetCustomTile`, `SetAcid`, `SetSmoke`, `SetCracked`, `Board:SetFrozen`, `RemoveShield` e `Pawn:SetFrozen`. Conferido contra o jogo: `Mission_Train`/`trainAE4`, 0 divergências em 67.433 linhas, com mudanças de casa. Fila de uso: 19 stubs.
- **Fila por uso, bloco 2:** `GetDistanceToBuilding`, `GetDistanceToPawn`, `IsDamaged`, `GetMechDamage`, `AddUniqueBuilding`, `MarkSpaceDesc`, `IsRanged`, `IsTempUnit`; `IsBusy` e `GetEventCount` parciais. `unique_0x27ac` virou `unique_building` (nome do prédio único). Conferido contra o jogo: `Mission_Tanks`/`any1`, 0 divergências em 66.666 linhas. Fila de uso: 26 stubs.
- **Fila por uso, bloco 1:** `IsPowered`, `IsFrozen`, `IsDangerousItem`, `GetEnemyCount`, `GetPawnCount` (time e tipo), `IsNonGridStructure`, `GetCustomPositionScore`, `Objective:Failed` e `GameMap:AddObjective`; `IsTargeted` parcial (sem fila de ataques). Conferido contra o jogo: `Mission_Volatile`/`mix6`, 0 divergências em 66.302 linhas. Provas em `re/evidencias/v1.2.93/rotinas_fila_uso.md`.
- **Preparação da missão** (`BoardPlayer::LoadMission` até `InitializeMission`): `Host::load_mission` carrega o mapa, dá vida aos prédios (`SetBuildingHP`, conta em `float` com `ceil`), sorteia floresta, areia, gelo, ácido, rachaduras e chuva (`RandomizeEnvironment`, `RandomizeTerrain`) com as funções Lua originais. Também `Board:GetHealth`, `RandomizeTerrain`, `SetWeather` e a lista de prédios. Provas em `re/evidencias/v1.2.93/rotinas_preparo.md`.

- **Teste diferencial pronto** (`ferramentas/diferencial/`): a mesma coleta roda no jogo original e no Rust.

| Métrica | Antes | Agora |
|---|---|---|
| Mapas originais carregados | 0 | **376 de 376**, sem chave ignorada |
| Unidades criadas pelos mapas | 0 | **6 de 6** |
| Nomes reconstruídos / plataforma / stubs (475) | 212 / 13 / 250 | **282 / 13 / 179** |
| Testes | 100 + 2 | **147 + 6** |
| Fatos conferidos na 1.2.93 | 36 | **56** |

## Teste diferencial contra o jogo original
Duas partidas novas, agora com a preparação da missão feita pelo Rust (o terreno real não é mais copiado):
`Mission_Airstrike` no mapa `any30` e `Mission_Tides` no `hightide18`. Ambas dão **0 divergências** (65.790 e 64.890
linhas iguais), com terreno e vida das 64 casas iguais aos do jogo e o `rand()` em sincronia em 6 dos 7 pontos da
preparação (o primeiro depende de `SelectMap`). Resultado da segunda em `docs/relatorios/diferencial_1293.txt`.

| Seção (segunda partida) | Linhas iguais |
|---|---|
| Armas: área e efeito das 559 | 57.144 (varia com o tabuleiro: 58.044 na primeira) |
| Unidades: 24 consultas em 145 tipos | 3.481 |
| Casas, com e sem unidades, agora com a vida | 3.202 |
| Preparação: chamadas às funções Lua, com argumentos e retorno | 9 |
| `SpaceDamage` | 260 |
| Globais | 208 |
| Caminho e alcance | 325 |
| `CreatePawn`: consumo de `rand()` | 145 |
| Sorteio de casas, com `rand()` sincronizado | 6 |
| Outras | 110 |

O que o teste corrigiu: `CreatePawn` gasta 3 `rand()`; a energia inicial é 5/7; `IsNew*` são opções da partida. O que revelou: o motor troca chão por floresta antes de `BaseStart`, agora reconstruído e conferido.

**Limites:** um único caminho de cada arma, de uma casa para outra. Só setor de grama na dificuldade Normal. Não cobre a aplicação de efeitos, turnos, IA, objetivos, a escolha do mapa nem a chuva (sem método de leitura).

**M4 ainda não está pronto:** falta o resto de `BoardPlayer` (`SelectMap`, `InitializeMission`, `Start`, `StartState`, posicionamento dos mechs, turno).

**Próximo:** ver `docs/CONTINUAR.md`.

# Migração para o Breach.exe 1.2.93

Método e limites em `re/evidencias/v1.2.93/LEIA-ME.md`.

## Feito
- **Base do projeto agora é a 1.2.93** (SHA256 `0f5bdc90…b910`, instalação com `resources/`).
- **Inventário da API** gerado na 1.2.93: `map_lua_api_v2.py` aceita as duas versões (`PROFILES`). O manifesto só mudou nos endereços.
- **Tabela de endereços 1.2.88 → 1.2.93** (`migrar_enderecos.py`), sem o EXE antigo: inventário, Android e bytes das provas, conferidos entre si.
- **Nomes do Android** casados direto com a 1.2.93, com o mesmo `libmain.so` de antes.
- **Conferência repetível:** `conferir_1293.py` lê 36 fatos no EXE novo (tempos, tabelas, constantes, slots de vftable).
- **Erro corrigido em `Pawn::Push`:** `DIR_FLIP` depende de `IgnoreFlip` (+0x91F), não de `Pushable` (+0x91E). Vale na 1.2.93 e no Android 1.2.92.
- **Erro corrigido nos nomes:** `ActiveLua::GetScripts` é o slot 2; o slot 1 só faz `SetPawn/SetBoard/SetGame(nil)`.

## O que mudou entre as versões
| Item | Resultado |
|---|---|
| Registros luabind (551), ordem e assinaturas RTTI | idênticos |
| Classes Lua (32) | idênticas |
| Valores globais (197) | idênticos |
| Provas da Etapa 05 com bytes (24) | as 24 achadas, iguais fora os campos de endereço |
| Pontos em que o deslocamento muda | 52, somando 3669 bytes em cerca de 4 MB |
| Maior mudança | carregamento de arquivos e fontes (+3216 bytes); depois salvamento (+400) |

## Números
| Métrica | Antes | Agora |
|---|---|---|
| Versão de referência | 1.2.88 | **1.2.93** |
| Endereços citados no projeto com par na 1.2.93 (342) | — | **342** (312 início de função ou prova direta; 30 pontos internos interpolados) |
| Boot: arquivos ok / ausentes / erros | 137 / 1 / 0 | **137 / 1 / 0** |
| Armas sem erro nem stub (559) | 559 | **559** |
| Nomes reconstruídos / plataforma / stubs (475) | 212 / 13 / 250 | **212 / 13 / 250** |
| Testes | 99 + 2 | **100 + 2** |

## Ainda não provado
- Sem o EXE 1.2.88, deslocamento igual prova tamanho igual, não bytes iguais. Uma mudança do mesmo tamanho passaria despercebida.
- Das 52 regiões, só foram lidas as que contêm rotinas citadas nas regras (`Pawn::Push`, `Pawn::GetType`, `ActiveLua`, `GetPawnCount`). Em `Pawn::Push`, o código muda depois de armar o cronômetro de 0,4 s; o motivo não foi identificado, porque não há os bytes antigos, mas a versão atual segue igual à do Android.
- Provas em `re/evidencias/rotinas_*.md` e endereços nos comentários do Rust continuam na 1.2.88; a tradução está em `v1.2.93/migracao_enderecos.csv`.
- Nomes provisórios no código (`Tile::Push`, `byte_0x91e`…) ainda não foram trocados pelos reais.

Gráficos e som (decisão 0001) já têm os arquivos necessários e vêm depois das regras.
