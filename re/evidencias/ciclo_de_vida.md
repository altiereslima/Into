# Etapa 07 — ciclo de vida do Lua no motor (marco M3, parte 1)

## Inicialização (0x0044CAC0)
1. `luaL_newstate`, `luaL_openlibs`, `luabind::open` (0x006E3080).
2. Registro dos tipos do motor (0x0044AC60: Point, Rect2D, GL_Color, Text, Button, desenho, aleatório…).
3. `ActiveLua` vftable[0] = 0x00676020: registro das classes do jogo (Board, Pawn, SkillEffect…).
4. 0x0044C8B0 → vftable[2] = 0x00675D00: executa `scripts/scripts.lua` e lê `GetScripts()`.
5. 0x0044D1A0: imprime `loading %d scripts`, executa cada arquivo (ausente → `%s doesn't exist!`) e
   chama vftable[1] = 0x00675C70: `SetPawn(nil)`, `SetBoard(nil)`, `SetGame(nil)`.

Recarga (vftable[3] = 0x00675E10): guarda `GetPawn()/GetBoard()/GetGame()`, refaz a inicialização e
devolve os objetos com `SetPawn/SetBoard/SetGame` (0x00675F80).

O host Rust reproduz os passos 1–5 (`Host::new` + `Host::boot`, teste `boot_clears_engine_globals_like_activelua`).

## API reversa: funções Lua que o motor chama
`re/tools/map_engine_callbacks.py` cruza os nomes definidos nos scripts com strings do EXE referenciadas
fora dos registradores do luabind. Resultado em `engine_callbacks.csv`: **118 callbacks candidatos**
(78 funções globais, 40 métodos) em 183 referências. Exemplos:

| Grupo | Callbacks |
|---|---|
| Estado global | `SetBoard`, `SetGame`, `SetPawn`, `GetBoard`, `GetGame`, `GetPawn`, `GetScripts` |
| Campanha | `startNewGame`, `LoadGame`, `SaveGame`, `getStartingSquad`, `createIncidents`, `createFinalMission`, `addCustomMission` |
| Missão (via `CallMethod("MissionN", ...)`) | `BaseStart`, `BaseDeployment`, `BaseNextTurn`, `BaseUpdate`, `BaseObjectives`, `BaseCompletedObjectives`, `MissionEnd`, `UpdateSpawning`, `PlanEnvironment`, `ApplyEnvironmentEffect`, `IsEndBlocked`, `GetMap`… |
| Armas (via `CallMethod("Skill_X", ...)`) | `GetTargetArea`, `GetSkillEffect`, `GetTargetScore`, `GetSecondTargetArea`, `GetFinalEffect_Helper`, `TranslateFirstClick`, `IsTwoClickException`, `GetCost`, `GetUpgradeCost`, `GetTipData`… |
| Texto/UI | `GetText`, `GetDialogText`, `GetPilotDialog`, `GetTileTooltip`, `GetStatusTooltip`, `GetColor*`, `GetCharSet*`… |
| Recompensas | `getWeaponDrop`, `getPilotDrop`, `getPodList`, `getCityPower`, `getBuildingHealth`, `getRainChance`… |

`CallMethod(obj, func, a1, a2)` (global.lua) é o despachante usado pelo motor para métodos de missões e armas.

Pendente (M3, parte 2): para cada callback, ler no código C++ a ordem das chamadas, os argumentos e o uso do retorno.
