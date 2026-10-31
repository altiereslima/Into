# CHECKPOINT — Etapa 09: alcance, caminho simples e 92% das armas rodando

**Data:** 2026-10-31. Etapas anteriores em `docs/historico/`.

## Feito
- **Decompilador** (angr) adotado para as rotinas grandes; cada uma conferida no assembly.
- **Passagem e alcance** (`itb-core/src/pathing.rs`): `IsPassable` por perfil de movimento, `IsFree`,
  `GetSimpleReachable` (área de mira padrão das armas) e `GetSimplePath` (caminho em L), além de
  `Pawn:GetPathProf`.
- **SkillEffect** agora com 29 métodos: + AddBounce, AddBoardShake, AddBurst, AddCharge, AddQueuedCharge,
  AddBurrow, AddLeap, AddTeleport, AddGrapple; global `SoundEffect`.
- **Pawn**: IsFire, IsFrozen, IsAcid, IsInfected, IsPowered, IsGuarding.
- **Varredura de todas as armas** (`itb-weapon <jogo> --todas`): executa `GetTargetArea` e `GetSkillEffect`
  de cada uma das 559 armas dos scripts e mostra o que ainda bloqueia.

## Números
| Métrica | Etapa 08 | Etapa 09 |
|---|---|---|
| Armas que rodam sem erro e sem tocar stub (559) | — | **514 (92%)** |
| Nomes da API reconstruídos / plataforma / stubs (475) | 165 / 11 / 299 | **184 / 11 / 280** |
| Entrypoints nativos com Rust testado (368) | 96 (26,09%) | **115 (31,25%)** — 76 com lógica, 39 acessores |
| Testes | 62 | **69** + 2 de integração |

## Próximo
`GetReachable`/`GetPath` (Dijkstra do EXE), `Board:IsDeadly`, `IsTerrain`, `IsPod`, e então **aplicar** um
`SkillEffect` ao tabuleiro (M5). Lista viva de bloqueios: `docs/relatorios/varredura_armas_etapa09.txt`.
