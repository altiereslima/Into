# Instruções para o Claude (Claude Code / Cowork)

Leia este arquivo inteiro antes de começar. Depois leia `CHECKPOINT.md` (estado atual), `docs/ROADMAP.md`
(marcos), `docs/DIVERGENCIAS.md` (o que ainda difere do original) e `docs/decisoes/`.

## Idioma e estilo
- Fale e escreva **em português** com o usuário (Altieres). Ele gosta de Python para ferramentas.
- Respostas curtas; relatórios de etapa com tabela de números (antes/depois) e "próximo passo".

## O projeto
Reimplementação do motor do **Into the Breach** em Rust que executa os **scripts Lua originais** da
instalação do usuário (modelo OpenMW/OpenRCT2). As regras são reconstruídas lendo o assembly do
`Breach.exe` (PC, v1.2.88 hoje; 1.2.92/93 a caminho, igual à versão Netflix).

- `crates/itb-core`: regras em Rust puro (tabuleiro, ocupação, pathing/pathfinding, dano, aplicação de
  efeitos, executor com relógio).
- `crates/itb-lua`: host Lua 5.1 (mlua) que expõe a API nativa aos scripts; binários `itb-boot` e `itb-weapon`.
- `re/tools`: ferramentas Python de engenharia reversa (só leitura dos binários).
- `re/evidencias`: inventário da API (`lua_api_v2_all.csv`), provas de cada rotina (`rotinas_*.md`),
  nomes vindos do Android (`nomes_android*.csv`, `nomes_android_resumo.md`).

## Regras invioláveis
1. **Nenhum arquivo do jogo no repositório**: EXE, DLLs, `scripts/`, `maps/`, `resources/`, `.bank`,
   `libmain.so`, PNGs/sons extraídos. Só código, nomes, endereços e documentação.
2. **Nunca gravar tokens/senhas** em arquivos, commits ou config do git. (Um token já foi exposto no chat
   e o usuário foi orientado a revogá-lo.)
3. Cada afirmação sobre o original precisa de prova: endereço + leitura do assembly (capstone; angr só
   como apoio, sempre conferido no asm). Suposições ficam marcadas como SUPOSIÇÃO e vão para
   `docs/DIVERGENCIAS.md`.
4. Fidelidade antes de beleza: reproduzir inclusive desempates, ordens de iteração, arredondamentos
   (`lua_tointeger` arredonda ao par; `rand()` do MSVC; heap do `std::priority_queue` do MSVC...).
5. Commits terminam com:
   ```
   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

## Como trabalhar
- Jogo do usuário: pasta da instalação (ex.: `C:\Program Files (x86)\Steam\steamapps\common\Into the Breach`).
- Testes: `cargo test --workspace` (sem o jogo) e `ITB_GAME_DIR=<jogo> cargo test -p itb-lua -- --ignored`.
- Boot e cobertura: `cargo run -p itb-lua --bin itb-boot -- <jogo> --cobertura re/evidencias/cobertura_api.tsv`.
- Armas: `cargo run -p itb-lua --bin itb-weapon -- <jogo> --todas` (varredura) ou `... <jogo> Prime_Punchmech 5 3 5 4`
  (uma arma, com linha do tempo e diferença do tabuleiro).
- Inventário: `python re/tools/map_lua_api_v2.py <Breach.exe> re/evidencias/` e
  `python re/tools/gen_api_manifest.py` (o CI confere que `api_manifest.rs` está atualizado).
- Ao reconstruir uma rotina: atualizar `re/tools/rust_status.json` (done/partial), regenerar inventário,
  `clippy` sem avisos, testes novos, `rotinas_*.md`, `CHECKPOINT.md` (mover o anterior para `docs/historico/`),
  relatórios em `docs/relatorios/`.
- Nomes: `python re/tools/match_android_symbols.py <Breach.exe> <libmain.so> re/evidencias/` e
  `python re/tools/gen_nomes_resumo.py`. Prefira o **nome real** (`Pawn::Damage`) ao endereço nos textos novos.

## Fila de trabalho (ver CHECKPOINT.md)
1. Quando chegar a versão 1.2.92/93 do PC: conferir que bate com o Android; migrar endereços pelos nomes;
   reler as rotinas que mudaram; refazer boot, varredura e testes; trocar nomes provisórios por reais.
2. Gráficos e som (decisão 0001): `itb-assets` lê `resource.dat` e `.bank`, extrai localmente com índice;
   som sem depender do FMOD (só 20 funções da API são usadas: ver `re/evidencias/fontes_externas/vita_loader.md`).
3. Etapa 13: tempo de voo dos projéteis, efeito de chegada completo (0x0059C5C0
   `BoardSpace::CheckTerrainDangers`), turno inimigo (M6).
4. Os idiomas são tratados por outra IA, fora deste trabalho.
