# CHECKPOINT — Etapa 10: alcance e caminho do EXE, dano letal; 559/559 armas rodando

**Data:** 2026-10-31. Etapas anteriores em `docs/historico/`.

## Feito
- **Alcance e caminho** (`itb-core/src/pathfinding.rs`): `Board:GetReachable` (Dijkstra com `std::set`
  ordenado por custo, x, y) e `Board:GetPath` (A* com heap binário do MSVC, heurística ×1.01 em float).
  Desempates reproduzidos instrução a instrução, inclusive o custo 1000 de prédio para projéteis e as
  paredes por direção (+0x2AC4).
- **Terreno** (`board_terrain.rs`): `IsTerrain` (com os valores especiais 14, 16, 18), `IsCrackable`,
  `IsPod`, `IsItem`, `IsDangerous`, `IsSpawning`.
- **Dano letal** (`damage.rs`): `Board:IsDeadly` e `Pawn::WouldDie` — impulso do atacante, escudo, gelo,
  `DAMAGE_ZERO`/`DAMAGE_DEATH`, blindagem, ácido, chão rachado e gelo danificado.
- **Pawn**: `IsShield`, `IsBoosted`, `IsFlying`, `IsJumper`, `IsTeleporter`, `IsBurrower`, `IsGrappled`,
  `GetMoveSpeed`, `GetType`, `IsCorpse` na Lua; `GetBaseMove`/`IsAbility` como dados do host.
  Correção: `GetPathProf` usa `IsJumper()`/`IsFlying()`, que exigem unidade viva.
- **SkillEffect**: + AddAirstrike, AddReverseAirstrike, AddDropper, AddEmitter, AddVoice, AddQueuedVoice.
  Global `IsPassiveSkill`.
- Provas: `re/evidencias/rotinas_alcance.md`.

## Números
| Métrica | Etapa 09 | Etapa 10 |
|---|---|---|
| Armas que rodam sem erro e sem tocar stub (559) | 514 (92%) | **559 (100%)** |
| Nomes da API reconstruídos / plataforma / stubs (475) | 184 / 11 / 280 | **210 / 13 / 252** |
| Entrypoints nativos com Rust testado (368) | 115 (31,25%) | **131 (35,60%)** — 91 com lógica, 40 acessores; +10 parciais |
| Testes | 69 | **83** + 2 de integração |

## Próximo
**Aplicar** um `SkillEffect` ao tabuleiro (M5): dano, empurrão, fogo, fumaça, ácido, gelo, rachaduras,
prédios — começando pela rotina que consome a `DamageList` em ordem com os atrasos. Depois, os scripts de IA
inimiga (M6) já têm `GetReachable`/`GetPath`.
