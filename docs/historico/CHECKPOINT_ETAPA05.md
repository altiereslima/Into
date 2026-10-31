# CHECKPOINT — Etapa 05: inventário luabind fechado + ocupação do tabuleiro em Rust

**Data:** 2026-10-30. **Estado:** inventário da API Lua nativa fechado por RTTI; 20 rotinas novas reconstruídas
e testadas; `cargo test` executado pela primeira vez (42 testes, 0 falhas, 0 avisos).

## Fontes e continuidade
- Original: `Into.the.Breach.v1.2.88.zip` (NÃO incluído). `Breach.exe` PE32, SHA256
  `4bd9e4cc43140244d7def56d440346be836efc5b072dd3e8b0dda3ff31fc84b0`.
- Código das Etapas 01R–04 preservado. Única mudança em arquivo antigo: comentários em `src/board_queries.rs`
  (constante 2 confirmada como `TEAM_NONE`; `get_pawn_team` antigo marcado como superado) e `src/lib.rs` (novo módulo).
- O catálogo de 5 padrões da Etapa 04 continua reproduzível (`tools/check_inventory.py` PASS) e é
  **subconjunto exato** do novo inventário (verificado em `tools/check_inventory_v2.py`).

## Meta permanente
Encontrar 100% das funções e bindings expostos à Lua no EXE de referência e reconstruir 100% dos comportamentos
nativos relevantes ao motor Rust, com testes.

## O que mudou no método
A Etapa 04 achava registros por 5 sequências de bytes. Agora:
1. Cada objeto de registro luabind tem **RTTI do MSVC** (`memfun_registration<Classe, assinatura>`,
   `property_registration`, `constructor_registration`, `function_registration`). A vftable prova a família e
   o demangling (`llvm-undname`) dá a **assinatura C++ completa** de cada método.
2. Uma emulação simples de pilha x86 nas funções registradoras recupera nome, ponteiro de membro, ajuste de
   `this` e offset de propriedade — inclusive na família com **nome embutido** no registrador (109 métodos que
   os padrões antigos não viam: `GetPawn`, `IsPawnAlive`, `GetSimplePath`, `CreatePawn`, `GetZone`...).
3. A classe Lua de cada registro vem da ordem `class_<T>("Nome")` (32 classes).
4. Thunks `vcall` são resolvidos pela vftable RTTI do subobjeto indicado pelo ajuste de `this`.

## Fechamento do inventário (provas automáticas)
- **300/300** tipos RTTI de registro luabind presentes no EXE são usados pelo inventário.
- **358/358** escritas dessas vftables no `.text` ficam dentro de registradores conhecidos.
- Toda chamada a registrador vira uma linha; 0 wrappers globais perdidos; 0 thunks sem resolução.
- **110/110** pares `Board`/`Game`/`PAWN_FACTORY` usados nos scripts casam com a classe certa
  (`Game`→`GameMap`, `PAWN_FACTORY`→`PawnFactory`). Os 32 "sem binding" da Etapa 04 estão todos resolvidos.
- Todas as funções globais nativas chamadas pelos scripts estão no inventário.
- Os 3 candidatos `desc`/`money`/`type` eram **propriedades** (`PilotSkill.desc`, `Cost.money`, `PilotInfo.type`;
  `money` também em `Effect`).
- Chamadas diretas `lua_pushcclosure` (15) apontam todas para o runtime do luabind (`0x006E2xxx–0x006E8xxx`);
  nenhuma registra função do jogo.

## Inventário (Etapa 05)
| Família | Registros |
|---|---|
| Classes Lua | 32 |
| Métodos (`memfun_registration`) | 343 |
| Funções globais (`function_registration`) | 39 |
| Propriedades (`property_registration`) | 111 |
| Construtores (`constructor_registration`) | 58 |
| Constantes inteiras em `_G` (`lua_pushinteger`) | 92 (+101 outros usos de `globals[...]`: objetos/leituras) |

## Progresso medido (funções chamáveis = métodos + globais)
- **382 registros chamáveis → 368 entrypoints nativos distintos** (após resolver thunks e sobrecargas que
  compartilham o mesmo corpo, ex.: `GetTerrain(Point)` e `GetTerrain(int,int)` → `0x00503B60`).
- **29/368 entrypoints com semântica Rust testada = 7,88%**; **339/368 restantes = 92,12%** (inclui 1 parcial).
  - Destes 29: **18 com lógica** (9 de etapas anteriores + 9 novos de ocupação) e **11 acessores triviais** de `Pawn`.
  - Parcial: `Pawn::IsCorpse` (`0x00629540`), depende de `0x00639E10(12)`.
- Por registros chamáveis: 32/382 testados (8,38%).
- **Por que o percentual da Etapa 04 (3,98%) não é comparável:** o denominador subiu de 226 para 368 porque o
  inventário antigo era incompleto. Os 9 alvos antigos agora estão de fato testados com Cargo.
- Ao contrário da Etapa 04, o denominador **está fechado para a API luabind** (provas acima). Pode crescer
  apenas se surgir mecanismo fora do luabind — nenhum foi encontrado.

## Arquivos novos
- `tools/map_lua_api_v2.py` — inventário completo; gera `evidencias/lua_api_v2_all.csv` (551 linhas, com
  assinatura C++), `lua_api_v2_classes.csv`, `lua_global_values.csv`, `lua_api_v2_progress.json`.
- `tools/check_inventory_v2.py` — QA com as provas de fechamento e de não regressão (Etapas 03 e 04).
- `tools/compare_script_calls_v2.py` — cruzamento com scripts validando a classe.
- `tools/dump_disasm_etapa05.py` + `evidencias/etapa05_disasm/` — 24 desmontagens de prova.
- `tools/rust_status.json` — quais alvos têm Rust testado (lido pelo inventário).
- `src/board_occupancy.rs`, `tests/board_occupancy.rs`, `evidencias/rotinas_ocupacao.md`.
- `tools/requirements.txt` (capstone).

## Verificação feita nesta etapa
- `cargo test`: 42 testes, 0 falhas, 0 avisos (Rust 1.97). Backend SDL2 (`--features sdl2-backend`) **não** testado aqui.
- `check_inventory_v2.py` PASS e `check_inventory.py` (Etapa 04) PASS.
- `compare_script_calls_v2.py`: 110/110.
- Não houve execução diferencial contra o jogo original (só análise estática + testes do modelo).

## Regra para cada etapa daqui em diante
Rodar `map_lua_api_v2.py` + `check_inventory_v2.py` + `cargo test` e citar
**N entrypoints / M testados (lógica vs acessores) / K parciais / % restante**. Atualizar
`tools/rust_status.json` (ou sucessor) ao reconstruir algo.
