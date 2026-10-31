# CHECKPOINT — Etapa 12: executor de SkillEffect com relógio

**Data:** 2026-10-31. Etapas anteriores em `docs/historico/`.

## Feito
- **Fila de efeitos do tabuleiro** (`itb-core/src/executor.rs`), lida no EXE: `Board:AddEffect` enfileira com
  cronômetro −1; a atualização do tabuleiro espera atrasos positivos, `FULL_DELAY` (−1, espera tudo parar) e
  `PROJ_DELAY` (−2, espera só os projéteis) e inicia uma lista por vez.
- **Executor** (0x005601C0): corpo a corpo guardado na unidade e aplicado 0,15 s depois; dano direto na hora;
  projéteis/artilharia aplicam o dano ao chegar; movimento depois do dano; uma entrada com `fDelay` manda o
  resto da lista para o fim da fila com aquele cronômetro.
- **Empurrões com o relógio do EXE**: 0,4 s até resolver, reiniciados por um novo empurrão.
- `Board:AddEffect` na Lua; `itb-weapon` mostra a **linha do tempo** de cada arma.
- Provas: `re/evidencias/rotinas_executor.md`.

## Exemplo (`itb-weapon <jogo> Prime_Punchmech 5 3 5 4`)
```
0.00s corpo a corpo de 0 armado
0.00s pausa -1 (0 entradas restantes)
0.15s golpe corpo a corpo de 0 em (5,4)
0.55s empurrão de 1 resolvido
```

## Ainda suposto
Tempo de voo de projéteis (0,25 s) e de artilharia/ataque aéreo (0,5 s), e animações de movimento (a unidade
chega na hora). Estão em `Timing` e em `docs/DIVERGENCIAS.md`.

## Números
| Métrica | Etapa 11 | Etapa 12 |
|---|---|---|
| Armas aplicadas ao tabuleiro sem falha (559) | 559 | **559**, agora com a ordem e os atrasos do motor |
| Nomes da API reconstruídos / plataforma / stubs (475) | 211 / 13 / 251 | **212 / 13 / 250** |
| Testes | 94 | **99** + 2 de integração |

## Próximo
Ler a atualização dos projéteis (tempo de voo) e das animações de movimento; efeito de chegada completo
(0x0059C5C0); depois o turno inimigo (M6): preparar ataques (`q_effect`), mover e atacar.
