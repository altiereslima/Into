# Roadmap

Cada marco tem um critério verificável. ✅ = feito, ▶ = próximo.

| # | Marco | Critério de pronto | Estado |
|---|---|---|---|
| M0 | Inventário da API nativa | 100% dos registros luabind, fechado por RTTI; constantes com valor | ✅ Etapa 05/06 |
| M1 | Boot dos scripts originais | Todos os arquivos de `GetScripts()` presentes carregam sem erro no host Rust | ✅ Etapa 06 (137/137) |
| M2 | Tipos de valor completos | `Point`, `Rect2D`, `GL_Color`, `Button`, `Text`, `StyleInfo`, `Objective`, `SpaceDamage`, `SkillEffect`, listas | ▶ parcial (4/12) |
| M3 | Ciclo de vida do motor | Descobrir no EXE o que roda depois do boot (funções Lua chamadas pelo C++, ordem, argumentos) e reproduzir | |
| M4 | Missão em memória | Carregar um `.map` original, criar `Board`/`Game`, posicionar unidades via `PAWN_FACTORY` | |
| M5 | Armas e dano | `SkillEffect`/`SpaceDamage` aplicados ao tabuleiro (empurrão, fogo, fumaça, ácido, prédios) | |
| M6 | Turno inimigo | IA (scripts) + pathfinding nativo (`GetPath`, `GetReachable`...) idênticos | |
| M7 | Simulador sem gráficos | Uma missão completa jogada por script de teste, resultado comparável ao original | |
| M8 | Assets + render | Ler `resource.dat`, desenhar tabuleiro/unidades/UI (SDL2) | |
| M9 | Jogável | Menu, hangar, ilhas, missões, salvar/carregar | |

## Fila imediata (gerada pelo `itb-boot`)

Stubs mais chamados durante o boot: `StyleInfo()` (138), `Text()` (44), `Objective()`,
`RegionInfo()`, `JoyRumble()`, `PilotSkill()`. Depois de M2, o relatório de M3 passa a guiar.

## Paralelo: diferencial contra o original

Quando M4 existir, montar um mod Lua para o jogo original que imprime o resultado de chamadas
nativas (`Board:IsBlocked`, `GetPath`...) num tabuleiro conhecido, e comparar com o Rust.
