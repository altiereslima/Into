# Roadmap

Cada marco tem um critério verificável. ✅ = feito, ▶ = próximo.

| # | Marco | Critério de pronto | Estado |
|---|---|---|---|
| M0 | Inventário da API nativa | 100% dos registros luabind, fechado por RTTI; constantes com valor | ✅ Etapa 05/06 |
| M1 | Boot dos scripts originais | Todos os arquivos de `GetScripts()` presentes carregam sem erro no host Rust | ✅ Etapa 06 (137/137) |
| M2 | Tipos de valor completos | `Point`, `Rect2D`, `GL_Color`, `Button`, `Text`, `StyleInfo`, `Objective`, `SpaceDamage`, `SkillEffect`, listas | ▶ 17 classes prontas, incl. SkillEffect e listas (faltam Effect, Choice, Squad…) |
| M3 | Ciclo de vida do motor | Descobrir no EXE o que roda depois do boot (funções Lua chamadas pelo C++, ordem, argumentos) e reproduzir | ▶ inicialização reproduzida; 118 callbacks inventariados |
| M4 | Missão em memória | Carregar um `.map` original, criar `Board`/`Game`, posicionar unidades via `PAWN_FACTORY` | ▶ 376/376 mapas carregam, `PAWN_FACTORY` cria unidades e `startNewGame` + `BaseStart` terminam sem stub; preparação da missão (terreno aleatório, vida dos prédios, chuva) igual ao jogo; falta o resto de `BoardPlayer` (`SelectMap`, `InitializeMission`, posicionamento) |
| M5 | Armas e dano | `SkillEffect`/`SpaceDamage` aplicados ao tabuleiro (empurrão, fogo, fumaça, ácido, prédios) | ▶ 559/559 armas geram e **aplicam** seus efeitos, com fila e atrasos do motor; faltam tempo de voo, rede elétrica, itens |
| M6 | Turno inimigo | IA (scripts) + pathfinding nativo (`GetPath`, `GetReachable`...) idênticos | ▶ GetReachable/GetPath/GetSimpleReachable/GetSimplePath prontos |
| M7 | Simulador sem gráficos | Uma missão completa jogada por script de teste, resultado comparável ao original | |
| M8 | Assets + render + som | `itb-assets` extrai localmente PNG/fontes/sons (sem versionar nada do jogo); motor lê do `resource.dat`/`.bank` ou da pasta extraída; desenhar tabuleiro/unidades/UI (SDL2); som sem FMOD — ver [decisão 0001](decisoes/0001-recursos-graficos-e-som.md) | decidido; arquivos disponíveis na 1.2.93; ainda não começou |
| M9 | Jogável | Menu, hangar, ilhas, missões, salvar/carregar | |

## Fila imediata

O projeto já foi migrado para a 1.2.93 (ver `CHECKPOINT.md`).
Etapa 13: terminar M4 (resto de `BoardPlayer`: `SelectMap`, `InitializeMission`, posicionamento), estendendo o teste diferencial a cada regra nova; depois laço de quadro,
`BoardPlayer`, tempo de voo dos projéteis, efeito de chegada completo e turno inimigo (M6).
Antes disso, a fila vinha do M3:
ler, para cada callback de `re/evidencias/engine_callbacks.csv`, como o motor o chama, começando por
`startNewGame` → criação de ilhas/missões → `CallMethod("MissionN", "BaseStart")` e as armas
(`GetTargetArea`/`GetSkillEffect`), que exigem `SkillEffect`, `PointList` e `Board`.

## Paralelo: diferencial contra o original

Pronto para o que não depende de turnos: ver `ferramentas/diferencial/LEIA-ME.md`. Falta estender à aplicação de
efeitos, que no jogo real acontece ao longo dos quadros, e às regras novas à medida que forem reconstruídas.
