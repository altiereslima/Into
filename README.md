# Into — motor do Into the Breach em Rust

Reimplementação do motor do **Into the Breach** (Subset Games) em Rust, que executa os **scripts Lua
originais** da sua própria cópia do jogo. Projeto de fã, sem afiliação com a Subset Games.

> Este repositório **não contém nenhum arquivo do jogo** (EXE, DLLs, scripts, mapas, imagens, sons).
> Você precisa de uma cópia legítima do Into the Breach (testado com a v1.2.88).

## Estado (Etapa 12)
- Os scripts originais **carregam sem erro** no host Rust: 137/137 arquivos presentes (`itb-boot`).
- API nativa mapeada por completo a partir do EXE: 32 classes, 343 métodos, 39 funções globais,
  111 propriedades, 58 construtores, 190 constantes. 475 nomes expostos à Lua: 212 reconstruídos,
  13 respostas de plataforma, 250 ainda stubs. O boot só toca 1 stub.
- **As 559 armas originais rodam sem erro nem stub, e seus efeitos já são aplicados ao tabuleiro** (dano, empurrão, colisão, fogo, fumaça, ácido, gelo, terreno), na ordem e com os atrasos do motor original (`itb-weapon <jogo> --todas`). Uma arma: `cargo run -p itb-lua --bin itb-weapon -- "/caminho/do/jogo" Prime_Lasermech 3 4 3 1`.
- Inicialização do Lua igual à do motor original, e 118 funções Lua que o motor chama já inventariadas.
- Núcleo com regras reconstruídas e testadas: ocupação do tabuleiro, bloqueio de caminho, alcance (Dijkstra) e caminho (A*) idênticos aos do EXE, terreno, dano letal,
  `Point`/`Rect2D`/`GL_Color`/`Button`/`Text`/`SpaceDamage`/`Objective`…, `rand()` idêntico ao do jogo, escalas de tela.

Decisões registradas em [`docs/decisoes/`](docs/decisoes/). Detalhes em [`CHECKPOINT.md`](CHECKPOINT.md), estratégia em [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md),
próximos passos em [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Estrutura
| Pasta | Conteúdo |
|---|---|
| `crates/itb-core` | regras do jogo em Rust puro (sem IO) |
| `crates/itb-lua` | host Lua 5.1 + `itb-boot` |
| `re/tools` | ferramentas de engenharia reversa (Python, só leitura do EXE) |
| `re/evidencias` | inventário da API, constantes, desmontagens de prova |
| `docs` | arquitetura, roadmap, divergências, relatórios, histórico das etapas |

## Usar
```bash
cargo test --workspace                                   # não precisa do jogo
cargo run -p itb-lua --bin itb-boot -- "/caminho/Into the Breach"
ITB_GAME_DIR="/caminho/Into the Breach" cargo test -p itb-lua -- --ignored   # boot real como teste
```
No Windows: `TESTAR_WINDOWS.bat` e `RODAR_BOOT_WINDOWS.bat`.

Nomes das funções vindos da versão Android: `re/evidencias/nomes_android_resumo.md` (gerado com o `libmain.so` do usuário por `re/tools/match_android_symbols.py`).

Ferramentas de análise: `pip install -r re/tools/requirements.txt`, depois
`python re/tools/map_lua_api_v2.py Breach.exe re/evidencias/`,
`python re/tools/check_inventory_v2.py Breach.exe Into.the.Breach.zip` e
`python re/tools/gen_api_manifest.py`.
