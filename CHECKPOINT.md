# CHECKPOINT — Etapa 11: aplicar armas ao tabuleiro (versão 1)

**Data:** 2026-10-31. Etapas anteriores em `docs/historico/`.

## Feito
- **Aplicação de dano** (`itb-core/src/apply.rs`), lida no assembly: `Board:DamageSpace`,
  `Tile::ApplyDamage` (a rotina de ~1500 instruções que o motor usa para cada `SpaceDamage`),
  `Pawn::ApplyDamage`, `Health::Add`. Escudo e gelo absorvem, `DAMAGE_ZERO`/`DAMAGE_DEATH`, blindagem e ácido,
  cura que limpa estados, prédios perdendo vida e virando entulho, montanha e gelo com 2 de vida, floresta
  que pega fogo, areia que vira fumaça, chão rachado que vira buraco.
- **Empurrão**: agendamento (`Pawn::Push`), colisão (prédio, montanha, unidade, parede) com 1 de dano para
  os dois lados, unidades "estáveis", e chegada na casa nova (afogar na água, cair no buraco).
- **Fogo, fumaça, ácido, gelo, rachadura e terreno** nas casas e nas unidades, com as interações do EXE
  (fumaça e ácido apagam fogo, gelo derrete com fogo, água vira gelo...).
- `Board:DamageSpace` na Lua; `itb-weapon` agora **aplica** os efeitos e mostra o que mudou.
- O que ainda não é simulado vira `BoardEvent` (rede elétrica, itens, cápsulas, criação de unidades, teias,
  retirada, scripts visuais). Provas: `re/evidencias/rotinas_dano.md`.
- Correção: o campo de vida da casa é +0x78 (atual) / +0x7C (máximo); `IsTerrain(p, 18)` usa o atual.

## Exemplo (`itb-weapon <jogo> Prime_Punchmech 5 3 5 4`)
O soco empurra o inimigo contra o prédio em (5,5): o inimigo toma 2 + 1 da colisão e morre, o prédio
toma 1 e vira entulho, e a rede elétrica registra 1 de dano.

## Números
| Métrica | Etapa 10 | Etapa 11 |
|---|---|---|
| Armas cujo efeito é aplicado ao tabuleiro sem falhar (559) | 0 | **559** (415 mudam o tabuleiro no cenário de teste) |
| Nomes da API reconstruídos / plataforma / stubs (475) | 210 / 13 / 252 | **211 / 13 / 251** |
| Entrypoints com Rust testado (368) | 131 (35,60%) | 131 + **12 parciais** (DamageSpace entrou como parcial) |
| Testes | 83 | **94** + 2 de integração |

## Próximo
Fechar as partes "versão 1": ler o executor do `SkillEffect` (ordem real, atrasos e quando os empurrões
resolvem), o efeito de chegada completo (0x0059C5C0), rede elétrica/prédios de energia e a chance de resistir,
itens e criação de unidades. Depois, o turno inimigo (M6).
