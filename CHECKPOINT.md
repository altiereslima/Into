# CHECKPOINT — Etapa 07: tipos de dados do boot + ciclo de vida do Lua

**Data:** 2026-10-30. Etapas anteriores em `docs/historico/`.

## Feito nesta etapa
- **Classes de dados** num tipo genérico (`itb-lua/src/records.rs`), com valores padrão e construtores
  lidos do EXE: `Text` (TextData), `StyleInfo`, `JoyRumble`, `PilotSkill`, `TipData`, `Objective` (RepObj,
  6 construtores), `RegionInfo` e **`SpaceDamage`** (25 propriedades, 5 construtores; peça central do dano).
  Métodos ainda não implementados dessas classes continuam acessíveis como stubs que registram uso.
- **Inicialização igual à do motor** (`ActiveLua`, vftable 0x00836FC8): depois de carregar os scripts,
  o host chama `SetPawn(nil)`, `SetBoard(nil)`, `SetGame(nil)` e registra `loading N scripts` /
  `X doesn't exist!` como o original.
- **API reversa (M3):** `re/tools/map_engine_callbacks.py` inventariou **118 funções Lua que o motor
  chama** (78 globais, 40 métodos via `CallMethod`) — `re/evidencias/engine_callbacks.csv` e
  `re/evidencias/ciclo_de_vida.md`.

## Números
| Métrica | Etapa 06 | Etapa 07 |
|---|---|---|
| Scripts originais carregando sem erro | 137/137 | 137/137 |
| Nomes da API: reconstruídos / plataforma / stubs | 34 / 11 / 430 | **84 / 11 / 380** |
| Stubs chamados durante o boot | 10 tipos (330 chamadas) | **1** (`GetMechColorsInTexture`) |
| Entrypoints nativos com Rust testado | 47/368 (12,77%) | 47/368 (12,77%) — esta etapa foi de construtores/propriedades |
| Callbacks motor→Lua inventariados | — | 118 |
| Testes | 53 | **57** (+1 integração com o jogo) |

## Próximo passo
M3 parte 2: contrato de cada callback (ordem, argumentos, retorno), começando por `startNewGame` e pelo
ciclo de missão; em paralelo, `SkillEffect`/`PointList`/`DamageList` (M2) para as armas funcionarem.
