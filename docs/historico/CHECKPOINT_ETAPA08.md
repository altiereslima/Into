# CHECKPOINT — Etapa 08: armas originais rodando no motor Rust

**Data:** 2026-10-30. Etapas anteriores em `docs/historico/`.

## Feito
- **`SkillEffect`** com os construtores de efeito mais usados pelas armas (dano, atraso, projétil, laser,
  artilharia, corpo a corpo, movimento, som, script, animação) e **`DamageList`/`PointList`/`IntList`/`UIntList`**.
  Semântica de **referência** do luabind preservada (`ret.effect:back().bHide = true` funciona como no jogo).
- **`SpaceDamage`**: métodos `IsMovement`, `MoveStart` (inclusive o (−1,1) do caminho vazio), `MoveEnd`.
- **`Board` e `Pawn` na Lua**, ligados ao núcleo (`OccupancyBoard`, agora unificado com terreno/fogo/fumaça/ácido…):
  17 métodos de Board e 11 de Pawn; novos `IsPawnTeam`/`Pawn::IsTeam` e `GetSize`.
- **`itb-weapon`**: executa o `GetTargetArea` e o `GetSkillEffect` ORIGINAIS de qualquer arma num tabuleiro Rust.
  Exemplos em `docs/relatorios/armas_etapa08.txt` (soco, tanque, artilharia, laser, puxão).

## Números
| Métrica | Etapa 07 | Etapa 08 |
|---|---|---|
| Nomes da API reconstruídos / plataforma / stubs (475) | 84 / 11 / 380 | **165 / 11 / 299** |
| Entrypoints nativos com Rust testado (368) | 47 (12,77%) | **96 (26,09%)** — 63 com lógica, 33 acessores triviais |
| Testes | 57 | **62** + 2 de integração com o jogo |

## Próximo
`Board:GetSimpleReachable` / `GetReachable` / `GetPath` (alcance e caminho — usados pela área de alvo padrão
e pelo movimento), depois os efeitos restantes (`AddBounce`, `AddLeap`, `AddCharge`…) e a **aplicação** de um
`SkillEffect` ao tabuleiro (dano, empurrão, fogo) — marco M5.
